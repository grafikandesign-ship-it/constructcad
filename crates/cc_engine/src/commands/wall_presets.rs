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

#[derive(Debug, Clone, Copy)]
pub enum MaterialRole {
    Plaster,
    Masonry,
    Concrete,
    Gypsum,
    Insulation,
    Cavity,
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
    pub name_key: &'static str,
    pub function: WallFunction,
    pub layers: &'static [LayerSpec],
}

use LayerFunction::{Finish, Insulation, Structure};
use MaterialRole::{Cavity, Concrete, Gypsum, Insulation as InsulationMaterial, Masonry, Plaster};

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
        name_key: "wall-preset-exterior-masonry",
        function: WallFunction::Exterior,
        layers: EXTERIOR_MASONRY,
    },
    WallPreset {
        id: "WP-E02",
        name_key: "wall-preset-exterior-concrete",
        function: WallFunction::Exterior,
        layers: EXTERIOR_CONCRETE,
    },
    WallPreset {
        id: "WP-I01",
        name_key: "wall-preset-interior-masonry-12",
        function: WallFunction::Interior,
        layers: INTERNAL_MASONRY_12,
    },
    WallPreset {
        id: "WP-I02",
        name_key: "wall-preset-interior-masonry-15",
        function: WallFunction::Interior,
        layers: INTERNAL_MASONRY_15,
    },
    WallPreset {
        id: "WP-I03",
        name_key: "wall-preset-interior-drywall-10",
        function: WallFunction::Interior,
        layers: INTERNAL_DRYWALL_10,
    },
    WallPreset {
        id: "WP-I04",
        name_key: "wall-preset-interior-drywall-15",
        function: WallFunction::Interior,
        layers: INTERNAL_DRYWALL_15,
    },
    WallPreset {
        id: "WP-I05",
        name_key: "wall-preset-interior-concrete",
        function: WallFunction::Interior,
        layers: INTERNAL_CONCRETE,
    },
];

pub fn catalog() -> &'static [WallPreset] {
    PRESETS
}

impl WallPreset {
    pub fn thickness(self) -> f64 {
        self.layers.iter().map(|l| l.thickness).sum()
    }
}

fn installed(s: &Session, preset: &WallPreset) -> Option<ElementTypeId> {
    s.project().types_of(cc_core::Category::Wall).find(|t| t.info.type_mark == preset.id).map(|t| t.id)
}

/// List available exterior and interior wall presets and their installation state.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WallPresetsList {}

impl Command for WallPresetsList {
    const NAME: &'static str = "wall_presets_list";
    const KIND: CommandKind = CommandKind::Query;
    fn run(self, s: &mut Session) -> Result<Value> {
        Ok(json!(
            PRESETS
                .iter()
                .map(|p| json!({
                    "id": p.id, "name": crate::i18n::tr(p.name_key),
                    "group": if p.function == WallFunction::Exterior { "exterior" } else { "interior" },
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
        let preset =
            PRESETS.iter().find(|p| p.id == self.preset).ok_or_else(|| EngineError::invalid("unknown wall preset"))?;
        if let Some(id) = installed(s, preset) {
            return Ok(json!({"id": id.0, "created": false}));
        }
        let name = crate::i18n::tr(preset.name_key);
        if s.project().types_of(cc_core::Category::Wall).any(|t| t.name == name) {
            return Err(EngineError::invalid("a wall type with this preset name already exists"));
        }
        let id = edit(s, "Add wall preset".into(), |e| {
            let layers = preset
                .layers
                .iter()
                .map(|spec| Layer {
                    material: ensure_material(e, spec.role),
                    thickness: spec.thickness,
                    function: spec.function,
                })
                .collect();
            let id = ElementTypeId(e.alloc_id());
            let info = TypeInfo {
                type_mark: preset.id.into(),
                description: crate::i18n::tr("wall-preset-description"),
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

fn ensure_material(e: &mut cc_core::EditCtx<'_>, role: MaterialRole) -> String {
    let (class, key, color, pattern) = match role {
        Plaster => (MaterialClass::Mortar, "wall-preset-material-plaster", [232, 226, 214], Pattern::Sand),
        Masonry => (MaterialClass::Masonry, "wall-preset-material-masonry", [196, 120, 92], Pattern::Brick),
        Concrete => (MaterialClass::Concrete, "wall-preset-material-concrete", [170, 170, 165], Pattern::Concrete),
        Gypsum => (MaterialClass::Gypsum, "wall-preset-material-gypsum", [245, 245, 240], Pattern::Sand),
        InsulationMaterial => {
            (MaterialClass::Insulation, "wall-preset-material-insulation", [225, 218, 160], Pattern::Insulation)
        }
        Cavity => (MaterialClass::Generic, "wall-preset-material-cavity", [235, 235, 220], Pattern::Insulation),
    };
    if !matches!(role, Cavity)
        && let Some(existing) = e.project().materials.iter().find(|m| m.identity.class == class)
    {
        return existing.name.clone();
    }
    let name = crate::i18n::tr(key);
    if e.project().material(&name).is_some() {
        return name;
    }
    let mut material = Material::new(name.clone(), color);
    material.identity.class = class;
    material.graphics.cut = PatternFill::lines(pattern);
    e.materials_mut().push(material);
    name
}
