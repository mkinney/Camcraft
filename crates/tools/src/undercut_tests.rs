//! Unit tests of the undercut cutter record law (#1444): each kind's
//! sound numbers pass, each broken number is refused by name, and the
//! drawn outline is the head the laws read.
//!
//! Gate map: `camcraft_tools::undercut`.

use super::*;
use crate::cutter_profile::{check, parametric};

fn tslot() -> Tool {
    cutter(ToolKind::TSlot, 20.0, 6.0, 8.0, 30.0, 0.0)
}

fn dovetail() -> Tool {
    cutter(ToolKind::Dovetail, 20.0, 5.0, 8.0, 30.0, 60.0)
}

fn lollipop() -> Tool {
    cutter(ToolKind::Lollipop, 10.0, 5.0, 4.0, 30.0, 0.0)
}

#[test]
fn each_undercut_kind_is_a_sane_tool_with_a_head_wider_than_its_neck() {
    for t in [tslot(), cutter(ToolKind::Keyseat, 16.0, 3.0, 6.0, 25.0, 0.0), dovetail(), lollipop()] {
        assert_eq!(t.why_not_sane(), None, "{:?}", t.kind);
        let u = Undercut::of(&t).unwrap();
        assert!(u.overhang() > 0.0, "{:?}", t.kind);
        assert!((u.contact().0 - t.diameter * 0.5).abs() < 1e-12, "{:?}", t.kind);
    }
    assert_eq!(Undercut::of(&crate::tools::probe_tool(2.0, 20.0)), None);
}

#[test]
fn a_neck_as_wide_as_the_head_is_refused_by_name() {
    let t = Tool { shank_d: 20.0, ..tslot() };
    let why = t.why_not_sane().unwrap();
    assert!(why.contains("T-slot cutter") && why.contains("neck") && why.contains("under its 20 mm head"), "{why}");
    assert_eq!(Undercut::of(&t), None);
}

#[test]
fn a_dovetail_whose_flank_cuts_under_its_neck_is_refused_by_name() {
    // 30 degrees over 5 mm takes 8.66 mm off the radius: 10 -> 1.34 < 4.
    let why = Tool { tip_angle: 30.0, ..dovetail() }.why_not_sane().unwrap();
    assert!(why.contains("dovetail cutter") && why.contains("under its 8 mm neck"), "{why}");
    let why = Tool { tip_angle: 90.0, ..dovetail() }.why_not_sane().unwrap();
    assert!(why.contains("flank angle 90 degrees"), "{why}");
}

#[test]
fn a_lollipop_neck_must_leave_the_ball_above_its_centre() {
    let why = Tool { flute_len: 4.0, ..lollipop() }.why_not_sane().unwrap();
    assert!(why.contains("lollipop cutter") && why.contains("ball's centre"), "{why}");
    // The neck leaves the 10 mm ball at 5 + sqrt(25 - 4) = 9.583 mm.
    assert_eq!(Tool { flute_len: 9.58, ..lollipop() }.why_not_sane(), None);
    assert!(Tool { flute_len: 9.6, ..lollipop() }.why_not_sane().is_some());
}

#[test]
fn the_head_radius_follows_the_profile_not_the_tip() {
    let d = Undercut::of(&dovetail()).unwrap();
    assert!((d.rho(0.0) - 10.0).abs() < 1e-12);
    // 60 degrees: the radius falls by 5 / tan(60) = 2.887 over the head.
    assert!((d.rho(5.0) - (10.0 - 5.0 / 60f64.to_radians().tan())).abs() < 1e-12);
    assert_eq!(d.contact(), (10.0, 0.0));
    let b = Undercut::of(&lollipop()).unwrap();
    assert_eq!(b.contact(), (5.0, 5.0));
    assert!(b.rho(0.0).abs() < 1e-12 && (b.rho(5.0) - 5.0).abs() < 1e-12);
    assert!((b.head_top() - 10.0).abs() < 1e-12, "the whole ball is the head");
    assert_eq!(b.rho(10.5), 0.0);
}

#[test]
fn the_drawn_outline_is_the_head_then_the_neck() {
    assert_eq!(tslot().profile(), vec![(0.0, 0.0), (0.0, 10.0), (6.0, 10.0), (6.0, 4.0), (36.0, 4.0), (36.0, 0.0)]);
    let top = 10.0 - 5.0 / 60f64.to_radians().tan();
    assert_eq!(dovetail().profile(), vec![(0.0, 0.0), (0.0, 10.0), (5.0, top), (5.0, 4.0), (35.0, 4.0), (35.0, 0.0)]);
    let ball = lollipop().profile();
    assert!(ball.iter().any(|&(z, r)| z == 5.0 && (r - 5.0).abs() < 1e-12), "the equator is drawn: {ball:?}");
    assert!(ball.iter().all(|&(z, r)| z.is_finite() && r.is_finite() && r <= 5.0 + 1e-12));
}

#[test]
fn each_outline_passes_the_drawn_cutter_law() {
    for t in [tslot(), dovetail(), lollipop()] {
        let s = check(&t, parametric(&t)).unwrap_or_else(|e| panic!("{:?}: {e}", t.kind));
        assert!((s.max_radius() - t.diameter * 0.5).abs() < 1e-9, "{:?}", t.kind);
    }
}

#[test]
fn the_fixture_stack_carries_the_whole_ball_and_keeps_the_neck_top() {
    // The 10 mm ball's neck leaves it at 5 mm; its pole is at 10 mm.
    let g = fixture_geom(&lollipop());
    let plain = lollipop().geom();
    assert_eq!(g.radius, 5.0);
    assert_eq!(g.flute_len, 10.0, "the head band reaches the ball's pole");
    assert_eq!(g.flute_len + g.shank_len, plain.flute_len + plain.shank_len, "the neck top and the holder stay");
    assert_eq!(g.shank_radius, plain.shank_radius);
    // A neck shorter than the cap keeps no shank band, never a negative one.
    let short = Tool { shank_len: 2.0, ..lollipop() };
    assert_eq!(fixture_geom(&short).shank_len, 0.0);
    // The disc and frustum heads end at the neck start: unchanged.
    assert_eq!(fixture_geom(&tslot()), tslot().geom());
    assert_eq!(fixture_geom(&dovetail()), dovetail().geom());
}
