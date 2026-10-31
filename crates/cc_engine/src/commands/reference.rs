//! Embedded 2D plan references from PDF, JPEG and ASCII DXF files.

use super::edit;
use crate::error::{EngineError, Result};
use crate::refs::{self, Point, Ref};
use crate::registry::{Command, CommandKind};
use crate::session::Session;
use base64::Engine as _;
use cc_core::{PlanReference, ReferenceData};
use glam::DVec2;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::PathBuf;

const MAX_FILE: u64 = 32 * 1024 * 1024;
const MAX_SEGMENTS: usize = 200_000;

/// Import a PDF page, JPEG image or ASCII DXF as a persistent reference behind a floor plan.
/// Images default to 10 m wide; DXF defaults to its measured width. Adjust with reference_set.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReferenceImport {
    /// Floor plan id or name.
    pub view: Ref,
    /// PDF, JPG/JPEG or DXF file.
    pub path: PathBuf,
    /// Desired width in meters (optional for DXF).
    #[serde(default)]
    pub width: Option<f64>,
    /// Bottom-left position in plan meters; defaults to the drawing origin.
    #[serde(default)]
    pub origin: Option<Point>,
    /// PDF page number, starting at one (default one).
    #[serde(default)]
    pub page: Option<u32>,
}

impl Command for ReferenceImport {
    const NAME: &'static str = "reference_import";
    const KIND: CommandKind = CommandKind::File;
    fn run(self, s: &mut Session) -> Result<Value> {
        let view_id = refs::view(s.project(), &self.view)?;
        if s.project().view(view_id).and_then(|v| v.level()).is_none() {
            return Err(EngineError::invalid("references can only be imported into plan views"));
        }
        let origin = refs::point(self.origin.unwrap_or([0.0, 0.0]))?;
        let page = self.page.unwrap_or(1);
        if page == 0 || page > 10_000 {
            return Err(EngineError::invalid("PDF page must be between 1 and 10000"));
        }
        let size = std::fs::metadata(&self.path)?.len();
        if size == 0 || size > MAX_FILE {
            return Err(EngineError::invalid("reference file must be between 1 byte and 32 MB"));
        }
        let ext = self.path.extension().and_then(|v| v.to_str()).unwrap_or_default().to_ascii_lowercase();
        let (data, default_width) = match ext.as_str() {
            "jpg" | "jpeg" => (image_data(std::fs::read(&self.path)?, "image/jpeg")?, 10.0),
            "pdf" => (image_data(render_pdf(&self.path, page)?, "image/png")?, 10.0),
            "dxf" => {
                let (segments, native_width) = parse_dxf(&std::fs::read(&self.path)?)?;
                (ReferenceData::Lines { segments, native_width }, native_width)
            }
            _ => return Err(EngineError::invalid("supported references: PDF, JPG/JPEG and DXF")),
        };
        let width = refs::positive(self.width.unwrap_or(default_width), "reference width")?;
        if width > 100_000.0 {
            return Err(EngineError::invalid("reference width is too large"));
        }
        let name = self.path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| ext.clone());
        let reference = PlanReference { name, origin, width, visible: true, data };
        let index = edit(s, "Import plan reference".into(), |e| {
            let view = e.view_mut(view_id).ok_or_else(|| EngineError::not_found("view"))?;
            if view.references.len() >= 50 {
                return Err(EngineError::rejected("a plan can contain at most 50 references"));
            }
            let index = view.references.len();
            view.references.push(reference);
            Ok(index)
        })?;
        Ok(json!({"view": view_id.0, "index": index, "format": ext, "width": width}))
    }
}

/// Change the position, calibrated width or visibility of an imported plan reference.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReferenceSet {
    /// Floor plan id or name.
    pub view: Ref,
    /// Zero-based reference index within the view.
    pub index: usize,
    /// Width in meters.
    #[serde(default)]
    pub width: Option<f64>,
    /// Bottom-left position in plan meters.
    #[serde(default)]
    pub origin: Option<Point>,
    /// Whether the reference is shown.
    #[serde(default)]
    pub visible: Option<bool>,
}

impl Command for ReferenceSet {
    const NAME: &'static str = "reference_set";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let id = refs::view(s.project(), &self.view)?;
        let width = self.width.map(|v| refs::positive(v, "reference width")).transpose()?;
        if width.is_some_and(|v| v > 100_000.0) {
            return Err(EngineError::invalid("reference width is too large"));
        }
        let origin = self.origin.map(refs::point).transpose()?;
        edit(s, "Adjust plan reference".into(), |e| {
            let reference = e
                .view_mut(id)
                .and_then(|v| v.references.get_mut(self.index))
                .ok_or_else(|| EngineError::not_found("reference"))?;
            if let Some(v) = width {
                reference.width = v;
            }
            if let Some(v) = origin {
                reference.origin = v;
            }
            if let Some(v) = self.visible {
                reference.visible = v;
            }
            Ok(())
        })?;
        Ok(json!({"view": id.0, "index": self.index}))
    }
}

