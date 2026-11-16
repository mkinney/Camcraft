//! Provenance: original safety and formatting pins for RichAuto A11 issue #1914.
use super::*;

#[test]
fn an_unstated_feed_keeps_a_known_modal_feed_and_refuses_an_unknown_one() {
    let out = run(
        &program(
            "Rapid X 0 Y 0 Z 5000\nFeed X 10000 Y 0 Z 5000 F 300000\nFeed X 20000 Y 0 Z 5000\n",
        ),
        &Value::Null,
    );
    clean(&out);
    let first = out.program.find("G1 X10 Y0 Z5 F300").expect("known feed");
    assert!(
        out.program[first..]
            .lines()
            .any(|line| line == "G1 X20 Y0 Z5" || line == "G1 X20 Y0 Z5 F300"),
        "{}",
        out.program
    );
    refused(
        &program("Rapid X 0 Y 0 Z 5000\nFeed X 10000 Y 0 Z 5000\n"),
        &["feed"],
    );
}

#[test]
fn g89_and_ream_boring_expand_to_feed_in_and_feed_out() {
    for (kind, dwell) in [("BACKBORE", 0), ("BACKBORE", 500), ("REAM", 0)] {
        let stream = hole(kind, dwell, " F 100000", true).replace(
            "Rapid X 10000 Y 20000 Z 2000\nRapid X 10000 Y 20000 Z 10000\n",
            "Feed X 10000 Y 20000 Z 2000 F 100000\nRapid X 10000 Y 20000 Z 10000\n",
        );
        let out = run(&stream, &Value::Null);
        clean(&out);
        for word in ["G85", "G87", "G89"] {
            assert!(!out.program.contains(word), "{}", out.program);
        }
        let floor = out.program.find("G1 X10 Y20 Z-6 F100").expect("feed in");
        let retract = out.program[floor..]
            .find("G1 X10 Y20 Z2 F100")
            .expect("feed out")
            + floor;
        assert!(
            !out.program[floor..retract].contains("M5"),
            "{}",
            out.program
        );
        if dwell > 0 {
            assert!(
                out.program[floor..retract].contains("G04 P500"),
                "{}",
                out.program
            );
        }
    }
}

#[test]
fn zero_radius_arc_is_refused_by_name() {
    refused(
        &program("Rapid X 0 Y 0 Z 5000\nArc CCW X 10000 Y 0 Z 5000 F 300000 I 0 J 0\n"),
        &["radius"],
    );
}

#[test]
fn invalid_tool_and_coolant_are_refused_by_name() {
    refused(&program(CUT).replace("TOOL 1", "TOOL 0"), &["tool number"]);
    let script = read("assets/posts/richauto.cps");
    for (ask, coolants, word) in [
        ("mist", json!([]), "coolant mist is unsupported"),
        ("flood", json!([{"name": "Mist", "mCode": "M7"}]), "flood"),
        ("flood", json!([{"name": "Flood", "mCode": "M7"}]), "m8"),
    ] {
        let stream = program(CUT).replace("Coolant off", &format!("Coolant {ask}"));
        let out = engine::run(
            &script,
            &json!({"spindleMax": 24000, "coolants": coolants}),
            &stream,
        )
        .expect("post");
        assert!(
            out.warnings.join(" ").to_ascii_lowercase().contains(word),
            "{:?}",
            out.warnings
        );
    }
}

#[test]
fn a_bore_without_rapid_retract_is_refused_by_name() {
    let stream = hole("BORE", 0, " F 100000", true).replace(
        "Rapid X 10000 Y 20000 Z 2000\nRapid X 10000 Y 20000 Z 10000\n",
        "",
    );
    refused(&stream, &["rapid retract"]);
}

#[test]
fn nonpositive_cut_feed_is_refused_by_name() {
    refused(&program(CUT).replace("F 300000", "F 0"), &["feed"]);
}

