//! Small schematic wall type catalog. A preset is copied into a project only when chosen.

use super::edit;
use crate::error::{EngineError, Result};
use crate::registry::{Command, CommandKind};
use crate::session::Session;
use cc_core::{
    ElementType, ElementTypeId, Layer, LayerFunction, Material, MaterialClass, Pattern, PatternFill, TypeData,
    TypeInfo, WallFunction,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy)]
pub enum MaterialRole {
    Plaster,
    Masonry,
    Concrete,
    Gypsum,
    Insulation,
    Cavity,
    Wood,
    Generic,
}

#[derive(Debug, Clone, Copy)]
pub struct LayerSpec {
    pub role: MaterialRole,
    pub thickness: f64,
    pub function: LayerFunction,
}

#[derive(Debug, Clone, Copy)]
pub struct WallPreset {
    pub id: &'static str,
    pub name_key: Option<&'static str>,
    pub name: &'static str,
    pub category: WallPresetCategory,
    pub function: WallFunction,
    pub layers: &'static [LayerSpec],
    pub material_names: Option<&'static [&'static str]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallPresetCategory {
    Exterior,
    Interior,
    Concrete,
    Timber,
    HollowBrick,
    Generic,
}

impl WallPresetCategory {
    pub const ALL: [Self; 6] =
        [Self::Exterior, Self::Interior, Self::Concrete, Self::Timber, Self::HollowBrick, Self::Generic];

    pub fn id(self) -> &'static str {
        match self {
            Self::Exterior => "exterior",
            Self::Interior => "interior",
            Self::Concrete => "concrete",
            Self::Timber => "timber",
            Self::HollowBrick => "hollow_brick",
            Self::Generic => "generic",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::Exterior => "wall-presets-exterior",
            Self::Interior => "wall-presets-interior",
            Self::Concrete => "wall-presets-concrete",
            Self::Timber => "wall-presets-timber",
            Self::HollowBrick => "wall-presets-hollow-brick",
            Self::Generic => "wall-presets-generic",
        }
    }
}

use LayerFunction::{Finish, Insulation, Structure};
use MaterialRole::{Cavity, Concrete, Generic, Gypsum, Insulation as InsulationMaterial, Masonry, Plaster, Wood};

const EXTERIOR_MASONRY: &[LayerSpec] = &[
    LayerSpec { role: Plaster, thickness: 0.015, function: Finish },
    LayerSpec { role: InsulationMaterial, thickness: 0.14, function: Insulation },
    LayerSpec { role: Masonry, thickness: 0.20, function: Structure },
    LayerSpec { role: Plaster, thickness: 0.015, function: Finish },
];
const EXTERIOR_CONCRETE: &[LayerSpec] = &[
    LayerSpec { role: Plaster, thickness: 0.015, function: Finish },
    LayerSpec { role: InsulationMaterial, thickness: 0.14, function: Insulation },
    LayerSpec { role: Concrete, thickness: 0.20, function: Structure },
    LayerSpec { role: Plaster, thickness: 0.015, function: Finish },
];
const INTERNAL_MASONRY_12: &[LayerSpec] = &[
    LayerSpec { role: Plaster, thickness: 0.015, function: Finish },
    LayerSpec { role: Masonry, thickness: 0.09, function: Structure },
    LayerSpec { role: Plaster, thickness: 0.015, function: Finish },
];
const INTERNAL_MASONRY_15: &[LayerSpec] = &[
    LayerSpec { role: Plaster, thickness: 0.015, function: Finish },
    LayerSpec { role: Masonry, thickness: 0.12, function: Structure },
    LayerSpec { role: Plaster, thickness: 0.015, function: Finish },
];
const INTERNAL_DRYWALL_10: &[LayerSpec] = &[
    LayerSpec { role: Gypsum, thickness: 0.0125, function: Finish },
    LayerSpec { role: Cavity, thickness: 0.075, function: Structure },
    LayerSpec { role: Gypsum, thickness: 0.0125, function: Finish },
];
const INTERNAL_DRYWALL_15: &[LayerSpec] = &[
    LayerSpec { role: Gypsum, thickness: 0.0125, function: Finish },
    LayerSpec { role: Gypsum, thickness: 0.0125, function: Finish },
    LayerSpec { role: Cavity, thickness: 0.10, function: Structure },
    LayerSpec { role: Gypsum, thickness: 0.0125, function: Finish },
    LayerSpec { role: Gypsum, thickness: 0.0125, function: Finish },
];
const INTERNAL_CONCRETE: &[LayerSpec] = &[
    LayerSpec { role: Plaster, thickness: 0.015, function: Finish },
    LayerSpec { role: Concrete, thickness: 0.20, function: Structure },
    LayerSpec { role: Plaster, thickness: 0.015, function: Finish },
];

