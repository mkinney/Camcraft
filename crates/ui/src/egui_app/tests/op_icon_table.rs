//! #661: the structural pins of the icon tables, split out of
//! structural.rs (which sits at the 400-line law): every operation kind
//! and every tool kind wears its own SVG-mastered glyph, and the tree rows,
//! status badges and actions are SVG-mastered too.
//! Provenance: our own test; no HSMWorks material involved.

use super::*;

/// #661: every operation kind wears its own glyph - an `op-*` PNG drawn
/// from an SVG master in assets/ui/svg/ - and no two kinds share one (the
/// toolbar's Create cells and the tree's op rows tell kinds apart by it).
/// The pixels themselves are pinned against their masters by xtask's
/// `committed_icon_pngs_match_their_svg_masters`.
#[test]
fn every_op_kind_has_its_own_svg_mastered_icon() {
    use std::collections::HashSet;
    let svg = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/ui/svg");
    let mut seen = HashSet::new();
    for k in camcraft_app::operation::KindInfo::ALL {
        let icon = crate::egui_app::op_icons::op_icon(k.ty);
        assert!(seen.insert(icon), "{:?} shares {icon:?} with another kind", k.ty);
        let file = icon.file();
        assert!(file.starts_with("op-"), "{:?} wears the stand-in {file}.png", k.ty);
        assert!(svg.join(format!("{file}.svg")).is_file(), "{file}.png has no SVG master");
    }
    // No two slots of the table load the same PNG either.
    let files: HashSet<&str> = Icon::ALL.iter().map(|i| i.file()).collect();
    assert_eq!(files.len(), Icon::ALL.len(), "two icon slots share a PNG");
}

/// Every tool kind the editor can set, read from the editor's own kind
/// numbering (`ToolUi::tool_kind`, exhaustive in `from_tool`) - not a hand
/// list: a kind the numbering gains is enumerated here with it.
pub(super) fn every_tool_kind() -> Vec<camcraft_app::tools::ToolKind> {
    let mut ui = crate::state::ToolUi::default();
    let mut kinds = Vec::new();
    for i in 0..64 {
        ui.kind = i;
        let k = ui.tool_kind();
        if !kinds.contains(&k) {
            kinds.push(k);
        }
    }
    kinds
}

/// #661 section 4: every tool kind wears its own `tool-*` glyph with an SVG
/// master, and no two kinds share one (the tool library rows and the
/// picker tell kinds apart by it).
#[test]
fn every_tool_kind_has_its_own_svg_mastered_icon() {
    use std::collections::HashSet;
    let svg = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/ui/svg");
    let kinds = every_tool_kind();
    assert!(kinds.len() >= 13, "the editor numbers {} tool kinds", kinds.len());
    let mut seen = HashSet::new();
    for k in kinds {
        let icon = crate::egui_app::tool_icons::tool_icon(k);
        assert!(seen.insert(icon), "{k:?} shares {icon:?} with another kind");
        let file = icon.file();
        assert!(file.starts_with("tool-"), "{k:?} wears the stand-in {file}.png");
        assert!(svg.join(format!("{file}.svg")).is_file(), "{file}.png has no SVG master");
        // Never an operation's glyph: Drill the tool is not Drill the op.
        assert!(
            camcraft_app::operation::KindInfo::ALL.iter().all(|o| crate::egui_app::op_icons::op_icon(o.ty) != icon),
            "{k:?} wears an operation's glyph"
        );
    }
}

/// The operation page's six section tabs wear their own `sec-*` glyphs,
/// one per tab the page can show, each SVG-mastered - the rule the
/// operation rows follow, applied to the tab bar.
#[test]
fn every_section_tab_has_its_own_svg_mastered_icon() {
    use std::collections::HashSet;
    let svg = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/ui/svg");
    let mut seen = HashSet::new();
    for i in 0..crate::egui_app::op_tabs::SECTIONS.len() {
        let icon = crate::egui_app::op_tabs::sec_icon(i);
        assert!(seen.insert(icon), "tab {i} shares an icon with another tab");
        let file = icon.file();
        assert!(file.starts_with("sec-"), "tab {i} wears the stand-in {file}.png");
        assert!(svg.join(format!("{file}.svg")).is_file(), "{file}.png has no SVG master");
    }
}

