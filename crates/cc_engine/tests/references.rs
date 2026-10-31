//! Imported plan references persist and remain editable through engine commands.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_core::{ReferenceData, ViewId};
use cc_engine::Session;
use serde_json::json;

fn plan(session: &mut Session) -> ViewId {
    let views = session.execute("views_list", json!({"category":"floor_plan"})).unwrap();
    ViewId(views[0]["id"].as_u64().unwrap())
}

#[test]
fn ascii_dxf_reference_scales_moves_and_undoes() {
    let mut session = Session::default();
    let view = plan(&mut session);
    let path = std::env::temp_dir().join(format!("cc-reference-{}.dxf", std::process::id()));
    std::fs::write(&path, "0\nSECTION\n2\nHEADER\n9\n$INSUNITS\n70\n4\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLINE\n10\n1000\n20\n2000\n11\n5000\n21\n2000\n0\nLWPOLYLINE\n70\n1\n10\n1000\n20\n2000\n10\n1000\n20\n4000\n10\n5000\n20\n4000\n0\nENDSEC\n0\nEOF\n").unwrap();
    session.execute("reference_import", json!({"view":view.0,"path":path})).unwrap();
    let reference = &session.project().view(view).unwrap().references[0];
    assert_eq!(reference.width, 4.0);
    assert!(matches!(&reference.data, ReferenceData::Lines { segments, .. } if segments.len() == 4));
    session.execute("reference_set", json!({"view":view.0,"index":0,"width":8.0,"origin":[2.0,3.0]})).unwrap();
    let reference = &session.project().view(view).unwrap().references[0];
    assert_eq!(reference.width, 8.0);
    assert_eq!(reference.origin.to_array(), [2.0, 3.0]);
    let saved = cc_core::to_ron(session.project()).unwrap();
    assert_eq!(cc_core::from_ron(&saved).unwrap().view(view).unwrap().references[0], *reference);
    session.doc.undo();
    assert_eq!(session.project().view(view).unwrap().references[0].width, 4.0);
    session.execute("reference_remove", json!({"view":view.0,"index":0})).unwrap();
    assert!(session.project().view(view).unwrap().references.is_empty());
    let _ = std::fs::remove_file(path);
}

#[test]
fn jpeg_reference_is_embedded_and_invalid_files_are_rejected() {
    let mut session = Session::default();
    let view = plan(&mut session);
    let path = std::env::temp_dir().join(format!("cc-reference-{}.jpg", std::process::id()));
    let image = image::RgbImage::from_pixel(16, 8, image::Rgb([220, 210, 200]));
    image.save(&path).unwrap();
    session.execute("reference_import", json!({"view":view.0,"path":path,"width":6.0})).unwrap();
    std::fs::remove_file(&path).unwrap();
    let reference = &session.project().view(view).unwrap().references[0];
    assert!(
        matches!(&reference.data, ReferenceData::Image { mime, pixels, encoded } if mime == "image/jpeg" && *pixels == [16,8] && !encoded.is_empty())
    );
    assert_eq!(reference.width, 6.0);
    assert!(
        session
            .execute(
                "reference_calibrate",
                json!({"view":view.0,"index":0,"first":[1.0,1.0],"second":[1.0,1.0],"distance":4.0})
            )
            .is_err()
    );
    session
        .execute(
            "reference_calibrate",
            json!({"view":view.0,"index":0,"first":[1.0,1.0],"second":[3.0,1.0],"distance":4.0}),
        )
        .unwrap();
    let calibrated = &session.project().view(view).unwrap().references[0];
    assert_eq!(calibrated.width, 12.0);
    assert_eq!(calibrated.origin.to_array(), [-1.0, -1.0]);
    session.doc.undo();
    assert_eq!(session.project().view(view).unwrap().references[0].width, 6.0);
    assert!(session.execute("reference_set", json!({"view":view.0,"index":0,"width":0.0})).is_err());
    assert_eq!(session.project().view(view).unwrap().references[0].width, 6.0);
}
