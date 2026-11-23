//! The UCCNC built-in post (#3101): its registry entry and a full app
//! setup posted through it.
//!
//! Gate map (M4.4): covers app::posting (BUILTINS, post_serves, post_setup)
//! with assets/posts/uccnc.cps.

use camcraft_app::commands::{
    AddTool, CreateOperation, CreateSetup, GenerateOperation, History, SetSetupMachine, SetSetupPost, SetStock,
};
use camcraft_app::machine::{parse_mch, Kind};
use camcraft_app::operation::{Height, OpParams, SrcRef};
use camcraft_app::posting;
use camcraft_app::schema::PostRef;
use camcraft_app::stock::StockShape;
use camcraft_app::tools::catalog;
use camcraft_app::world::new_world;

#[test]
fn uccnc_is_registered_and_serves_mills_only() {
    assert!(
        posting::BUILTINS.iter().any(|(name, _)| *name == "UCCNC"),
        "UCCNC must be registered in posting::BUILTINS"
    );
    assert!(posting::post_serves("UCCNC", Kind::Mill));
    for kind in [Kind::Lathe, Kind::Plasma, Kind::WireEdm, Kind::SinkerEdm, Kind::Waterjet] {
        assert!(!posting::post_serves("UCCNC", kind), "UCCNC must not serve {kind:?}");
    }
}

#[test]
fn uccnc_posts_a_face_setup_with_its_tool_change_and_arc_mode() {
    let world = new_world();
    let mut history = History::default();
    history.execute(&world, CreateSetup::new(&world, None, None).expect("Setup1"));

    let vmc_json = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/machines/3axis-vmc.mch"),
    )
    .expect("3axis-vmc.mch reads");
    let vmc = parse_mch(&vmc_json).expect("3axis-vmc parses");
    history.execute(
        &world,
        SetSetupMachine::new(&world, "Setup1", Some(("3axis-vmc.mch".into(), Some(vmc)))).expect("valid"),
    );
    let stock = SetStock::new(&world, StockShape::Box { dx: 60.0, dy: 60.0, dz: 20.0 }, Default::default())
        .expect("stock");
    history.execute(&world, stock);

    let tool_name = "EM-0600-4FL";
    let tool = catalog()
        .into_iter()
        .find(|(n, _)| *n == tool_name)
        .map(|(_, t)| t)
        .expect("catalog tool");
    history.execute(&world, AddTool::new(&world, tool_name, tool).expect("add tool"));
    let params = OpParams::Face {
        holes: Default::default(),
        blend_turns: false,
        tilt: None,
        stepover: 3.0,
        clearance_z: Height::fixed(25.0),
        cut_z: Height::fixed(19.5),
        top_z: None,
        stepdown: None,
        feed: 1200.0,
        plunge_feed: 300.0,
        climb: true,
    };
    history.execute(&world, CreateOperation::new(&world, "Face1", tool_name, SrcRef::Stock, params).expect("create"));
    history.execute(&world, GenerateOperation::new(&world, "Face1").expect("generate"));
    history.execute(
        &world,
        SetSetupPost::new(&world, "Setup1", Some(PostRef::Named("UCCNC".into()))).expect("set post"),
    );

    let dir = std::env::temp_dir().join(format!("camcraft-3101-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let out = posting::post_setup(&world, &dir.join("uccnc_setup.nc")).expect("UCCNC setup posts");
    let p = &out.program;
    // The header states the unit and the incremental arc-centre mode: the
    // control's own IJ setting never decides where an arc goes.
    assert!(p.contains("G21 G17 G90 G91.1 G94 G40 G49 G80"), "header modal line:\n{p}");
    // The M6 macro of the UCCNC profile runs on its own line, then the
    // tool table's length.
    assert!(p.lines().any(|l| l.trim() == "T1 M6"), "T1 M6 on its own line:\n{p}");
    assert!(p.lines().any(|l| l.trim() == "G43 H1"), "G43 H1 after the tool change:\n{p}");
    assert!(p.trim_end().ends_with("M30"), "the program ends with M30:\n{p}");
}
