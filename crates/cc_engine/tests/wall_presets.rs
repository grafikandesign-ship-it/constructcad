//! Wall presets become ordinary editable project types in one undoable step.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_core::{Category, TypeData, WallFunction};
use cc_engine::Session;
use serde_json::json;

#[test]
fn catalog_installs_editable_exterior_and_interior_types() {
    let mut session = Session::default();
    let initial = session.project().types_of(Category::Wall).count();
    let catalog = session.execute("wall_presets_list", json!({})).unwrap();
    let builtin: Vec<_> =
        catalog.as_array().unwrap().iter().filter(|p| p["id"].as_str().unwrap().starts_with("WP-")).collect();
    assert_eq!(builtin.len(), 7);
    assert_eq!(builtin.iter().filter(|p| p["group"] == "exterior").count(), 2);
    for entry in builtin {
        let id = entry["id"].as_str().unwrap();
        let created = session.execute("wall_preset_add", json!({"preset":id})).unwrap();
        assert_eq!(created["created"], true);
        let ty = session.project().element_type(cc_core::ElementTypeId(created["id"].as_u64().unwrap())).unwrap();
        assert_eq!(ty.info.type_mark, id);
        assert!((ty.thickness() - entry["thickness"].as_f64().unwrap()).abs() < 1e-9);
        assert!(ty.layers().iter().all(|layer| session.project().material(&layer.material).is_some()));
        assert!(
            matches!(&ty.data, TypeData::Wall { function: WallFunction::Exterior, .. })
                == (entry["group"] == "exterior")
        );
        assert_eq!(session.execute("wall_preset_add", json!({"preset":id})).unwrap()["created"], false);
    }
    assert_eq!(session.project().types_of(Category::Wall).count(), initial + 7);
    let saved = cc_core::to_ron(session.project()).unwrap();
    assert_eq!(cc_core::from_ron(&saved).unwrap().types_of(Category::Wall).count(), initial + 7);
    session.doc.undo();
    assert_eq!(session.project().types_of(Category::Wall).count(), initial + 6);
}

#[test]
fn archline_catalog_has_unique_ids_and_imports_layer_widths() {
    let mut session = Session::default();
    let initial = session.project().types_of(Category::Wall).count();
    let catalog = session.execute("wall_presets_list", json!({})).unwrap();
    let archline: Vec<_> =
        catalog.as_array().unwrap().iter().filter(|p| p["id"].as_str().unwrap().starts_with("AL24-")).collect();
    assert_eq!(archline.len(), 35);
    let ids: std::collections::BTreeSet<_> = archline.iter().map(|p| p["id"].as_str().unwrap()).collect();
    assert_eq!(ids.len(), archline.len());
    let interior = archline.iter().find(|p| p["name"] == "ARCHLine · INTERNO 12 Strati").unwrap();
    assert_eq!(interior["group"], "interior");
    assert!((interior["thickness"].as_f64().unwrap() - 0.12).abs() < 1e-9);
    let result = session.execute("wall_preset_add", json!({"preset": interior["id"]})).unwrap();
    let ty = session.project().element_type(cc_core::ElementTypeId(result["id"].as_u64().unwrap())).unwrap();
    let widths: Vec<_> = ty.layers().iter().map(|layer| layer.thickness).collect();
    assert_eq!(widths, vec![0.02, 0.08, 0.02]);
    assert_eq!(ty.layers()[1].material, "ARCHLine · Terca_0043");
    assert_eq!(session.execute("wall_preset_add", json!({"preset": interior["id"]})).unwrap()["created"], false);
    for preset in archline {
        session.execute("wall_preset_add", json!({"preset": preset["id"]})).unwrap();
    }
    assert_eq!(session.project().types_of(Category::Wall).count(), initial + 35);
}