/// #661 sections 6-8: the tree rows, badges and CamCraft actions are our
/// own SVG-mastered glyphs, no longer the stand-ins they borrowed.
#[test]
fn tree_rows_badges_and_actions_are_svg_mastered() {
    let svg = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/ui/svg");
    let ours = [
        (Icon::Setup, "tree-"), (Icon::Model, "tree-"), (Icon::Stock, "tree-"), (Icon::WorkOffset, "tree-"),
        (Icon::Fixture, "tree-"), (Icon::Machine, "tree-"), (Icon::OpGroup, "tree-"),
        (Icon::BadgeNever, "badge-"), (Icon::BadgeOk, "badge-"), (Icon::BadgeStale, "badge-"), (Icon::BadgeWarning, "badge-"),
        (Icon::Generate, "act-"), (Icon::Post, "act-"), (Icon::Simulate, "act-"), (Icon::Toolpaths, "act-"),
        (Icon::EyeOff, "act-"), (Icon::FrameAll, "act-"), (Icon::DebugInfo, "act-"),
    ];
    for (icon, prefix) in ours {
        let file = icon.file();
        assert!(file.starts_with(prefix), "{icon:?} loads {file}.png, not a {prefix}* glyph");
        assert!(svg.join(format!("{file}.svg")).is_file(), "{file}.png has no SVG master");
    }
}

/// #661 section 7: the badge says what the row's text says - refused or
/// warned wears the warning, otherwise the cache status; no new state.
#[test]
fn status_badge_follows_the_row_status() {
    use camcraft_app::operation::OpStatus;
    use crate::egui_app::tree_badges::status_badge;
    assert_eq!(status_badge(OpStatus::Never, false, 0), Icon::BadgeNever);
    assert_eq!(status_badge(OpStatus::Ok, false, 0), Icon::BadgeOk);
    assert_eq!(status_badge(OpStatus::Stale, false, 0), Icon::BadgeStale);
    assert_eq!(status_badge(OpStatus::Ok, false, 2), Icon::BadgeWarning);
    assert_eq!(status_badge(OpStatus::Stale, false, 2), Icon::BadgeStale, "a stale path's warnings are old news");
    for s in [OpStatus::Never, OpStatus::Ok, OpStatus::Stale] {
        assert_eq!(status_badge(s, true, 0), Icon::BadgeWarning, "a BLOCKED row is refused");
    }
}

