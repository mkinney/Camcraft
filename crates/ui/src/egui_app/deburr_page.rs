//! The Deburr page's edge filters (#1888): the minimum edge angle, the
//! round-hole filter and the list of excluded edges. Every value edits
//! the page draft, and Apply rides the command history.
//!
//! Provenance: our own UI over the Deburr parameter model
//! (`OpParams::Deburr`); no vendor material or text.

use camcraft_app::operation::{FeatureSel, GeoSel};

use super::fields::*;

/// The Geometry tab rows under the break width.
#[allow(clippy::too_many_arguments)]
pub(super) fn filters(
    ui: &mut Ui,
    world: &World,
    u: Unit,
    picks: &[GeoSel],
    exclude_edges: &mut Vec<GeoSel>,
    min_edge_angle_deg: &mut f64,
    exclude_holes: &mut bool,
    hole_min_diameter: &mut f64,
    hole_max_diameter: &mut f64,
) {
    prop_row(ui, "Minimum edge angle", |ui| num_plain(ui, min_edge_angle_deg));
    hint(ui, "degrees between the two faces' normals: a convex edge that bends less is a seam of a smooth surface and is not broken; 0 breaks every convex edge");
    group(ui, "Holes", None, |ui| {
        check_row(ui, "Skip round holes", exclude_holes);
        if *exclude_holes {
            prop_row(ui, "Minimum diameter", |ui| num(ui, u, hole_min_diameter));
            prop_row(ui, "Maximum diameter", |ui| num(ui, u, hole_max_diameter));
            hint(ui, "the rim of a round hole whose diameter is inside this range is not broken");
        }
    });
    group(ui, "Excluded edges", None, |ui| {
        let mut remove = None;
        for (i, g) in exclude_edges.iter().enumerate() {
            ui.horizontal(|ui| {
                if ui.small_button("×").on_hover_text("break this edge again").clicked() {
                    remove = Some(i);
                }
                ui.add(egui::Label::new(camcraft_app::pick::geo_label(world, g)).truncate());
            });
        }
        if let Some(i) = remove
            && i < exclude_edges.len()
        {
            exclude_edges.remove(i);
        }
        let chains: Vec<&GeoSel> = picks.iter().filter(|g| matches!(g.feature, FeatureSel::Chain(_))).collect();
        let tip = "add the edges picked in Selection to this list";
        if ui
            .add_enabled(!chains.is_empty(), egui::Button::new("Exclude the picked edges").small())
            .on_hover_text(tip)
            .on_disabled_hover_text("pick edges in Selection first")
            .clicked()
        {
            for g in chains {
                if !exclude_edges.contains(g) {
                    exclude_edges.push(g.clone());
                }
            }
        }
        hint(ui, "picked edges are the only edges broken; with no edge picked, every exposed edge is broken except the edges listed here");
    });
}
