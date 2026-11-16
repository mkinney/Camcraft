//! Provenance: original #1914 pins for the RichAuto DSP A11 post contract
//! specced from OEM A11 manuals, RS-274/ISO 6983, and CamCraft post laws.
//! Gate map (M4.4): covers assets/posts/richauto.cps, camcraft_post::post::engine,
//! camcraft_post::post::end_units, camcraft_post::post::options.

use camcraft_post::post::options::{declared_options, resolve_options, OptionKind, OptionValue};
use camcraft_post::post::{end_units, engine, PostError};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

#[macro_use]
mod recorded;

post_suite!("richauto.cps");

fn read(path: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path),
    )
    .unwrap_or_else(|e| panic!("{path}: {e}"))
}

const HEADER: &str = "PARAM OperationName A11\nPARAM SpindleSpeed 8000\nPARAM WorkOffset 1\nPARAM ClearancePlane 10\nPARAM RetractPlane 3\nPARAM StockZTop 0\nPARAM Coolant off\nTOOL 1 ROUTER\nSECTION\n";
const CUT: &str = "Rapid X 10000 Y 0 Z 5000\nPlunge X 10000 Y 0 Z 0 F 300000\nArc CCW X 0 Y 10000 Z 0 F 800000 I 0 J 0\nRapid X 0 Y 10000 Z 10000\n";

fn run(stream: &str, options: &Value) -> engine::PostOutput {
    engine::run_with(
        &read("assets/posts/richauto.cps"),
        &json!({"model": "RichAuto A11", "spindleMax": 24000, "coolants": []}),
        stream,
        options,
    )
    .expect("the post runs or refuses by name")
}

fn program(body: &str) -> String {
    format!("{HEADER}{body}ENDSECTION\n")
}

fn clean(out: &engine::PostOutput) {
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
}

fn refused(stream: &str, words: &[&str]) {
    let out = run(stream, &Value::Null);
    let warnings = out.warnings.join(" ").to_ascii_lowercase();
    assert!(
        words.iter().any(|word| warnings.contains(word)),
        "expected {words:?}: {warnings}"
    );
}

#[test]
fn metric_preamble_incremental_arc_and_clearance_footer() {
    let out = run(&program(CUT), &Value::Null);
    clean(&out);
    let p = &out.program;
    for word in [
        "G21", "G90", "G17", "G40", "G80", "G54", "S8000 M3", "I-10", "J0", "M5", "M30",
    ] {
        assert!(p.contains(word), "missing {word}: {p}");
    }
    for word in ["%", "G91.1", "G94", "G43", "G49", "G53", "M6", ";"] {
        assert!(!p.contains(word), "forbidden {word}: {p}");
    }
    assert!(p.is_ascii(), "{p}");
    assert!(!p.lines().any(|line| line.starts_with('O')), "{p}");
    let first_xy = p
        .lines()
        .position(|line| line.contains('X') && !line.starts_with('('))
        .expect("first XY");
    assert!(
        p.lines()
            .take(first_xy)
            .any(|line| line.starts_with("G0 Z10")),
        "{p}"
    );
    let stop = p.rfind("M5").expect("spindle stop");
    assert!(p[..stop].rfind("G0 Z10").is_some(), "{p}");
    assert!(p.trim_end().ends_with("M30"), "{p}");
}

#[test]
fn world_origin_does_not_change_incremental_arc_centres() {
    let translated = program(CUT).replace(
        "TOOL 1",
        "PARAM OriginX 5\nPARAM OriginY 7\nPARAM OriginZ 2\nTOOL 1",
    );
    let out = run(&translated, &Value::Null);
    clean(&out);
    assert!(
        out.program.contains("I-10") && out.program.contains("J0"),
        "{}",
        out.program
    );
    assert!(out.program.contains("X-5 Y3"), "{}", out.program);
    assert!(out.program.contains("G0 Z8"), "{}", out.program);
}

