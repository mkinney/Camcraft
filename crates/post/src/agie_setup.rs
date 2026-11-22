//! Setup-only native Classic Cut 2S records, from Quintin Smits's MIT Agie Processor and our offline control checks.
//!
//! This encoder references an existing ISO geometry file for one closed punch.
//! The caller must verify the nominal contour, its lead from the threading hole,
//! and the compensated cuts against the stock, installed travel and fixtures.
//! The piece uses stock top Z0, 20 mm steel 0001, Berco Cut 0.25 and a 1 mm separation.
//!
//! Place the SBL script and SBR resource manifest beside the ISO, with the same
//! stem and their respective extensions. Import the ISO geometry into ISO Sys.
//! Keep the matching SBR beside the SBL when importing the script into SBL Sys.
//! Create a new workpiece from the imported script. Earlier pieces can retain
//! their erosion steps on repeat import. In the new piece, match the work's ISO
//! geometry and start hole to the setup and confirm zero steps. Follow
//! [`SetupFiles::controller_review`] before assigning TECCUT.
//!
//! ```
//! use camcraft_post::agie_setup::PunchSetup20;
//!
//! let setup = PunchSetup20 {
//!     stem: "CCSEP20",
//!     stock_min_um: [-40_000, -30_000],
//!     stock_max_um: [40_000, 30_000],
//!     start_hole_um: [-30_000, 0],
//! };
//! let files = setup.encode()?;
//! assert_eq!(files.sbr, "AGIE.USING_CCSEP20.ISO IMPORT CCSEP20.ISO;\r\n");
//! assert!(files.sbl.contains("ASDELETE"));
//! # Ok::<(), camcraft_post::agie_setup::SetupError>(())
//! ```

/// Setup coordinates for a closed punch with a predrilled 2 mm start hole.
/// XY coordinates are integer micrometres in the same work frame as the ISO.
pub struct PunchSetup20<'a> {
    /// Common DOS file stem for the ISO, SBR and SBL (for example, `CCSEP20`).
    pub stem: &'a str,
    /// Lower stock limits [X, Y].
    pub stock_min_um: [i32; 2],
    /// Upper stock limits [X, Y].
    pub stock_max_um: [i32; 2],
    /// Centre of the threading hole [X, Y].
    pub start_hole_um: [i32; 2],
}

/// The two native setup files. Save their ASCII bytes with the supplied CRLF endings.
#[derive(Debug)]
pub struct SetupFiles {
    /// Piece script for the control.
    pub sbl: String,
    /// Import manifest referring to the caller's ISO geometry.
    pub sbr: String,
    /// Operator checks for the installed effective work technology.
    pub controller_review: &'static str,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum SetupError {
    #[error(
        "Use a DOS stem of 1 to 8 uppercase letters or digits beginning with a letter. Choose a name other than a DOS device."
    )]
    InvalidStem,
    #[error(
        "Use increasing stock bounds. The stock and work origin must fit the 350 × 250 mm XY span."
    )]
    InvalidStock,
    #[error("Place the complete 2 mm start hole inside the stock.")]
    HoleOutsideStock,
}

impl PunchSetup20<'_> {
    /// Encode one native work with zero erosion steps on import.
    ///
    /// # Errors
    /// Refuses invalid file stems, stock bounds and threading-hole positions.
    pub fn encode(&self) -> Result<SetupFiles, SetupError> {
        if !valid_stem(self.stem) {
            return Err(SetupError::InvalidStem);
        }
        for (axis, stroke_um) in [350_000, 250_000].into_iter().enumerate() {
            let min = i64::from(self.stock_min_um[axis]);
            let max = i64::from(self.stock_max_um[axis]);
            let hole = i64::from(self.start_hole_um[axis]);
            if min >= max || max.max(0) - min.min(0) > stroke_um {
                return Err(SetupError::InvalidStock);
            }
            if hole - 1000 < min || hole + 1000 > max {
                return Err(SetupError::HoleOutsideStock);
            }
        }
        Ok(SetupFiles {
            sbl: piece(self),
            sbr: format!("AGIE.USING_{0}.ISO IMPORT {0}.ISO;\r\n", self.stem),
            controller_review: concat!(
                "Import the ISO into ISO Sys and keep its SBR beside the SBL during script import. ",
                "Select the newly created piece. Earlier pieces can retain their steps on repeat import. ",
                "Match this work's ISO resource, geometry and start hole to the setup. Confirm zero steps. ",
                "Verify steel 0001, Berco Cut 0.25 and 20 mm. CCB030 requests Ra 0.30, Te 1.50, ",
                "Tkm 1.00 and three disabled options. Check the effective values and both visible options ",
                "on this work. Assign the matching TECCUT row. Check offsets of 245, 155, 140 and ",
                "134 µm in order before GRAFICHECK and FORMALCHECK.",
            ),
        })
    }
}

fn valid_stem(stem: &str) -> bool {
    let bytes = stem.as_bytes();
    (1..=8).contains(&bytes.len())
        && bytes[0].is_ascii_uppercase()
        && bytes
            .iter()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        && !matches!(stem, "CON" | "AUX" | "NUL" | "PRN")
        && !(bytes.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && bytes[3].is_ascii_digit())
}

/// Format native millimetres without rounding a floating point coordinate.
fn mm(value_um: i64) -> String {
    let sign = if value_um < 0 { "-" } else { "" };
    let value = value_um.unsigned_abs();
    format!("{sign}{}.{:03}", value / 1000, value % 1000)
}

fn attribute(lines: &mut Vec<String>, object: &str, context: &str, key: &str, value: &str) {
    lines.push(format!(
        "  ok=JE_AttribObject({object},SETATTR,je_piece,{context},{key},{value},je_dim)"
    ));
}