static PRESETS: &[WallPreset] = &[
    WallPreset {
        id: "WP-E01",
        name_key: Some("wall-preset-exterior-masonry"),
        name: "",
        category: WallPresetCategory::Exterior,
        function: WallFunction::Exterior,
        layers: EXTERIOR_MASONRY,
        material_names: None,
    },
    WallPreset {
        id: "WP-E02",
        name_key: Some("wall-preset-exterior-concrete"),
        name: "",
        category: WallPresetCategory::Exterior,
        function: WallFunction::Exterior,
        layers: EXTERIOR_CONCRETE,
        material_names: None,
    },
    WallPreset {
        id: "WP-I01",
        name_key: Some("wall-preset-interior-masonry-12"),
        name: "",
        category: WallPresetCategory::Interior,
        function: WallFunction::Interior,
        layers: INTERNAL_MASONRY_12,
        material_names: None,
    },
    WallPreset {
        id: "WP-I02",
        name_key: Some("wall-preset-interior-masonry-15"),
        name: "",
        category: WallPresetCategory::Interior,
        function: WallFunction::Interior,
        layers: INTERNAL_MASONRY_15,
        material_names: None,
    },
    WallPreset {
        id: "WP-I03",
        name_key: Some("wall-preset-interior-drywall-10"),
        name: "",
        category: WallPresetCategory::Interior,
        function: WallFunction::Interior,
        layers: INTERNAL_DRYWALL_10,
        material_names: None,
    },
    WallPreset {
        id: "WP-I04",
        name_key: Some("wall-preset-interior-drywall-15"),
        name: "",
        category: WallPresetCategory::Interior,
        function: WallFunction::Interior,
        layers: INTERNAL_DRYWALL_15,
        material_names: None,
    },
    WallPreset {
        id: "WP-I05",
        name_key: Some("wall-preset-interior-concrete"),
        name: "",
        category: WallPresetCategory::Interior,
        function: WallFunction::Interior,
        layers: INTERNAL_CONCRETE,
        material_names: None,
    },
];

include!("archline_wall_presets.rs");

static CATALOG: LazyLock<Vec<WallPreset>> = LazyLock::new(|| PRESETS.iter().chain(ARCHLINE_PRESETS).copied().collect());

pub fn catalog() -> &'static [WallPreset] {
    &CATALOG
}

impl WallPreset {
    pub fn thickness(self) -> f64 {
        self.layers.iter().map(|l| l.thickness).sum()
    }

    pub fn name(self) -> String {
        self.name_key.map_or_else(|| self.name.to_string(), crate::i18n::tr)
    }
}

fn installed(s: &Session, preset: &WallPreset) -> Option<ElementTypeId> {
    s.project().types_of(cc_core::Category::Wall).find(|t| t.info.type_mark == preset.id).map(|t| t.id)
}

/// List wall presets by category with their thickness and installation state.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WallPresetsList {}