#[test]
fn spindle_dwell_is_configurable_and_declared_in_schema() {
    let cps = read("assets/posts/richauto.cps");
    let schema = declared_options(&cps).expect("declaredOptions parses");
    let dwell = schema
        .iter()
        .find(|o| o.key == "SpindleDwellMs")
        .expect("SpindleDwellMs declared");
    assert_eq!(dwell.default, OptionValue::Float(0.0));
    assert!(matches!(
        dwell.kind,
        OptionKind::Number {
            min: Some(0.0),
            max: Some(2147483647.0),
        }
    ));
    let defaults = resolve_options(&schema, &BTreeMap::new()).expect("defaults resolve");

    let base = run(&program(CUT), &Value::Null);
    clean(&base);
    assert!(!base.program.contains("G04"), "{}", base.program);
    let with_defaults = run(&program(CUT), &Value::Object(defaults));
    clean(&with_defaults);
    assert_eq!(with_defaults.program, base.program);

    let out = run(&program(CUT), &json!({"SpindleDwellMs": 1250}));
    clean(&out);
    assert!(
        out.program.contains("S8000 M3\nG04 P1250\n"),
        "{}",
        out.program
    );
    let fractional = run(&program(CUT), &json!({"SpindleDwellMs": 0.1}));
    clean(&fractional);
    assert!(
        fractional.program.contains("G04 P1\n"),
        "{}",
        fractional.program
    );
}

#[test]
fn bad_spindle_dwell_is_refused_by_name() {
    for value in [json!(-1), json!(2147483648_u64), json!("bad"), json!("2")] {
        let out = run(&program(CUT), &json!({"SpindleDwellMs": value}));
        assert!(
            out.warnings
                .join(" ")
                .to_ascii_lowercase()
                .contains("dwell"),
            "{:?}",
            out.warnings
        );
    }
}

#[test]
fn end_units_declares_note_and_notes_inch_ask_without_g20() {
    let cps = read("assets/posts/richauto.cps");
    assert_eq!(end_units::words(&cps).expect("manifest"), None);
    let base = run(&program(CUT), &Value::Null);
    clean(&base);
    let mm_same = run(&program(CUT), &json!({"EndUnits": "mm"}));
    clean(&mm_same);
    assert_eq!(mm_same.program, base.program);

    let inch_end = run(&program(CUT), &json!({"EndUnits": "inch"}));
    clean(&inch_end);
    let noted: Vec<&str> = inch_end
        .program
        .lines()
        .filter(|l| l.contains("END UNITS NOT WRITTEN"))
        .collect();
    assert_eq!(noted.len(), 1, "{}", inch_end.program);
    assert!(!inch_end.program.contains("G20"), "{}", inch_end.program);
}

#[test]
fn peck_drilling_keeps_each_expanded_plunge_and_retract() {
    let out = run(
        &read("crates/post/tests/recorded/streams/drill_retract.cnc"),
        &Value::Null,
    );
    clean(&out);
    for word in ["G81", "G82", "G83", "G73", "G85", "G86", "G89"] {
        assert!(!out.program.contains(word), "{}", out.program);
    }
    for z in [-3, -7, -11, -15, -19, -20] {
        assert!(
            out.program
                .lines()
                .any(|line| line.starts_with("G1 ") && line.contains(&format!("Z{z}"))),
            "missing plunge {z}: {}",
            out.program
        );
    }
    assert!(
        out.program
            .lines()
            .any(|line| line.starts_with("G0 ") && line.contains("Z-2")),
        "{}",
        out.program
    );
}

#[test]
fn tapping_is_refused_by_name() {
    for kind in ["TAP", "FLOATINGTAP"] {
        let stream = program(&format!(
            "Rapid X 0 Y 0 Z 3000\nCYCLE {kind} X 0 Y 0 Z -10000 R 3000 ENTRY 0 Q 0 P 0 F 120000 AT 1\nFeed X 0 Y 0 Z -10000 F 120000\nFeed X 0 Y 0 Z 3000 F 120000\n"
        ));
        let out = engine::run_with(
            &read("assets/posts/richauto.cps"),
            &json!({"model": "RichAuto A11", "spindleMax": 24000, "coolants": []}),
            &stream,
            &Value::Null,
        )
        .expect("runs");
        let warnings = out.warnings.join(" ").to_ascii_lowercase();
        assert!(
            warnings.contains("tapping") || warnings.contains("g84") || warnings.contains("floating"),
            "{kind}: {warnings}"
        );
    }
}

#[test]
fn cutter_compensation_is_refused_by_name() {
    for side in ["LEFT", "RIGHT"] {
        refused(
            &program(&format!(
                "Rapid X 10000 Y 0 Z 5000\nCOMP {side} 3000\nFeed X 20000 Y 0 Z 5000 F 300000\nCOMP OFF\n"
            )),
            &["compensation", "g41", "g42"],
        );
    }
}

