//! Known-fail heavy tests pinned to an open, blocked issue: the release heavy lanes skip them by name, and one
//! extra step for each runs it alone and must see it fail, so a skip that starts to pass fails the gate.
//! Provenance: our own build law (AGENTS.md "Three gate tiers"; the #1950 known-fail decision, RECOMMENDED.md);
//! libtest's public `--exact` / `--skip` / `--ignored` flags and the `gh issue view` CLI.

use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// One `#[ignore]`d test that is red on purpose: it pins the target of an open issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnownFail {
    /// Crate dir (`motion`, `app`).
    pub dir: &'static str,
    /// Heavy target label: `lib` or the integration binary's name.
    pub target: &'static str,
    /// The libtest path of the test inside that target.
    pub test: &'static str,
    /// The open issue the test pins; the gate refuses the skip once the issue reads closed.
    pub issue: u32,
    /// The test's source file (from the workspace root, `/`): its `panicked at` line must name it.
    pub file: &'static str,
    /// The assert text in the test body whose failure is the issue's (pinned by `known_fail_tests`).
    pub assert: &'static str,
    /// A stable piece of that assert's panic message: the step is ok only when the panic block holds it.
    pub reason: &'static str,
}

/// Only these tests. Each entry needs a `docs/decisions/` entry and an open issue; delete the entry when the
/// test passes (its known-fail step then reads FAIL and says so).
pub const KNOWN_FAIL: &[KnownFail] = &[
    KnownFail {
        dir: "motion",
        target: "lib",
        test: "machining2d::pocket_hsm::ring_tests::the_stock_ring_level_reaches_the_air_planner_with_the_stock",
        issue: 1950,
        file: "crates/motion/src/machining2d/pocket_hsm_ring.rs",
        assert: "assert!(travel.hops <= HOP_BUDGET, \"hops {} over {HOP_BUDGET}\"",
        reason: "hops ",
    },
    KnownFail {
        dir: "motion",
        target: "lib",
        test: "machining2d::pocket_hsm::ring_tests::the_stock_ring_level_plans_adaptive_with_no_budget_note",
        issue: 1950,
        file: "crates/motion/src/machining2d/pocket_hsm_ring.rs",
        assert: "assert_eq!(s, PocketStrategy::Adaptive, ",
        // The assert_eq's own `right:` line; the note text before it carries moving numbers.
        reason: " right: Adaptive",
    },
    KnownFail {
        dir: "app",
        target: "pocket3d_hsm",
        test: "the_users_whole_model_bore_roughs_adaptive_on_each_level",
        issue: 1950,
        file: "crates/app/tests/pocket3d_hsm.rs",
        assert: "assert!(fell.is_empty(), \"every level keeps the HSM plan",
        reason: "every level keeps the HSM plan",
    },
];

/// The known-fail tests of one heavy target.
pub fn of(dir: &str, target: &str) -> Vec<&'static KnownFail> {
    KNOWN_FAIL.iter().filter(|k| k.dir == dir && k.target == target).collect()
}

/// The libtest args (before `--test-threads`/`--ignored`) that leave the known-fail tests of a target out.
pub fn skip_args(dir: &str, target: &str) -> Vec<String> {
    let mut out = Vec::new();
    for k in of(dir, target) {
        if out.is_empty() {
            out.push("--exact".to_string());
        }
        out.push("--skip".to_string());
        out.push(k.test.to_string());
    }
    out
}

/// The line the release plan prints for each skipped test.
pub fn note(k: &KnownFail) -> String {
    format!(
        "SKIPPED (known fail, #{}): {} heavy: {} {} - its own step must still fail",
        k.issue, k.dir, k.target, k.test
    )
}

/// The short step name of a known-fail check.
pub fn step_name(k: &KnownFail) -> String {
    let short = k.test.rsplit("::").next().unwrap_or(k.test);
    format!("{} heavy: known fail #{} {}", k.dir, k.issue, short)
}

/// The repository whose issues pin the entries (a fork reads upstream's issue, not its own number).
pub const REPO: &str = "NeoTech/Camcraft";
/// The longest wait for one `gh issue view`.
const GH_TIMEOUT: Duration = Duration::from_secs(30);

/// Reads each pinned issue's state with `gh`. A closed issue is an error (the skip is stale); an issue that
/// `gh` cannot read (no token, as in the cut job, or no answer in 30 s) gives a note and no verdict, so the
/// known-fail steps stay the guard.
pub fn check_issues(root: &Path, dry: bool) -> Result<Vec<String>, String> {
    let mut issues: Vec<u32> = KNOWN_FAIL.iter().map(|k| k.issue).collect();
    issues.sort_unstable();
    issues.dedup();
    let mut notes = Vec::new();
    for n in issues {
        if dry {
            notes.push(format!("known fail: #{n} state not read (dry run)"));
            continue;
        }
        match gh_state(root, n) {
            Some(state) => {
                if state.eq_ignore_ascii_case("closed") {
                    return Err(format!(
                        "known fail: #{n} is closed, but xtask/src/known_fail.rs still skips its tests - remove the entries"
                    ));
                }
                notes.push(format!("known fail: #{n} is {state}"));
            }
            None => notes.push(format!(
                "known fail: could not read #{n} with gh - issue state not checked; the known-fail steps still must fail"
            )),
        }
    }
    Ok(notes)
}

/// `gh issue view <n> -R REPO` with a timeout; `None` when it fails, times out or cannot start.
fn gh_state(root: &Path, n: u32) -> Option<String> {
    let mut child = Command::new("gh")
        .args(["issue", "view", &n.to_string(), "-R", REPO, "--json", "state", "-q", ".state"])
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let t0 = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(st)) if st.success() => break,
            Ok(Some(_)) | Err(_) => return None,
            Ok(None) if t0.elapsed() > GH_TIMEOUT => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(100)),
        }
    }
    let mut out = String::new();
    child.stdout.take()?.read_to_string(&mut out).ok()?;
    Some(out.trim().to_string())
}
