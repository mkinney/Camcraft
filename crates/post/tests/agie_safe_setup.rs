//! Setup-only Classic Cut 2S reference records from CamCraft's offline controller-checked synthetic punch.
//! Gate map: covers camcraft_post::agie_setup.
use camcraft_post::agie_setup::{PunchSetup20, SetupError};

fn punch() -> PunchSetup20<'static> {
    PunchSetup20 {
        stem: "CCSEP20",
        stock_min_um: [-40_000, -30_000],
        stock_max_um: [40_000, 30_000],
        start_hole_um: [-30_000, 0],
    }
}

#[test]
fn setup_files_match_the_control_checked_zero_step_punch() {
    let files = punch().encode().unwrap();
    assert_eq!(
        files.sbl.as_bytes(),
        include_bytes!("golden/agie-setup/CCSEP20.SBL")
    );
    assert_eq!(
        files.sbr.as_bytes(),
        include_bytes!("golden/agie-setup/CCSEP20.SBR")
    );
    let cuts: Vec<_> = files
        .sbl
        .lines()
        .filter(|l| l.contains("JE_GenerateCuts("))
        .collect();
    assert_eq!(
        cuts,
        ["  ok=JE_GenerateCuts(WORK,je_piece,\"CCSEP20\",\"CUT1\",ASDELETE,NULL)"]
    );
    assert!(files.controller_review.contains("245, 155, 140 and 134 µm"));
}

#[test]
fn a_named_resource_and_signed_micron_coordinates_round_trip() {
    let setup = PunchSetup20 {
        stem: "DEMO2026",
        stock_min_um: [-2_010, -1_001],
        stock_max_um: [1_001, 1_000],
        start_hole_um: [-1, -1],
    };
    let files = setup.encode().unwrap();
    assert_eq!(
        files.sbr,
        "AGIE.USING_DEMO2026.ISO IMPORT DEMO2026.ISO;\r\n"
    );
    for field in [
        "ID_NOMEOBJ,\"DEMO2026\",je_dim",
        "ID_GEOMETRY,\"AGIE.USING_DEMO2026.ISO\",je_dim",
        "ID_DIML,\"3.011\",je_dim",
        "ID_DIMB,\"2.001\",je_dim",
        "ID_POSPOSY,\"-1.001\",je_dim",
        "\"stp1\",ID_POSX,\"-0.001\",je_dim",
    ] {
        assert!(files.sbl.contains(field), "{field}");
    }
}

#[test]
fn device_names_paths_and_script_text_cannot_enter_an_agie_record() {
    for stem in [
        "",
        "9PART",
        "ABCDEFGHI",
        "con",
        "CON",
        "COM1",
        "LPT9",
        "../ISO",
        "X.ISO",
        "X\"\r\n",
        "Aé",
    ] {
        let setup = PunchSetup20 { stem, ..punch() };
        assert_eq!(
            setup.encode().unwrap_err(),
            SetupError::InvalidStem,
            "{stem:?}"
        );
    }
}

#[test]
fn stock_and_full_start_hole_must_fit_the_setup_span() {
    let setup = PunchSetup20 {
        stock_min_um: [-175_000, -125_000],
        stock_max_um: [175_000, 125_000],
        start_hole_um: [174_000, -124_000],
        ..punch()
    };
    assert!(setup.encode().is_ok());
    for (min, max, hole, error) in [
        (
            [-175_001, -125_000],
            [175_000, 125_000],
            [0, 0],
            SetupError::InvalidStock,
        ),
        (
            [-175_000, -125_001],
            [175_000, 125_000],
            [0, 0],
            SetupError::InvalidStock,
        ),
        ([0, 0], [0, 2_000], [0, 1_000], SetupError::InvalidStock),
        (
            [349_000, 0],
            [351_000, 2_000],
            [350_000, 1_000],
            SetupError::InvalidStock,
        ),
        (
            [i32::MIN, 0],
            [i32::MAX, 10_000],
            [0, 5_000],
            SetupError::InvalidStock,
        ),
        (
            setup.stock_min_um,
            setup.stock_max_um,
            [174_001, 0],
            SetupError::HoleOutsideStock,
        ),
        (
            setup.stock_min_um,
            setup.stock_max_um,
            [0, -124_001],
            SetupError::HoleOutsideStock,
        ),
        (
            setup.stock_min_um,
            setup.stock_max_um,
            [i32::MAX, 0],
            SetupError::HoleOutsideStock,
        ),
    ] {
        let bad = PunchSetup20 {
            stock_min_um: min,
            stock_max_um: max,
            start_hole_um: hole,
            ..setup
        };
        assert_eq!(bad.encode().unwrap_err(), error);
    }
}