fn doc_section_for(ty: camcraft_app::operation::OpType) -> (&'static str, &'static str) {
    use camcraft_app::operation::OpType;
    match ty {
        OpType::Face => ("operations/operations-2d.md", "Face"),
        OpType::Contour => ("operations/operations-2d.md", "2D Contour"),
        OpType::Pocket => ("operations/operations-2d.md", "2D Pocket"),
        OpType::Adaptive2D => ("operations/operations-2d.md", "2D Adaptive"),
        OpType::Offset2D => ("operations/operations-2d.md", "2D Offset"),
        OpType::Slot => ("operations/operations-2d.md", "Slot"),
        OpType::TrochoidalSlot => ("operations/operations-2d.md", "Trochoidal slot"),
        OpType::Trace => ("operations/operations-2d.md", "Trace"),
        OpType::Engrave => ("operations/operations-2d.md", "Engrave"),
        OpType::Chamfer2D => ("operations/operations-2d.md", "2D Chamfer"),
        OpType::Circular => ("operations/operations-2d.md", "Circular"),
        OpType::Thread => ("operations/operations-2d.md", "Thread"),
        OpType::Drill => ("operations/operations-2d.md", "Drill"),
        OpType::Bore => ("operations/operations-2d.md", "Bore"),
        OpType::Pocket3D => ("operations/operations-3d.md", "3D Pocket"),
        OpType::Finish3D => ("operations/operations-3d.md", "3D Finish"),
        OpType::Deburr => ("operations/operations-3d.md", "Deburr"),
        OpType::Probe => ("operations/operations-probing.md", "Probe"),
        OpType::Facing => ("operations/operations-turning.md", "Facing"),
        OpType::ProfileRough => ("operations/operations-turning.md", "Profile Rough"),
        OpType::ProfileFinish => ("operations/operations-turning.md", "Profile Finish"),
        OpType::GrooveRough => ("operations/operations-turning.md", "Groove Rough"),
        OpType::GrooveFinish => ("operations/operations-turning.md", "Groove Finish"),
        OpType::Part => ("operations/operations-turning.md", "Part"),
        OpType::Threading => ("operations/operations-turning.md", "Threading"),
        OpType::Chamfering => ("operations/operations-turning.md", "Chamfer"),
        OpType::TaperTurning => ("operations/operations-turning.md", "Taper"),
        OpType::MultiPassTurning => ("operations/operations-turning.md", "Multi-Pass"),
        OpType::FormTurning => ("operations/operations-turning.md", "Form"),
        OpType::BoringTurning => ("operations/operations-turning.md", "Bore"),
        OpType::PlasmaProfile => ("operations/operations-plasma.md", "Plasma Profile"),
        OpType::WireContour => ("operations/operations-edm.md", "Wire Contour"),
        OpType::SinkerBurn => ("operations/operations-edm.md", "Sinker Burn"),
        OpType::SinkerSideBurn => ("operations/operations-edm.md", "Sinker Side Burn"),
        OpType::Contour5D => ("multi-axis/operations-multiaxis.md", "5-Axis Contour"),
        OpType::Parallel5D => ("multi-axis/operations-multiaxis.md", "5-Axis Parallel"),
        OpType::Flow5D => ("multi-axis/operations-multiaxis.md", "5-Axis Flow"),
        OpType::MultiAxisContour => ("multi-axis/operations-multiaxis.md", "Multi-Axis Contour"),
        OpType::Swarf => ("multi-axis/operations-multiaxis.md", "Swarf"),
        OpType::AdvancedSwarf => ("multi-axis/operations-multiaxis.md", "Advanced Swarf"),
        OpType::RotaryContour => ("multi-axis/operations-multiaxis.md", "Rotary Contour"),
        OpType::RotaryParallel => ("multi-axis/operations-multiaxis.md", "Rotary Parallel"),
        OpType::RotaryOffset => ("multi-axis/operations-multiaxis.md", "Rotary Offset"),
        OpType::MultiAxisRough => ("multi-axis/operations-multiaxis.md", "Multi-Axis Rough"),
    }
}

fn collect_md_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("read markdown dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            collect_md_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "md") {
            out.push(path);
        }
    }
}

fn section_body<'a>(normalized: &'a str, heading: &str) -> Option<&'a str> {
    let needle = format!("\n## {heading}\n");
    let after = if let Some(pos) = normalized.find(&needle) {
        pos + needle.len()
    } else {
        let top = format!("## {heading}\n");
        if normalized.starts_with(&top) { top.len() } else { return None; }
    };
    let rest = &normalized[after..];
    let end = rest.find("\n## ").unwrap_or(rest.len());
    Some(&rest[..end])
}