fn checked(lines: &mut Vec<String>, command: String) {
    lines.push(command);
    lines.extend([
        "  if (ok<>0) then".into(),
        "    stop".into(),
        "  end if".into(),
    ]);
}

fn piece(setup: &PunchSetup20<'_>) -> String {
    let [xmin, ymin] = setup.stock_min_um.map(i64::from);
    let [xmax, ymax] = setup.stock_max_um.map(i64::from);
    let stem = setup.stem;
    let group = format!("\"{stem}\"");
    let work = format!("{group},\"CUT1\"");
    let work_context = format!("{work},NULL");
    let hole_context = format!("{work},\"stp1\"");
    let mut lines = vec!["'$Include \"jescript.sbh\"".into(), "sub SetOrigin".into()];
    for (field, value) in [
        ("ID_HEIGHT", "20.000".into()),
        ("ID_NOMEOBJ", stem.into()),
        ("ID_MATERIAL", "0001".into()),
        ("ID_THREAD", "Berco Cut 0.25".into()),
        ("ID_POSX", "0".into()),
        ("ID_POSY", "0".into()),
        ("ID_POSZ", "0".into()),
        ("ID_ROTATION", "0".into()),
        ("ID_DIML", mm(xmax - xmin)),
        ("ID_DIMB", mm(ymax - ymin)),
        ("ID_POSPOSX", mm(xmin)),
        ("ID_POSPOSY", mm(ymin)),
        ("ID_POSPOSZ", "-20.000".into()),
        ("ID_VALSECP", "3.0".into()),
        ("ID_VALRETP", "30.0".into()),
    ] {
        attribute(
            &mut lines,
            "PIECE",
            "NULL,NULL,NULL",
            field,
            &format!("\"{value}\""),
        );
    }
    lines.extend([
        "  ok=JE_GenerateAttrib(c_tecauto,c_tecauto_t)".into(),
        "  ok=JE_GenerateAttrib(c_tecassign,c_tecass_fi)".into(),
        "  ok=JE_GenerateAttrib(c_tecalert,c_tecalert_sea)".into(),
        "end sub".into(),
        "sub CutPunch".into(),
    ]);
    checked(
        &mut lines,
        "  ok=JE_CreateQuality(je_piece,\"CCB030\",\"0.30,1.50,1.00,F,F,F\",je_q1)".into(),
    );
    checked(
        &mut lines,
        format!("  ok=JE_Create(GROUP,je_piece,{group},NULL)"),
    );
    checked(&mut lines, format!("  ok=JE_Create(WORK,je_piece,{work})"));
    attribute(
        &mut lines,
        "WORK",
        &work_context,
        "ID_GEOMETRY",
        &format!("\"AGIE.USING_{stem}.ISO\""),
    );
    attribute(&mut lines, "WORK", &work_context, "ID_PUNCH", "c_punch");
    attribute(&mut lines, "WORK", &work_context, "ID_WORKCOND", "\"2\"");
    checked(
        &mut lines,
        format!(
            "  ok=JE_AttribObject(WORK,SETATTR,je_piece,{work_context},ID_QUALITY,je_q1,je_dim)"
        ),
    );
    for (field, value) in [
        ("ID_THREAD", "\"Berco Cut 0.25\""),
        ("ID_ENTRYMODE", "c_entper"),
        ("ID_HEIGHT", "\"20.000\""),
        ("ID_POSX", "\"0\""),
        ("ID_POSY", "\"0\""),
        ("ID_SEPCUT", "\"1.0\""),
        ("ID_REVCUT", "\"T\""),
        ("ID_COMMPOINTENTRY", "\"3.0\""),
        ("ID_COMMPOINTEXIT", "\"0\""),
        ("ID_PRIORITY", "\"10\""),
        ("ID_RADIUSMINLA", "\"T\""),
        ("ID_I_TYPE", "c_radius_rm"),
        ("ID_E_TYPE", "c_radius_rm"),
        ("ID_ENTRY", "\"2\""),
    ] {
        attribute(&mut lines, "WORK", &work_context, field, value);
    }
    lines.push(format!(
        "  ok=JE_CreateStartPoint(WORK,\"stp1\",je_piece,{work})"
    ));
    for (field, coordinate) in ["ID_POSX", "ID_POSY"].into_iter().zip(setup.start_hole_um) {
        attribute(
            &mut lines,
            "STARTPOINT",
            &hole_context,
            field,
            &format!("\"{}\"", mm(i64::from(coordinate))),
        );
    }
    attribute(&mut lines, "WORK", &work_context, "ID_STP", "\"stp1\"");
    attribute(&mut lines, "WORK", &work_context, "ID_STPMODE", "c_work");
    attribute(
        &mut lines,
        "STARTPOINT",
        &hole_context,
        "ID_STP_PL",
        "\"4\"",
    );
    attribute(
        &mut lines,
        "STARTPOINT",
        &hole_context,
        "ID_DIAMETER",
        "\"2.0\"",
    );
    // An existing quality name can bind to stale data even after a successful assignment.
    checked(
        &mut lines,
        format!("  ok=JE_GenerateCuts(WORK,je_piece,{work},ASDELETE,NULL)"),
    );
    lines.extend([
        "end sub".into(),
        "sub main".into(),
        "  ok=JE_Initiate".into(),
        "  if (ok=0) then".into(),
        "    SetOrigin".into(),
        "    CutPunch".into(),
        "    ok=JE_AttribObject(PIECE,SETATTR,je_piece,je_grp,NULL,NULL,ID_STATEGY,\"EW\",je_dim)"
            .into(),
        "    ok=JE_ClosePiece(je_piece)".into(),
        "  else".into(),
        "    stop".into(),
        "  end if".into(),
        "end sub".into(),
        String::new(),
    ]);
    lines.join("\r\n")
}