#[test]
fn probing_is_refused_by_name() {
    refused(
        &read("crates/post/tests/recorded/streams/probe.cnc"),
        &["probing"],
    );
}

#[test]
fn a_second_tool_is_refused_but_repeated_same_tool_is_allowed() {
    let first = program(CUT);
    let repeated = format!("{first}{}", program(CUT));
    clean(&run(&repeated, &Value::Null));
    let changed = format!("{first}{}", program(CUT).replace("TOOL 1", "TOOL 2"));
    refused(&changed, &["tool change", "single-tool", "single tool"]);
}

#[test]
fn another_work_offset_is_refused_by_name() {
    refused(
        &program(CUT).replace("WorkOffset 1", "WorkOffset 2"),
        &["offset", "g55", "g54"],
    );
}

#[test]
fn missing_or_substock_clearance_is_refused_by_name() {
    for stream in [
        program(CUT).replace("PARAM ClearancePlane 10\n", ""),
        program(CUT).replace("ClearancePlane 10", "ClearancePlane -1"),
    ] {
        refused(&stream, &["clearance"]);
    }
}

#[test]
fn inch_document_is_refused_by_name() {
    let err = engine::run_with(
        &read("assets/posts/richauto.cps"),
        &Value::Null,
        &program(CUT),
        &json!({"Units": "inch"}),
    )
    .expect_err("metric-only post refuses inch");
    assert!(
        matches!(err, PostError::Refused(_)) && err.to_string().to_ascii_lowercase().contains("inch"),
        "{err}"
    );
}

fn hole(kind: &str, dwell: u32, feed: &str, floor: bool) -> String {
    let chain = if floor {
        "Plunge X 10000 Y 20000 Z -6000 F 100000\n"
    } else {
        ""
    };
    program(&format!(
        "Rapid X 10000 Y 20000 Z 2000\nCYCLE {kind} X 10000 Y 20000 Z -6000 R 2000 ENTRY 0 Q 0 P {dwell}{feed} AT 1\n{chain}Rapid X 10000 Y 20000 Z 2000\nRapid X 10000 Y 20000 Z 10000\n"
    ))
}

#[test]
fn bore_stops_at_floor_and_restarts_after_rapid_retract() {
    for dwell in [0, 500] {
        let out = run(
            &hole("BORE", dwell, " F 100000", true),
            &json!({"SpindleDwellMs": 100}),
        );
        clean(&out);
        let p = &out.program;
        let bottom = p.find("Z-6").expect("bore floor");
        let stop = bottom + p[bottom..].find("M5").expect("stop at floor");
        let restart = stop + p[stop..].find("S8000 M3").expect("restart after retract");
        assert!(
            p[stop..restart]
                .lines()
                .any(|line| line.starts_with("G0 ") && line.contains("Z2")),
            "{p}"
        );
        assert!(p[restart..].contains("G04 P100"), "{p}");
        if dwell > 0 {
            let at_floor = bottom + p[bottom..].find("G04 P500").expect("floor dwell");
            assert!(at_floor < stop, "{p}");
        }
        assert!(!p.contains("G86") && !p.contains("G89"), "{p}");
    }
}

#[test]
fn drill_dwell_waits_for_floor_and_is_not_silently_lost() {
    let out = run(&hole("DRILL", 500, " F 100000", true), &Value::Null);
    clean(&out);
    let floor = out.program.find("Z-6").expect("drill floor");
    let dwell = out.program.find("G04 P500").expect("drill floor dwell");
    assert!(floor < dwell, "{}", out.program);
    refused(&hole("DRILL", 500, " F 100000", false), &["dwell", "floor"]);
}

#[test]
fn missing_cycle_feed_is_refused_by_name() {
    refused(&hole("DRILL", 0, "", true), &["feed"]);
}

#[test]
fn invalid_spindle_and_tilt_and_unknown_feed_start_are_refused() {
    for rpm in [0, 24001] {
        refused(
            &program(CUT).replace("SpindleSpeed 8000", &format!("SpindleSpeed {rpm}")),
            &["spindle", "rpm"],
        );
    }
    refused(
        &program(CUT).replace(
            "Rapid X 10000 Y 0 Z 5000",
            "Rapid X 10000 Y 0 Z 5000 A 1000 B 0 C 0",
        ),
        &["rotary", "tilt", "3-axis"],
    );
    refused(
        &program("Feed X 10000 Y 0 Z 0 F 300000\n"),
        &["start", "rapid", "position"],
    );
}

mod richauto_safety;
