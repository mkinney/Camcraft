//! HELD WORK for issue #1587 (not for merge): readings of the 2D Adaptive
//! trochoidal method and of the Spiral method on wide areas. The test
//! prints numbers and asserts nothing. Run it in release:
//! `cargo test --release -p camcraft-motion --test adaptive_troch_wide -- --ignored --nocapture`.
//! Gate map: covers camcraft_motion::machining2d (adaptive_troch, pocket).
use camcraft_motion::cl::{MoveKind, Toolpath};
use camcraft_motion::entry::Entry3D;
use camcraft_motion::geo2d::{Loop, Point2, Region};
use camcraft_motion::links::Links;
use camcraft_motion::machining2d::{PocketParams, PocketStrategy, adaptive_trochoidal, pocket};

fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Region {
    let Ok(lp) = Loop::new(vec![Point2::new(x0, y0), Point2::new(x1, y0), Point2::new(x1, y1), Point2::new(x0, y1)]) else { panic!("loop") };
    let Ok(reg) = Region::new(vec![lp]) else { panic!("region") };
    reg
}

fn disc(x: f64, y: f64, r: f64) -> Region {
    let Ok(d) = Region::sweep(&[vec![Point2::new(x, y)]], r) else { panic!("disc") };
    d
}

fn ring() -> Region {
    let part = disc(-20.0, 0.0, 15.0).union(&disc(20.0, 0.0, 15.0)).unwrap().union(&rect(-20.0, -6.0, 20.0, 6.0)).unwrap();
    rect(-82.0, -62.0, 82.0, 62.0).difference(&part).unwrap()
}

fn params(d: f64) -> PocketParams {
    PocketParams {
        blend_radius: None,
        tool_diameter: d,
        stepover: 1.0,
        stock_to_leave: 0.0,
        clearance_z: 10.0,
        top_z: 0.0,
        bottom_z: -3.0,
        stepdown: 3.0,
        feed: 800.0,
        plunge_feed: 200.0,
        climb: true,
        strategy: PocketStrategy::Adaptive,
        retract_z: Some(2.0),
        entry: Entry3D::Helix,
        helix_radius: 1.0,
        optimal_load: camcraft_motion::load::DEFAULT_OPTIMAL_LOAD,
        blend_turns: false,
        links: Links::PLANAR,
    }
}

fn report(name: &str, t: &Toolpath) {
    let arcs = t.moves.iter().filter(|m| m.arc.is_some()).count();
    let plunges = t.moves.windows(2).filter(|w| w[0].kind == MoveKind::Rapid && w[1].kind != MoveKind::Rapid).count();
    let mut radii: Vec<f64> = t
        .moves
        .iter()
        .filter_map(|m| m.arc.map(|a| (m.to.x - a.center.x).hypot(m.to.y - a.center.y)))
        .collect();
    radii.sort_by(f64::total_cmp);
    let med = radii.get(radii.len() / 2).copied().unwrap_or(0.0);
    let cut: f64 = t.moves.iter().filter(|m| m.kind != MoveKind::Rapid).flat_map(|m| m.segments3(0.05)).map(|(a, b)| (b.x - a.x).hypot(b.y - a.y)).sum();
    eprintln!("{name}: {} moves, {arcs} arcs, {plunges} descents, median arc r {med:.2}, max arc r {:.2}, feed length {cut:.0}", t.moves.len(), radii.last().copied().unwrap_or(0.0));
    for n in &t.notes {
        eprintln!("   note: {n}");
    }
}

fn quality(name: &str, region: &Region, p: &PocketParams, t: &Toolpath) {
    use camcraft_motion::quality::{QualitySpec, measure, rest_area, stock_for};
    let spec = QualitySpec::new(p.tool_diameter);
    let mut dex = stock_for(region, p.top_z, p.bottom_z, p.tool_diameter, &spec).unwrap();
    let q = measure(t, &mut dex, &spec).unwrap();
    let rest = rest_area(&dex, region, p.bottom_z).unwrap();
    eprintln!("   {name}: engagement {:.1} deg at {:?}, slot {:.3}, rest {rest:.1} mm2 of {:.0}", q.max_deg, q.max_at, q.slot_len, region.area());
}

#[test]
#[ignore = "held work for #1587: readings only, release"]
fn explore() {
    let part = rect(-30.0, -15.0, 30.0, 15.0).offset(4.0).unwrap();
    let rr = part.offset(40.0).unwrap().difference(&part).unwrap();
    let cases: Vec<(&str, Region, f64)> = vec![
        ("dumbbell D10", ring(), 10.0),
        ("constant ring D10", rr, 10.0),
        ("rect 60x40 D6", rect(0.0, 0.0, 60.0, 40.0), 6.0),
        ("rect 120x80 D10", rect(0.0, 0.0, 120.0, 80.0), 10.0),
    ];
    for (name, reg, d) in cases {
        let p = params(d);
        let t0 = std::time::Instant::now();
        match adaptive_trochoidal(&reg, &p) {
            Ok(t) => {
                report(&format!("troch {name}"), &t);
                eprintln!("  took {:?}", t0.elapsed());
                quality(name, &reg, &p, &t);
            }
            Err(e) => eprintln!("troch {name} refused: {e} ({:?})", t0.elapsed()),
        }
        if let Ok(t) = pocket(&reg, &p) {
            report(&format!("spiral {name}"), &t);
            quality(name, &reg, &p, &t);
        }
    }
}
