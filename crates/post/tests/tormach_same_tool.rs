//! #1626: a section retaining the program's loaded tool skips G30/M6,
//! while length compensation, the machine-Z lift and the stated first
//! traverse clearance remain. The initial spindle tool is unknown.
//! Provenance: our own tests over our own post and stream format.
//!
//! Gate map (M4.4): covers assets/posts/tormach.cps, camcraft_post::post::engine.

use camcraft_post::machine::parse_mch;
use camcraft_post::post::{engine, machine};
use std::path::Path;

fn read(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .join(rel);
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

fn post(tools: &[u32]) -> String {
    let mut stream = String::new();
    for (i, tool) in tools.iter().enumerate() {
        let x = (i + 1) * 10000;
        stream.push_str(&format!(
            "PARAM OperationName section-{i}\nPARAM SpindleSpeed 1500\nTOOL {tool} DRILL\n\
             PARAM ClearancePlane 25.4\nPARAM RetractPlane 12.7\nSECTION\n\
             Rapid {x} 10000 2000\nPlunge {x} 10000 -1000 F120000\n\
             Rapid {x} 10000 2000\nENDSECTION\n"
        ));
    }
    let machine =
        machine::mirror(&parse_mch(&read("assets/machines/tormach-770mx.mch")).expect("770MX"));
    let out = engine::run(&read("assets/posts/tormach.cps"), &machine, &stream).expect("runs");
    assert!(
        out.warnings.is_empty(),
        "{:?}\n{}",
        out.warnings,
        out.program
    );
    out.program
}

fn count(program: &str, word: &str) -> usize {
    program.lines().filter(|line| *line == word).count()
}

fn assert_section_clearance(program: &str, tools: &[u32]) {
    for (i, tool) in tools.iter().enumerate() {
        let marker = format!("(OPERATION: section-{i}");
        let section = program.split(&marker).nth(1).expect("section comment");
        let section = section.split("(OPERATION:").next().expect("section body");
        let lines: Vec<_> = section.lines().collect();
        let g43 = format!("G43 H{tool}");
        let g43 = lines
            .iter()
            .position(|line| *line == g43)
            .expect("length offset every section");
        let xy = format!("G0 X{} Y10", (i + 1) * 10);
        let xy = lines
            .iter()
            .position(|line| *line == xy)
            .expect("first traverse");
        assert_eq!(lines[g43 + 1], "G53 G0 Z0", "{section}");
        assert!(g43 + 1 < xy - 1, "{section}");
        assert_eq!(lines[xy - 1], "G0 Z25.4", "{section}");
        assert!(
            !lines[g43..xy]
                .iter()
                .any(|line| line.contains('X') || line.contains('Y')),
            "{section}"
        );
    }
}

#[test]
fn repeated_tool_skips_g30_and_m6_but_keeps_compensation_lift_and_clearance() {
    let program = post(&[4, 4]);
    assert_eq!(count(&program, "G30"), 1, "{program}");
    assert_eq!(count(&program, "T4 M6"), 1, "{program}");
    assert_eq!(count(&program, "G43 H4"), 2, "{program}");
    assert_section_clearance(&program, &[4, 4]);
    // Each engine run starts with an unknown spindle: another program
    // with the same first tool still performs its initial G30 and M6.
    assert_eq!(post(&[4, 4]), program);
    let first_g30 = program.find("G30\n").expect("initial G30");
    let first_m6 = program.find("T4 M6\n").expect("initial change");
    assert!(first_g30 < first_m6, "{program}");
}

#[test]
fn a_different_tool_and_a_return_to_an_earlier_tool_each_change_again() {
    let tools = [4, 7, 7, 4];
    let program = post(&tools);
    assert_eq!(count(&program, "G30"), 3, "{program}");
    assert_eq!(count(&program, "T4 M6"), 2, "{program}");
    assert_eq!(count(&program, "T7 M6"), 1, "{program}");
    let changes: Vec<_> = program
        .lines()
        .filter(|line| line.ends_with(" M6"))
        .collect();
    assert_eq!(changes, ["T4 M6", "T7 M6", "T4 M6"], "{program}");
    assert_section_clearance(&program, &tools);
}
