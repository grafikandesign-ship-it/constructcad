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
    assert_eq!(catalog.as_array().unwrap().len(), 7);
    assert_eq!(catalog.as_array().unwrap().iter().filter(|p| p["group"] == "exterior").count(), 2);
    for entry in catalog.as_array().unwrap() {
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
