//! #2002: selected tool settings and the catalog occupy separate dock surfaces.
//! Provenance: our own structural tests over the shipping browser and ribbon handlers.
//! Gate map: covers ui::egui_app::pages, ui::egui_app::layout, ui::egui_app::browser_pane.

use super::*;

const TOOL: &str = "EM-0600-4FL";

fn select_tool(h: &mut Harness<'static, ShellUi>) {
    super::super::super::browser_pane::click(
        &mut h.state_mut().state,
        panes::Leaf::Tool(TOOL.into()),
    );
    h.run_steps(3);
}

/// Whether a node carries `label`. A property row names its control by
/// the row's label (#1119), so a row can match twice; `is_some` on the
/// single-match query would panic.
fn here(h: &Harness<'static, ShellUi>, label: &str) -> bool {
    h.query_all_by_label(label).next().is_some()
}

#[test]
fn setup_tool_list_edit_opens_tool_settings_in_properties() {
    let mut h = shell_harness();
    super::super::super::browser_pane::click(
        &mut h.state_mut().state,
        panes::Leaf::Page(String::new(), crate::state::SetupPage::Tools),
    );
    h.run_steps(3);
    h.get_by_label("Edit").click_accesskit();
    h.run_steps(3);
    assert_eq!(h.state().state.ui_target, UiTarget::Tool(TOOL.into()), "the setup's tool row opens the named Properties subject");
    assert!(h.state().state.tool_shape_editor.is_none(), "the setup tool list edits Properties directly");
    assert!(here(&h, "Diameter"));
    assert!(h.query_by_label("Import library…").is_none());
}

#[test]
fn browser_library_door_and_tool_selection_focus_their_own_dock_tabs() {
    let mut h = shell_harness();
    h.get_by_label("View").click();
    h.run_steps(2);
    h.get_by_label("Tool library…").click();
    h.run_steps(3);
    assert!(h.query_by_label("Import library…").is_some(), "the browser door opens the catalog");
    select_tool(&mut h);
    assert!(here(&h, "Diameter"), "selecting a tool brings Properties in front of the catalog");
    assert!(h.query_by_label("Import library…").is_none(), "the catalog stays behind Properties");
    crate::actions::open_tool_shape_editor(&mut h.state_mut().state, TOOL);
    h.run_steps(2);
    assert!(h.state().state.tool_shape_editor.is_some());
    select_tool(&mut h);
    assert!(h.state().state.tool_shape_editor.is_none(), "browser selection closes the explicit tool editor");
    assert!(here(&h, "Diameter"), "the tool's settings return to Properties");
}

#[test]
fn selected_tool_settings_live_in_properties_without_the_catalog() {
    let mut h = shell_harness();
    select_tool(&mut h);
    assert!(h.state().state.tool_shape_editor.is_none(), "selection must edit Properties directly");
    let properties_x = super::at(&painted(&h), "Properties").x;
    let diameter = h
        .query_all_by_role(egui::accesskit::Role::SpinButton)
        .find(|n| n.value().as_deref() == Some("6.0 mm"))
        .expect("editable tool diameter uses the document value")
        .rect();
    assert!(diameter.min.x > properties_x && diameter.max.x <= 1280.0, "settings belong to Properties");
    for label in ["New tool…", "Import library…", "Import tool holders", "Support features"] {
        assert!(h.query_by_label(label).is_none(), "catalog control {label:?} must not occupy Properties");
    }
    assert_eq!(h.state().state.ui_target, UiTarget::Tool(TOOL.into()));
}

#[test]
fn properties_tool_apply_and_undo_use_the_command_history() {
    let mut h = shell_harness();
    select_tool(&mut h);
    let diameter = |h: &Harness<'static, ShellUi>| {
        model::find_by_name(&h.state().state.world, TOOL)
            .and_then(|e| model::tool_of(&h.state().state.world, &e))
            .expect("selected tool").diameter
    };
    assert!(here(&h, "Diameter"), "Properties exposes the editable diameter");
    let before = diameter(&h);
    let at = h.query_all_by_role(egui::accesskit::Role::SpinButton)
        .find(|n| n.value().as_deref() == Some("6.0 mm"))
        .expect("diameter number").rect().center();
    h.drag_at(at);
    h.run();
    h.hover_at(at + Vec2::new(30.0, 0.0));
    h.run();
    h.drop_at(at + Vec2::new(30.0, 0.0));
    h.run();
    let staged = h.state().state.tool_ui.diameter;
    assert!((staged - (before + 3.0)).abs() < 0.05, "the drag changes the draft");
    assert_eq!(diameter(&h), before, "a draft does not write the world");
    h.get_by_label("Apply").scroll_to_me();
    h.run_steps(3);
    h.get_by_label("Apply").click();
    h.run_steps(2);
    assert_eq!(diameter(&h), staged, "Apply lands the draft through ChangeTool: {}", h.state().state.status);
    crate::actions::undo(&mut h.state_mut().state);
    h.run_steps(2);
    assert_eq!(diameter(&h), before, "Undo restores the tool's previous numbers");
}