impl Command for WallPresetsList {
    const NAME: &'static str = "wall_presets_list";
    const KIND: CommandKind = CommandKind::Query;
    fn run(self, s: &mut Session) -> Result<Value> {
        Ok(json!(
            catalog()
                .iter()
                .map(|p| json!({
                    "id": p.id, "name": p.name(),
                    "group": p.category.id(),
                    "thickness": p.thickness(), "type_id": installed(s, p).map(|id| id.0),
                }))
                .collect::<Vec<_>>()
        ))
    }
}

/// Copy a wall preset into the current project's editable wall types; reuse it when already installed.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WallPresetAdd {
    /// Stable preset code from `wall_presets_list` (for example `WP-E01`).
    pub preset: String,
}

impl Command for WallPresetAdd {
    const NAME: &'static str = "wall_preset_add";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let preset = catalog()
            .iter()
            .find(|p| p.id == self.preset)
            .ok_or_else(|| EngineError::invalid("unknown wall preset"))?;
        if let Some(id) = installed(s, preset) {
            return Ok(json!({"id": id.0, "created": false}));
        }
        let name = preset.name();
        if s.project().types_of(cc_core::Category::Wall).any(|t| t.name == name) {
            return Err(EngineError::invalid("a wall type with this preset name already exists"));
        }
        let id = edit(s, "Add wall preset".into(), |e| {
            let layers = preset
                .layers
                .iter()
                .enumerate()
                .map(|(index, spec)| Layer {
                    material: ensure_material(
                        e,
                        spec.role,
                        preset.material_names.and_then(|names| names.get(index).copied()),
                    ),
                    thickness: spec.thickness,
                    function: spec.function,
                })
                .collect();
            let id = ElementTypeId(e.alloc_id());
            let info = TypeInfo {
                type_mark: preset.id.into(),
                description: crate::i18n::tr(if preset.material_names.is_some() {
                    "wall-preset-description-archline"
                } else {
                    "wall-preset-description"
                }),
                ..Default::default()
            };
            e.add_type(ElementType {
                id,
                name,
                data: TypeData::Wall { layers, function: preset.function },
                info,
                uid: Default::default(),
            });
            Ok(id)
        })?;
        Ok(json!({"id": id.0, "created": true}))
    }
}

fn ensure_material(e: &mut cc_core::EditCtx<'_>, role: MaterialRole, source_name: Option<&str>) -> String {
    let (class, key, color, pattern) = match role {
        Plaster => (MaterialClass::Mortar, "wall-preset-material-plaster", [232, 226, 214], Pattern::Sand),
        Masonry => (MaterialClass::Masonry, "wall-preset-material-masonry", [196, 120, 92], Pattern::Brick),
        Concrete => (MaterialClass::Concrete, "wall-preset-material-concrete", [170, 170, 165], Pattern::Concrete),
        Gypsum => (MaterialClass::Gypsum, "wall-preset-material-gypsum", [245, 245, 240], Pattern::Sand),
        InsulationMaterial => {
            (MaterialClass::Insulation, "wall-preset-material-insulation", [225, 218, 160], Pattern::Insulation)
        }
        Cavity => (MaterialClass::Generic, "wall-preset-material-cavity", [235, 235, 220], Pattern::Insulation),
        Wood => (MaterialClass::Wood, "wall-preset-material-wood", [170, 132, 90], Pattern::Diagonal),
        Generic => (MaterialClass::Generic, "wall-preset-material-generic", [200, 200, 195], Pattern::Diagonal),
    };
    if source_name.is_none()
        && !matches!(role, Cavity | Generic)
        && let Some(existing) = e.project().materials.iter().find(|m| m.identity.class == class)
    {
        return existing.name.clone();
    }
    let name = source_name.map_or_else(|| crate::i18n::tr(key), |source| format!("ARCHLine · {source}"));
    if e.project().material(&name).is_some() {
        return name;
    }
    let mut material = Material::new(name.clone(), color);
    material.identity.class = class;
    material.graphics.cut = PatternFill::lines(pattern);
    e.materials_mut().push(material);
    name
}
