//! #2016: a library holder is the tool's holder by reference. Apply
//! holder stores the entry's name and the two lengths; the Properties
//! tool settings then show the holder cylinder the laws read for that
//! entry instead of two fields to size by hand; the Profile tab names
//! the entry and its picker shows it; the unchanged outline applied
//! again keeps the reference, an edited point makes the holder a drawn
//! profile and the two fields come back. Provenance: our own page law.
//! Gate map: covers ui::egui_app::tool_settings, ui::egui_app::tool_profile_page,
//! ui::egui_app::tool_holder_picker, ui::state (tool profile).

use super::*;
use camcraft_app::holder_library::HolderRef;
use egui_kittest::kittest::NodeT;

/// A 6 mm end mill on an 8 mm shank (flutes 15, shank 40).
const TOOL: &str = "EM-0600-4FL";
const FROM: &str = "from the library holder BT30 ER20";

fn reference(state: &State) -> Option<HolderRef> {
    model::find_by_name(&state.world, TOOL)
        .and_then(|e| camcraft_app::tool_body::body_of(&state.world, &e))
        .and_then(|b| b.library)
}

fn properties() -> Harness<'static, ShellUi> {
    let mut h = shell_harness();
    h.run();
    crate::egui_app::browser_pane::click(&mut h.state_mut().state, panes::Leaf::Tool(TOOL.into()));
    h.run_steps(3);
    h
}

/// The values of the typed number fields on screen.
fn spins(h: &Harness<'static, ShellUi>) -> Vec<String> {
    h.query_all_by_role(egui::accesskit::Role::SpinButton).filter_map(|n| n.value()).collect()
}

/// Are the tool's own holder diameter and length (the catalog's
/// Ø44 x 35) on screen as typed fields?
fn typed(h: &Harness<'static, ShellUi>) -> bool {
    let s = spins(h);
    s.iter().any(|v| v == "44.0 mm") && s.iter().any(|v| v == "35.0 mm")
}

fn set_bt30(h: &mut Harness<'static, ShellUi>) {
    let cmd = CommandKind::SetToolHolder { name: TOOL.into(), entry: "bt30 er20".into(), gauge: None, projection: None };
    h.state_mut().state.pending.push_back(cmd);
    h.run_steps(3);
}

fn apply_profile(h: &mut Harness<'static, ShellUi>, edit: impl Fn(&mut Vec<camcraft_app::spun::SpunPoint>)) {
    let st = &mut h.state_mut().state;
    st.sync_tool_profile_ui();
    edit(&mut st.tool_profile_ui.points);
    st.queue_apply_tool_profile();
    h.run_steps(3);
}

#[test]
fn a_library_holder_sizes_the_holder_fields_until_its_outline_is_edited() {
    let mut h = properties();
    assert!(h.query_all_by_label_contains("from the library holder").next().is_none(), "a holder cylinder is typed");
    assert!(typed(&h), "the catalog's Ø44 x 35 holder cylinder is typed: {:?}", spins(&h));
    set_bt30(&mut h);
    let want = HolderRef { entry: "BT30 ER20".into(), gauge_len: 60.0, projection: 55.0 };
    assert_eq!(reference(&h.state().state), Some(want.clone()), "status: {}", h.state().state.status);
    assert_eq!(h.query_all_by_label_contains(FROM).count(), 2, "holder diameter and length name the entry");
    assert!(h.query_all_by_label_contains("46.000 mm").next().is_some(), "the BT30 flange is the holder diameter");
    assert!(h.query_all_by_label_contains("60.000 mm").next().is_some(), "nose face to gauge line is its length");
    assert!(!typed(&h), "the two fields are not typed: {:?}", spins(&h));
    // The outline as it stands, applied again: the same holder.
    apply_profile(&mut h, |_| {});
    assert_eq!(reference(&h.state().state), Some(want), "status: {}", h.state().state.status);
    // One point moved: a drawn profile of its own, sized by hand again.
    apply_profile(&mut h, |p| {
        p[1].r += 1.0;
        p[2].r += 1.0;
    });
    assert_eq!(reference(&h.state().state), None, "status: {}", h.state().state.status);
    assert!(h.query_all_by_label_contains("from the library holder").next().is_none());
    assert!(typed(&h), "{:?}", spins(&h));
    h.state_mut().state.pending.push_back(CommandKind::Undo);
    h.run_steps(3);
    assert!(reference(&h.state().state).is_some(), "undo: the library holder again");
    assert_eq!(h.query_all_by_label_contains(FROM).count(), 2);
}

#[test]
fn the_profile_tab_names_the_entry_and_the_picker_shows_it() {
    let mut h = properties();
    set_bt30(&mut h);
    crate::actions::open_tool_editor(&mut h.state_mut().state, TOOL);
    h.run_steps(2);
    h.get_by_label("Profile").click();
    h.run_steps(2);
    assert!(h.query_all_by_label_contains("landed: library holder BT30 ER20").next().is_some());
    let d = &h.state().state.tool_profile_ui;
    assert_eq!((d.library.as_deref(), d.gauge, d.projection), (Some("BT30 ER20"), 60.0, 55.0), "the picker shows the tool's holder");
    assert!(!h.get_by_label("Apply holder").accesskit_node().is_disabled());
}