/// Calibrate a plan reference from two picked points and their real-world distance.
/// The first point remains fixed while the reference scales.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReferenceCalibrate {
    /// Floor plan id or name.
    pub view: Ref,
    /// Zero-based reference index within the view.
    pub index: usize,
    /// First picked point in current plan meters.
    pub first: Point,
    /// Second picked point in current plan meters.
    pub second: Point,
    /// Actual distance between the two points in meters.
    pub distance: f64,
}

impl Command for ReferenceCalibrate {
    const NAME: &'static str = "reference_calibrate";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let id = refs::view(s.project(), &self.view)?;
        let first = refs::point(self.first)?;
        let second = refs::point(self.second)?;
        let distance = refs::positive(self.distance, "real distance")?;
        let measured = first.distance(second);
        if measured < 1e-6 || distance > 100_000.0 {
            return Err(EngineError::invalid("choose distinct points and a real distance no greater than 100 km"));
        }
        let factor = distance / measured;
        let reference = s
            .project()
            .view(id)
            .and_then(|v| v.references.get(self.index))
            .ok_or_else(|| EngineError::not_found("reference"))?;
        let width = reference.width * factor;
        let origin = first + (reference.origin - first) * factor;
        if !origin.is_finite() || !width.is_finite() || !(0.001..=100_000.0).contains(&width) {
            return Err(EngineError::invalid("the calibrated reference would have an invalid size or position"));
        }
        edit(s, "Calibrate plan reference".into(), |e| {
            let reference = e
                .view_mut(id)
                .and_then(|v| v.references.get_mut(self.index))
                .ok_or_else(|| EngineError::not_found("reference"))?;
            reference.width = width;
            reference.origin = origin;
            Ok(())
        })?;
        Ok(json!({"view": id.0, "index": self.index, "width": width, "origin": origin.to_array()}))
    }
}

/// Remove a plan reference; undo restores its embedded data and placement.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReferenceRemove {
    /// Floor plan id or name.
    pub view: Ref,
    /// Zero-based reference index within the view.
    pub index: usize,
}

impl Command for ReferenceRemove {
    const NAME: &'static str = "reference_remove";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let id = refs::view(s.project(), &self.view)?;
        edit(s, "Remove plan reference".into(), |e| {
            let view = e.view_mut(id).ok_or_else(|| EngineError::not_found("view"))?;
            if self.index >= view.references.len() {
                return Err(EngineError::not_found("reference"));
            }
            view.references.remove(self.index);
            Ok(())
        })?;
        Ok(json!({"view": id.0, "index": self.index}))
    }
}

fn image_data(bytes: Vec<u8>, mime: &str) -> Result<ReferenceData> {
    if bytes.len() > MAX_FILE as usize {
        return Err(EngineError::invalid("image is too large"));
    }
    let image = image::load_from_memory(&bytes).map_err(|e| EngineError::invalid(format!("invalid image: {e}")))?;
    let pixels = [image.width(), image.height()];
    if pixels[0] == 0 || pixels[1] == 0 || u64::from(pixels[0]) * u64::from(pixels[1]) > 40_000_000 {
        return Err(EngineError::invalid("image dimensions exceed 40 megapixels"));
    }
    Ok(ReferenceData::Image {
        mime: mime.into(),
        encoded: base64::engine::general_purpose::STANDARD.encode(bytes),
        pixels,
    })
}