#[test]
fn missing_tool_and_excessive_floor_dwell_are_refused_by_name() {
    refused(&program(CUT).replace("TOOL 1 ROUTER\n", ""), &["no tool"]);
    refused(&hole("DRILL", 2147483648, " F 100000", true), &["dwell"]);
}

#[test]
fn feeding_before_bore_restart_is_refused_by_name() {
    let stream = hole("BORE", 0, " F 100000", true).replace(
        "Rapid X 10000 Y 20000 Z 2000\nRapid X 10000 Y 20000 Z 10000\n",
        "Feed X 10000 Y 20000 Z 2000 F 100000\n",
    );
    refused(&stream, &["spindle stopped"]);
}

#[test]
fn vertical_planes_use_incremental_centres_restore_g17_and_full_circle_splits() {
    let out = run(
        &read("crates/post/tests/recorded/streams/arcs.cnc"),
        &Value::Null,
    );
    clean(&out);
    for words in ["G18", "G19", "K0 I10", "J10 K0"] {
        assert!(
            out.program.contains(words),
            "missing {words}: {}",
            out.program
        );
    }
    // Verify G17 is restored if a section ends in G18 or G19.
    let g19_end = program(
        "Rapid X 0 Y 20000 Z 0\nArc CCW X 0 Y 30000 Z -10000 F 600000 J 30000 K -10000\nRapid X 0 Y 30000 Z 10000\n",
    );
    let out_g19 = run(&g19_end, &Value::Null);
    clean(&out_g19);
    let g19_pos = out_g19.program.rfind("G19").expect("G19");
    let g17_restore = out_g19.program[g19_pos..].find("G17").expect("G17 restored");
    let m5_pos = out_g19.program[g19_pos..].find("M5").expect("M5");
    assert!(g17_restore < m5_pos, "{}", out_g19.program);

    let circle = program(
        "Rapid X 10000 Y 0 Z 5000\nPlunge X 10000 Y 0 Z 0 F 300000\nArc CCW X 10000 Y 0 Z 0 F 800000 I 0 J 0\n",
    );
    let out = run(&circle, &Value::Null);
    clean(&out);
    let arcs: Vec<_> = out
        .program
        .lines()
        .filter(|line| line.starts_with("G3 "))
        .collect();
    assert_eq!(arcs.len(), 2, "{}", out.program);
    assert!(
        arcs[0].contains("X-10 Y0") && arcs[0].contains("I-10 J0"),
        "{}",
        out.program
    );
    assert!(
        arcs[1].contains("X10 Y0") && arcs[1].contains("I10 J0"),
        "{}",
        out.program
    );
}

#[test]
fn rotary_moves_emit_one_warning_per_program() {
    let out = run(
        &read("crates/post/tests/recorded/streams/rotary.cnc"),
        &Value::Null,
    );
    assert_eq!(out.warnings.len(), 1, "{:?}", out.warnings);
    assert!(out.warnings[0].to_ascii_lowercase().contains("rotary"));
}

#[test]
fn comments_are_ascii_parentheses_and_active_flood_stops() {
    let stream = program(CUT)
        .replace("OperationName A11", "OperationName A11 (x);% café")
        .replace("Coolant off", "Coolant flood");
    let out = engine::run(
        &read("assets/posts/richauto.cps"),
        &json!({"spindleMax": 24000, "coolants": [{"name": "Flood", "mCode": "M8"}]}),
        &stream,
    )
    .expect("post");
    clean(&out);
    assert!(out.program.is_ascii(), "{}", out.program);
    let comment = out
        .program
        .lines()
        .find(|line| line.contains("OPERATION:"))
        .expect("operation comment");
    assert!(
        comment.starts_with('(') && comment.ends_with(')'),
        "{comment}"
    );
    assert_eq!(comment.matches('(').count(), 1, "{comment}");
    assert_eq!(comment.matches(')').count(), 1, "{comment}");
    assert!(
        !comment.contains(';') && !comment.contains('%'),
        "{comment}"
    );
    assert!(out.program.contains("M8\n"), "{}", out.program);
    assert!(out.program.contains("M5\nM9\nM30\n"), "{}", out.program);
}
