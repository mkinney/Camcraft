// Provenance: issue #1832 and the repository's release workflow contract.
use std::{fs, path::Path};

/// The `site` job of release.yml: from its `  site:` line up to the next
/// job (a line indented two spaces) or the end of the file.
fn site_job() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../.github/workflows/release.yml");
    let workflow = fs::read_to_string(path).expect("read the release workflow");
    let body = workflow.split_once("\n  site:\n").expect("release.yml has a site job").1;
    body.lines()
        .take_while(|line| line.is_empty() || line.starts_with("   ") || line.starts_with("  #"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_site_job_runs_the_site_tests_with_its_pinned_zola() {
    let job = site_job();
    let install = job.find("- name: Install pinned Zola").expect("the site job installs Zola");
    let tests = job.find("- name: Site tests").expect("the site job runs the site tests (#1832)");
    assert!(install < tests, "the site tests run after Zola is installed");
    let step = &job[tests..];
    let step = step[1..].split_once("\n      - ").map_or(step, |(s, _)| s);
    assert!(
        step.lines().any(|l| l.trim() == "run: python3 -m unittest discover -s site/tests -v"),
        "the step runs every site test, the operation reference included:\n{step}"
    );
    assert!(step.lines().any(|l| l.trim() == "ZOLA: /tmp/zola"), "the tests build with the pinned Zola:\n{step}");
    assert!(!step.contains("continue-on-error"), "a red site test fails the site job:\n{step}");
}