#[test]
fn tools_ribbon_opens_a_docked_catalog_without_retargeting_properties() {
    let mut h = shell_harness();
    h.run_steps(2);
    let before = h.state().state.ui_target.clone();
    h.get_by_role_and_label(egui::accesskit::Role::Button, "Tools").click();
    h.run_steps(3);
    assert_eq!(h.state().state.ui_target, before, "opening the catalog preserves the Properties subject");
    assert!(painted(&h).iter().any(|(t, _)| t == "Tool library"), "the catalog has its own dock tab");
    assert!(h.query_by_label("Catalog").is_some(), "the catalog is visible");
    assert_eq!(h.get_by_label("Import tool holders").accesskit_node().role(), egui::accesskit::Role::CheckBox);
    assert_eq!(h.get_by_label("New tool…").accesskit_node().role(), egui::accesskit::Role::Button);
    assert!(!h.query_all_by_role(egui::accesskit::Role::Window).any(|n| n.accesskit_node().label().as_deref() == Some("Tool library")), "the library docks instead of opening a window");
    h.get_by_role_and_label(egui::accesskit::Role::Button, "Tools").click();
    h.run_steps(3);
    assert!(h.query_by_label("Catalog").is_some(), "opening the catalog twice keeps it open");
    assert_eq!(h.state().state.ui_target, before);
}

/// #2014: the tool settings are a property grid - each label left of its
/// input - in the groups Geometry, Holder, Cutting data and Catalog, and
/// each group folds away. The edits still ride Apply (see above).
#[test]
fn tool_settings_form_a_label_left_grid_in_collapsible_groups() {
    let mut h = shell_harness();
    select_tool(&mut h);
    for group in ["Geometry", "Holder", "Cutting data", "Catalog"] {
        assert!(h.query_by_label(group).is_some(), "the {group} group heads its rows");
    }
    let held: [(&str, &[&str]); 4] = [
        ("Geometry", &["Type", "Diameter", "Corner radius", "Flute length", "Shank diameter", "Shank length"]),
        ("Holder", &["Holder diameter", "Holder length"]),
        ("Cutting data", &["Centre cutting (may plunge into solid)"]),
        ("Catalog", &["Material group", "Part number", "Tool number"]),
    ];
    for (group, labels) in held {
        for label in labels {
            assert!(here(&h, label), "{group} holds the {label:?} row");
        }
    }
    // Spindle rpm heads the Cutting data group and also names a column of
    // its per-material table, so it is drawn more than once.
    assert!(h.query_all_by_label("Spindle rpm").next().is_some(), "Cutting data holds the spindle rpm");
    // Every row puts the label on the left of its input, on one line.
    for (label, value) in [
        ("Diameter", "6.0 mm"),
        ("Flute length", "15.0 mm"),
        ("Holder diameter", "44.0 mm"),
        ("Shank length", "40.0 mm"),
    ] {
        let l = super::at(&painted(&h), label);
        let input = h
            .query_all_by_role(egui::accesskit::Role::SpinButton)
            .find(|n| n.value().as_deref() == Some(value))
            .unwrap_or_else(|| panic!("{label:?} shows its input {value:?}"))
            .rect();
        assert!(l.x < input.center().x, "{label:?} label must sit left of its input: {l:?} vs {input:?}");
        assert!((l.y - input.center().y).abs() < 15.0, "{label:?} label and input must share a row: {l:?} vs {input:?}");
    }
    // A group header folds its rows away and brings them back.
    h.get_by_label("Holder").click();
    h.run_steps(2);
    assert!(!here(&h, "Holder diameter"), "a collapsed Holder group hides its rows");
    assert!(here(&h, "Diameter"), "the other groups stay open");
    h.get_by_label("Holder").click();
    h.run_steps(2);
    assert!(here(&h, "Holder diameter"), "an open Holder group shows its rows again");
}
/// #2397: Properties shows the settings and the one side preview, and no
/// body or profile editor. The one button opens the tool shape editor in
/// its own window.
#[test]
fn properties_shows_the_settings_and_the_shape_window_holds_the_shape() {
    let mut h = shell_harness();
    select_tool(&mut h);
    for label in [
        "Holder library",
        "Library holder",
        "Apply holder",
        "Apply profile",
        "Import body…",
        "Apply body",
        "Profile",
        "Draw holder",
    ] {
        assert!(h.query_by_label(label).is_none(), "Properties must not show the shape control {label:?}");
    }
    assert!(h.query_by_label("Edit tool shape").is_some(), "Properties offers the one door to the shape editor");
    h.get_by_label("Edit tool shape").click();
    h.run_steps(3);
    assert_eq!(
        h.state().state.tool_shape_editor.as_deref(),
        Some(TOOL),
        "the door opens the shape window"
    );
    let win = h
        .query_all_by_role(egui::accesskit::Role::Window)
        .find(|n| n.accesskit_node().label().as_deref() == Some("Tool shape: EM-0600-4FL"))
        .expect("the shape window is titled by the tool");
    for label in ["Holder library", "Library holder", "Body", "Profile", "Close"] {
        assert!(win.query_by_label(label).is_some(), "the shape window shows {label:?}");
    }
    // The settings stay on the page behind it.
    assert!(h.query_by_label("Shank diameter").is_some(), "Properties keeps the settings");
}

