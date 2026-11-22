//! #1444: UI pins for the undercut cutter tool editor (dovetail flank
//! angle, undercut kind defaults, hidden corner radius) and the Slot
//! Undercut operation page (`undercut_page.rs`).
//! Provenance: our own structural UI tests over the tool editor and
//! operation page; no vendor material consulted.

use camcraft_app::model;
use camcraft_app::operation::{KindInfo, OpType};
use camcraft_app::tools::ToolKind;
use camcraft_app::undercut;
use egui::accesskit::Role;
use egui_kittest::kittest::NodeT;

use super::*;

fn open_new_tool_dialog() -> Harness<'static, ShellUi> {
    let mut h = Harness::builder()
        .with_size(Vec2::new(1440.0, 960.0))
        .build_ui_state(frame, ShellUi::blank());
    h.run_steps(2);
    h.get_by_label("View").click();
    h.run_steps(2);
    h.get_by_label("Tool library…").click();
    h.run_steps(3);
    h.get_by_label("New tool…").click();
    h.run_steps(3);
    h
}

#[test]
fn new_tool_undercut_kinds_seed_sane_defaults_hide_corner_radius_and_show_dovetail_flank_angle() {
    let cases = [
        ("T-slot cutter", ToolKind::TSlot, "NewTSlot"),
        ("Keyseat cutter", ToolKind::Keyseat, "NewKeyseat"),
        ("Dovetail cutter", ToolKind::Dovetail, "NewDovetail"),
        ("Lollipop cutter", ToolKind::Lollipop, "NewLollipop"),
    ];
    for (label, kind, name) in cases {
        let mut h = open_new_tool_dialog();
        h.query_all_by_role(Role::ComboBox)
            .find(|n| n.value().as_deref() == Some("End mill"))
            .expect("Type combo showing End mill")
            .click();
        h.run_steps(3);
        h.get_by_label(label).click_accesskit();
        h.run_steps(3);

        assert_eq!(h.state().state.tool_ui.tool_kind(), kind);
        assert!(
            h.query_by_label("Corner radius").is_none(),
            "{label} must hide Corner radius"
        );
        assert_eq!(
            h.query_by_label("Flank angle (deg)").is_some(),
            kind == ToolKind::Dovetail,
            "{label} flank angle visibility"
        );

        h.state_mut().state.tool_ui.name = name.into();
        h.run_steps(2);
        h.get_by_label("Add tool").click_accesskit();
        h.run_steps(3);

        assert_eq!(
            h.state().state.error_for(crate::state::ErrorScope::Tool),
            None,
            "{label} should add without error"
        );
        let saved = model::find_by_name(&h.state().state.world, name)
            .and_then(|e| model::tool_of(&h.state().state.world, &e))
            .unwrap_or_else(|| panic!("{name} added to world"));
        assert_eq!(saved.kind, kind);
        assert_eq!(saved.why_not_sane(), None, "{label}: {saved:?}");
        assert!(undercut::Undercut::of(&saved).is_some(), "{label}");
        assert_eq!(saved.corner_r, 0.0, "{label} corner_r stays zero");
    }
}

#[test]
fn dovetail_properties_and_editor_edit_flank_angle_without_corner_radius() {
    let mut h = Harness::builder()
        .with_size(Vec2::new(1440.0, 960.0))
        .build_ui_state(frame, ShellUi::seeded());
    h.run_steps(2);
    let dove = undercut::cutter(ToolKind::Dovetail, 20.0, 5.0, 8.0, 30.0, 60.0);
    h.state_mut().state.pending.push_back(CommandKind::AddTool {
        name: "DT-20-60".into(),
        tool: dove,
    });
    h.run_steps(2);

    crate::actions::target(&mut h.state_mut().state, UiTarget::Tool("DT-20-60".into()));
    h.run_steps(3);
    assert!(h.query_by_label("Flank angle (deg)").is_some(), "Properties shows Flank angle (deg)");
    assert!(h.query_by_label("Corner radius").is_none(), "Properties hides Corner radius for dovetail");

    // #2397: the settings stay in Properties (the shape window holds the
    // body, profile and holder), so the flank angle is edited there.

    let angle_drag = h
        .query_all_by_role(Role::SpinButton)
        .find(|n| n.accesskit_node().numeric_value() == Some(60.0))
        .expect("flank angle SpinButton")
        .rect()
        .center();
    h.drag_at(angle_drag);
    h.run();
    h.hover_at(angle_drag + Vec2::new(-50.0, 0.0));
    h.run();
    h.drop_at(angle_drag + Vec2::new(-50.0, 0.0));
    h.run();
    let staged = h.state().state.tool_ui.tip_angle;
    assert!((staged - 60.0).abs() > 0.5, "dragging flank angle stages tip_angle: {staged}");
    h.get_by_label("Apply").click();
    h.run_steps(3);
    let updated = model::find_by_name(&h.state().state.world, "DT-20-60")
        .and_then(|e| model::tool_of(&h.state().state.world, &e))
        .expect("updated dovetail");
    assert!((updated.tip_angle - staged).abs() < 1e-6, "Apply updates tip_angle in world");
}

#[test]
fn slot_undercut_page_shows_its_geometry_heights_passes_and_feeds_fields() {
    let mut h = shell_harness();
    let idx = KindInfo::ALL
        .iter()
        .position(|k| k.ty == OpType::SlotUndercut)
        .expect("SlotUndercut kind");
    let cur = h.state().state.op_ui.params.clone();
    h.state_mut().state.op_ui.params = crate::operations::retype(&cur, idx);

    let tab_pos = |name: &str| {
        super::super::op_tabs::SECTIONS
            .iter()
            .position(|s| *s == name)
            .unwrap_or_else(|| panic!("tab {name}"))
    };

    h.state_mut().state.op_tab = tab_pos("Geometry");
    h.run_steps(2);
    for label in ["A X", "A Y", "B X", "B Y", "Bulge", "Undercut width", "Entry run-in", "Neck slot width"] {
        assert!(h.query_by_label(label).is_some(), "Geometry shows {label}");
    }

    h.state_mut().state.op_tab = tab_pos("Heights");
    h.run_steps(2);
    for label in ["Clearance Z", "Top Z", "Bottom Z"] {
        assert!(h.query_by_label(label).is_some(), "Heights shows {label}");
    }

    h.state_mut().state.op_tab = tab_pos("Passes");
    h.run_steps(2);
    assert!(h.query_by_label("Stepover").is_some(), "Passes shows Stepover");

    h.state_mut().state.op_tab = tab_pos("Feeds");
    h.run_steps(2);
    for label in ["Cutting feed", "Plunge feed", "Climb milling"] {
        assert!(h.query_by_label(label).is_some(), "Feeds shows {label}");
    }
}