fn render_pdf(path: &PathBuf, page: u32) -> Result<Vec<u8>> {
    let output = std::process::Command::new("pdftoppm")
        .args(["-f", &page.to_string(), "-l", &page.to_string(), "-singlefile", "-scale-to", "2400", "-png"])
        .arg(path)
        .output()
        .map_err(|e| EngineError::io(format!("PDF renderer pdftoppm unavailable: {e}")))?;
    if !output.status.success() || output.stdout.is_empty() {
        return Err(EngineError::invalid(format!(
            "PDF page could not be rendered: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(output.stdout)
}

fn parse_dxf(bytes: &[u8]) -> Result<(Vec<[DVec2; 2]>, f64)> {
    if bytes.starts_with(b"AutoCAD Binary DXF") {
        return Err(EngineError::invalid("binary DXF is not supported"));
    }
    let content = String::from_utf8_lossy(bytes);
    let lines: Vec<&str> = content.lines().collect();
    let pairs: Vec<(&str, &str)> = lines.as_chunks::<2>().0.iter().map(|c| (c[0].trim(), c[1].trim())).collect();
    let mut unit = 0_i32;
    for window in pairs.windows(2) {
        if window[0] == ("9", "$INSUNITS") && window[1].0 == "70" {
            unit = window[1].1.parse().unwrap_or(0);
        }
    }
    let factor = match unit {
        1 => 0.0254,
        2 => 0.3048,
        4 | 0 => 0.001,
        5 => 0.01,
        6 => 1.0,
        14 => 0.1,
        _ => 0.001,
    };
    let mut in_entities = false;
    let mut pending_section = false;
    let mut segments = Vec::new();
    let mut index = 0;
    while index < pairs.len() {
        let (code, value) = pairs[index];
        if code == "0" && value == "SECTION" {
            pending_section = true;
            index += 1;
            continue;
        }
        if pending_section && code == "2" {
            in_entities = value == "ENTITIES";
            pending_section = false;
            index += 1;
            continue;
        }
        if code == "0" && value == "ENDSEC" {
            in_entities = false;
        }
        if in_entities && code == "0" && matches!(value, "LINE" | "LWPOLYLINE" | "CIRCLE" | "ARC") {
            let start = index + 1;
            index = start;
            while index < pairs.len() && pairs[index].0 != "0" {
                index += 1;
            }
            let entity = &pairs[start..index];
            parse_entity(value, entity, factor, &mut segments);
            if segments.len() > MAX_SEGMENTS {
                return Err(EngineError::invalid("DXF contains too many segments"));
            }
            continue;
        }
        index += 1;
    }
    if segments.is_empty() {
        return Err(EngineError::invalid("DXF contains no supported 2D LINE, LWPOLYLINE, CIRCLE or ARC entities"));
    }
    let min = segments.iter().flat_map(|s| s.iter()).fold(DVec2::splat(f64::INFINITY), |m, p| m.min(*p));
    let max = segments.iter().flat_map(|s| s.iter()).fold(DVec2::splat(f64::NEG_INFINITY), |m, p| m.max(*p));
    let width = (max.x - min.x).max(0.001);
    if !min.is_finite() || !max.is_finite() || width > 100_000.0 {
        return Err(EngineError::invalid("DXF coordinates are invalid or too large"));
    }
    for segment in &mut segments {
        for point in segment {
            *point -= min;
        }
    }
    Ok((segments, width))
}

fn value(entity: &[(&str, &str)], code: &str) -> Option<f64> {
    entity.iter().find(|(c, _)| *c == code).and_then(|(_, v)| v.parse::<f64>().ok()).filter(|v| v.is_finite())
}

fn parse_entity(kind: &str, entity: &[(&str, &str)], factor: f64, out: &mut Vec<[DVec2; 2]>) {
    let point = |x, y| Some(DVec2::new(value(entity, x)?, value(entity, y)?) * factor);
    match kind {
        "LINE" => {
            if let (Some(a), Some(b)) = (point("10", "20"), point("11", "21"))
                && a != b
            {
                out.push([a, b]);
            }
        }
        "LWPOLYLINE" => {
            let mut vertices = Vec::new();
            let mut x = None;
            for (code, raw) in entity {
                if *code == "10" {
                    x = raw.parse::<f64>().ok();
                }
                if *code == "20"
                    && let (Some(x), Ok(y)) = (x.take(), raw.parse::<f64>())
                {
                    let p = DVec2::new(x, y) * factor;
                    if p.is_finite() {
                        vertices.push(p);
                    }
                }
            }
            for edge in vertices.windows(2) {
                if edge[0] != edge[1] {
                    out.push([edge[0], edge[1]]);
                }
            }
            let closed = entity.iter().any(|(c, v)| *c == "70" && v.parse::<u32>().is_ok_and(|flags| flags & 1 != 0));
            if closed && vertices.len() > 2 && vertices[0] != vertices[vertices.len() - 1] {
                out.push([vertices[vertices.len() - 1], vertices[0]]);
            }
        }
        "CIRCLE" | "ARC" => {
            if let (Some(center), Some(radius)) = (point("10", "20"), value(entity, "40")) {
                let radius = radius * factor;
                if radius <= 0.0 || !radius.is_finite() {
                    return;
                }
                let (start, end) = if kind == "CIRCLE" {
                    (0.0, 360.0)
                } else {
                    (value(entity, "50").unwrap_or(0.0), value(entity, "51").unwrap_or(0.0))
                };
                let sweep = (end - start).rem_euclid(360.0);
                let sweep = if kind == "CIRCLE" || sweep == 0.0 { 360.0 } else { sweep };
                let count = ((sweep / 10.0).ceil() as usize).clamp(4, 72);
                for i in 0..count {
                    let at = |j: usize| {
                        let angle = (start + sweep * j as f64 / count as f64).to_radians();
                        center + DVec2::new(angle.cos(), angle.sin()) * radius
                    };
                    out.push([at(i), at(i + 1)]);
                }
            }
        }
        _ => {}
    }
}