/// #2007: the Properties tool page keeps its action row (Apply / Add
/// tool beside Close) fixed at the bottom without scrolling, and uses
/// plain labels without "(0 = next free)".
#[test]
fn tool_page_keeps_apply_and_add_tool_beside_close_without_scrolling() {
    let mut h = shell_harness();
    select_tool(&mut h);
    let leaf = super::super::layout::leaf_rect(&h.state().dock, &Tab::Properties).expect("Properties leaf");
    let apply = h.get_by_label("Apply").rect();
    let close = h.get_by_label("Close").rect();
    assert!(leaf.expand(1.0).contains_rect(apply), "Apply {apply:?} must stay visible in Properties {leaf:?} without scrolling");
    assert!(leaf.expand(1.0).contains_rect(close), "Close {close:?} must stay visible in Properties {leaf:?} without scrolling");
    assert!((apply.center().y - close.center().y).abs() < 4.0, "Apply {apply:?} and Close {close:?} share a row");
    assert!(apply.max.x <= close.min.x, "Apply sits left of Close: {apply:?} vs {close:?}");

    crate::actions::begin_new_tool(&mut h.state_mut().state);
    h.run_steps(3);
    let add = h.get_by_label("Add tool").rect();
    let close = h.get_by_label("Close").rect();
    assert!(leaf.expand(1.0).contains_rect(add), "Add tool {add:?} must stay visible in Properties {leaf:?} without scrolling");
    assert!(leaf.expand(1.0).contains_rect(close), "Close {close:?} must stay visible in Properties {leaf:?} without scrolling");
    assert!((add.center().y - close.center().y).abs() < 4.0, "Add tool {add:?} and Close {close:?} share a row");
    assert!(add.max.x <= close.min.x, "Add tool sits left of Close: {add:?} vs {close:?}");
    assert!(h.query_by_label("Tool number (0 = next free)").is_none(), "no raw (0 = next free) label on Tool number");
    assert!(here(&h, "Tool number"), "Catalog shows plain Tool number");

    h.state_mut().state.tool_ui.name.clear();
    h.run_steps(3);
    let warn = h.get_by_label("Name the tool first.").rect();
    let add = h.get_by_label("Add tool").rect();
    let close = h.get_by_label("Close").rect();
    assert!(leaf.expand(1.0).contains_rect(warn), "block message {warn:?} stays visible in {leaf:?}");
    assert!(leaf.expand(1.0).contains_rect(add), "Add tool {add:?} stays visible with message in {leaf:?}");
    assert!(leaf.expand(1.0).contains_rect(close), "Close {close:?} stays visible with message in {leaf:?}");
}