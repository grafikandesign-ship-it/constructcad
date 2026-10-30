//! Named drawing layers shared by plans, 3D views and sheets.

use super::edit;
use crate::error::{EngineError, Result};
use crate::refs;
use crate::registry::{Command, CommandKind};
use crate::session::Session;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

fn check_name(name: &str) -> Result<()> {
    if name.trim() != name || name.is_empty() || name.len() > 80 {
        return Err(EngineError::invalid("layer name must be 1–80 characters without surrounding spaces"));
    }
    Ok(())
}

/// List named drawing layers, their visibility and the current layer.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LayersList {}

impl Command for LayersList {
    const NAME: &'static str = "layers_list";
    const KIND: CommandKind = CommandKind::Query;
    fn run(self, s: &mut Session) -> Result<Value> {
        let p = s.project();
        let names = std::iter::once("0").chain(p.model_layers.iter().map(String::as_str));
        Ok(json!({
            "current": p.current_layer,
            "layers": names.map(|name| json!({"name": name, "visible": !p.hidden_layers.contains(name)})).collect::<Vec<_>>()
        }))
    }
}

/// Create a named drawing layer.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LayerCreate {
    /// New layer name.
    pub name: String,
}

impl Command for LayerCreate {
    const NAME: &'static str = "layer_create";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        check_name(&self.name)?;
        if s.project().layer_exists(&self.name) {
            return Err(EngineError::invalid("layer already exists"));
        }
        edit(s, "Create layer".into(), |e| {
            e.layer_settings_mut().model_layers.insert(self.name.clone());
            Ok(())
        })?;
        Ok(json!({"name": self.name}))
    }
}

/// Rename a drawing layer and all element assignments to it.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LayerRename {
    /// Existing layer name (layer `0` cannot be renamed).
    pub name: String,
    /// New layer name.
    pub new_name: String,
}

impl Command for LayerRename {
    const NAME: &'static str = "layer_rename";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        check_name(&self.new_name)?;
        if self.name == "0" || !s.project().layer_exists(&self.name) {
            return Err(EngineError::not_found("layer"));
        }
        if self.name != self.new_name && s.project().layer_exists(&self.new_name) {
            return Err(EngineError::invalid("layer already exists"));
        }
        edit(s, "Rename layer".into(), |e| {
            let p = e.layer_settings_mut();
            p.model_layers.remove(&self.name);
            p.model_layers.insert(self.new_name.clone());
            for layer in p.element_layers.values_mut().filter(|name| **name == self.name) {
                *layer = self.new_name.clone();
            }
            if p.hidden_layers.remove(&self.name) {
                p.hidden_layers.insert(self.new_name.clone());
            }
            if p.current_layer == self.name {
                p.current_layer = self.new_name.clone();
            }
            Ok(())
        })?;
        Ok(json!({"name": self.new_name}))
    }
}

/// Delete a drawing layer, moving its elements to layer `0`.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LayerDelete {
    /// Layer name (layer `0` cannot be deleted).
    pub name: String,
}

impl Command for LayerDelete {
    const NAME: &'static str = "layer_delete";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        if self.name == "0" || !s.project().layer_exists(&self.name) {
            return Err(EngineError::not_found("layer"));
        }
        edit(s, "Delete layer".into(), |e| {
            let p = e.layer_settings_mut();
            p.model_layers.remove(&self.name);
            p.hidden_layers.remove(&self.name);
            p.element_layers.retain(|_, name| name != &self.name);
            if p.current_layer == self.name {
                p.current_layer = "0".into();
            }
            Ok(())
        })?;
        Ok(json!({"deleted": self.name}))
    }
}

/// Show or hide a drawing layer in every graphical view.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LayerSetVisible {
    /// Layer name.
    pub name: String,
    /// Whether the layer is shown.
    pub visible: bool,
}

impl Command for LayerSetVisible {
    const NAME: &'static str = "layer_set_visible";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        if !s.project().layer_exists(&self.name) {
            return Err(EngineError::not_found("layer"));
        }
        edit(s, "Layer visibility".into(), |e| {
            let hidden = &mut e.layer_settings_mut().hidden_layers;
            if self.visible {
                hidden.remove(&self.name);
            } else {
                hidden.insert(self.name.clone());
            }
            Ok(())
        })?;
        Ok(json!({"name": self.name, "visible": self.visible}))
    }
}

/// Make a drawing layer current for newly created elements.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LayerActivate {
    /// Layer name.
    pub name: String,
}

impl Command for LayerActivate {
    const NAME: &'static str = "layer_activate";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        if !s.project().layer_exists(&self.name) {
            return Err(EngineError::not_found("layer"));
        }
        edit(s, "Current layer".into(), |e| {
            e.layer_settings_mut().current_layer = self.name.clone();
            Ok(())
        })?;
        Ok(json!({"current": self.name}))
    }
}

/// Assign existing elements to a drawing layer.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ElementsSetLayer {
    /// Element ids.
    pub ids: Vec<u64>,
    /// Layer name.
    pub layer: String,
}

impl Command for ElementsSetLayer {
    const NAME: &'static str = "elements_set_layer";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        if !s.project().layer_exists(&self.layer) {
            return Err(EngineError::not_found("layer"));
        }
        let ids = refs::elements(s.project(), &self.ids)?;
        edit(s, "Assign layer".into(), |e| {
            let assignments = &mut e.layer_settings_mut().element_layers;
            for id in &ids {
                if self.layer == "0" {
                    assignments.remove(id);
                } else {
                    assignments.insert(*id, self.layer.clone());
                }
            }
            Ok(())
        })?;
        Ok(json!({"assigned": ids.len(), "layer": self.layer}))
    }
}