/// #1901: the user-guide site copies every UI icon from assets/ui/ into
/// site/static/icons/ (verified byte-for-byte, with no stale PNGs), keeps
/// assets/ui/LICENSES.md reachable from the site, provides the `icon`
/// shortcode, starts every operation section with its app icon, and never
/// leaves a renamed or unknown `{{ icon(name=...) }}` reference behind.
#[test]
fn docs_site_icons_match_app_table_and_operations() {
    use std::collections::{BTreeMap, HashSet};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let assets_ui = root.join("assets/ui");
    let site = root.join("site");
    let site_icons = site.join("static/icons");
    let shortcode = site.join("templates/shortcodes/icon.html");
    assert!(shortcode.is_file(), "missing Zola icon shortcode at {}", shortcode.display());
    let shortcode_src = std::fs::read_to_string(&shortcode).expect("read icon shortcode");
    assert!(
        shortcode_src.contains("get_url") && shortcode_src.contains("'icons/' ~ name ~ '.png'"),
        "icon shortcode must resolve icons/<name>.png via get_url"
    );

    let known: HashSet<&str> = Icon::ALL.iter().map(|i| i.file()).collect();
    for &stem in &known {
        let src = std::fs::read(assets_ui.join(format!("{stem}.png")))
            .unwrap_or_else(|e| panic!("missing app icon {stem}.png: {e}"));
        let dst = std::fs::read(site_icons.join(format!("{stem}.png")))
            .unwrap_or_else(|e| panic!("missing site icon {stem}.png under site/static/icons/: {e}"));
        assert_eq!(dst, src, "site/static/icons/{stem}.png drifted from assets/ui/{stem}.png");
    }
    let copied_stems: HashSet<String> = std::fs::read_dir(&site_icons)
        .expect("read site/static/icons")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "png"))
        .map(|p| p.file_stem().expect("png stem").to_string_lossy().into_owned())
        .collect();
    let known_owned: HashSet<String> = known.iter().map(|s| (*s).to_owned()).collect();
    assert_eq!(copied_stems, known_owned, "site/static/icons PNGs must match Icon::ALL cleanly");

    let upstream_notice = std::fs::read_to_string(assets_ui.join("LICENSES.md"))
        .expect("read assets/ui/LICENSES.md")
        .replace("\r\n", "\n");
    let site_notice = std::fs::read_to_string(site_icons.join("LICENSES.txt"))
        .expect("missing site/static/icons/LICENSES.txt notice")
        .replace("\r\n", "\n");
    assert_eq!(site_notice, upstream_notice, "site icon notice drifted from assets/ui/LICENSES.md");
    let base_html = std::fs::read_to_string(site.join("templates/base.html")).expect("read base.html");
    assert!(base_html.contains("icons/LICENSES.txt"), "site footer must link icons/LICENSES.txt");
    let style_css = std::fs::read_to_string(site.join("static/style.css")).expect("read style.css");
    assert!(style_css.contains(".ui-icon") && style_css.contains("invert("), "style.css must style and dark-invert .ui-icon");

    let mut page_cache: BTreeMap<&'static str, String> = BTreeMap::new();
    for k in camcraft_app::operation::KindInfo::ALL {
        let (rel, heading) = doc_section_for(k.ty);
        let text = page_cache.entry(rel).or_insert_with(|| {
            std::fs::read_to_string(site.join("content/guide").join(rel))
                .unwrap_or_else(|e| panic!("read {rel}: {e}"))
                .replace("\r\n", "\n")
        });
        let stem = crate::egui_app::op_icons::op_icon(k.ty).file();
        let rest = section_body(text, heading)
            .unwrap_or_else(|| panic!("missing ## {heading} in {rel}"));
        let first_line = rest.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
        let want = format!("{{{{ icon(name=\"{stem}\") }}}}");
        assert!(
            first_line.starts_with(&want),
            "{rel} ## {heading} must start with {want}, found: {first_line:?}"
        );
    }

    let mut md_files = Vec::new();
    collect_md_files(&site.join("content"), &mut md_files);
    let mut used_names = HashSet::new();
    for path in md_files {
        let text = std::fs::read_to_string(&path).expect("read markdown");
        let mut search = text.as_str();
        while let Some(pos) = search.find("{{ icon(") {
            let after = &search[pos + 8..];
            let end = after.find(") }}").expect("unclosed icon shortcode");
            let call = &after[..end];
            let name_start = call.find("name=\"").expect("icon shortcode missing name=\"...\"") + 6;
            let name_len = call[name_start..].find('"').expect("unterminated icon name");
            let name = &call[name_start..name_start + name_len];
            assert!(
                known.contains(name),
                "{} references unknown/renamed app icon '{name}'",
                path.display()
            );
            used_names.insert(name.to_owned());
            search = &after[end + 3..];
        }
    }
    for required in ["act-generate", "act-post", "act-simulate", "act-show-paths", "tree-stock", "tree-wcs", "tree-setup", "tools"] {
        assert!(used_names.contains(required), "user guide should show inline control icon {required}");
    }
}
