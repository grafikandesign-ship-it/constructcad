//! Drawing layers: command access, view visibility, undo and persistence.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_core::{ElementId, ViewId};
use cc_engine::Session;
use serde_json::{Value, json};

fn run(session: &mut Session, name: &str, args: Value) -> Value {
    session.execute(name, args).unwrap_or_else(|error| panic!("{name}: {error:?}"))
}

fn drawn(session: &mut Session, view: ViewId, id: ElementId) -> bool {
    session.drawing(view).unwrap().prims.iter().any(|prim| prim.element() == Some(id))
}

#[test]
fn layers_control_existing_and_new_elements_across_views() {
    let mut session = Session::default();
    let plan = ViewId(run(&mut session, "views_list", json!({"category": "floor_plan"}))[0]["id"].as_u64().unwrap());
    let three_d = ViewId(run(&mut session, "views_list", json!({"category": "three_d"}))[0]["id"].as_u64().unwrap());
    let wall = ElementId(
        run(&mut session, "wall_create", json!({"start": [0.0, 0.0], "end": [4.0, 0.0]}))["id"].as_u64().unwrap(),
    );
    assert_eq!(session.project().layer_of(wall), "A-Muri");
    let door =
        ElementId(run(&mut session, "door_create", json!({"wall": wall.0, "offset": 2.0}))["id"].as_u64().unwrap());
    assert_eq!(session.project().layer_of(door), "A-Porte");
    run(&mut session, "layer_create", json!({"name": "Existing"}));
    run(&mut session, "elements_set_layer", json!({"ids": [wall.0], "layer": "Existing"}));
    assert_eq!(session.project().layer_of(wall), "Existing");
    assert!(drawn(&mut session, plan, wall));

    run(&mut session, "layer_set_visible", json!({"name": "Existing", "visible": false}));
    assert!(!drawn(&mut session, plan, wall));
    session.sync();
    let project = session.project();
    let view = project.view(three_d).unwrap();
    let mut resolver = cc_views::graphics::Resolver::new(project, session.geom(), view, None);
    assert!(resolver.style(wall).is_none());
    session.doc.undo();
    assert!(drawn(&mut session, plan, wall));

    run(&mut session, "layer_activate", json!({"name": "Existing"}));
    let next = ElementId(
        run(&mut session, "wall_create", json!({"start": [0.0, 2.0], "end": [4.0, 2.0]}))["id"].as_u64().unwrap(),
    );
    assert_eq!(session.project().layer_of(next), "Existing");
    let text = cc_core::to_ron(session.project()).unwrap();
    let restored = cc_core::from_ron(&text).unwrap();
    assert_eq!(restored.layer_of(next), "Existing");
    assert_eq!(restored.current_layer, "Existing");

    run(&mut session, "layer_rename", json!({"name": "Existing", "new_name": "Walls"}));
    assert_eq!(session.project().layer_of(wall), "Walls");
    run(&mut session, "layer_delete", json!({"name": "Walls"}));
    assert_eq!(session.project().layer_of(wall), "0");
    assert_eq!(session.project().layer_of(next), "0");
}

#[test]
fn legacy_architectural_elements_are_assigned_when_opened() {
    let mut session = Session::default();
    let wall = ElementId(
        run(&mut session, "wall_create", json!({"start": [0.0, 0.0], "end": [4.0, 0.0]}))["id"].as_u64().unwrap(),
    );
    let mut legacy = session.project().clone();
    legacy.model_layers.clear();
    legacy.element_layers.clear();
    legacy.automatic_layers_initialized = false;
    let text = cc_core::to_ron(&legacy).unwrap();
    let (loaded, repairs) = cc_core::from_ron_with_repairs(&text).unwrap();
    assert_eq!(repairs, 0);
    assert_eq!(loaded.layer_of(wall), "A-Muri");
    assert!(loaded.model_layers.contains("A-Muri"));
    assert!(loaded.automatic_layers_initialized);
}
