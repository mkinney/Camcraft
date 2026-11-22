//! Pins for #2462: a file named on the command line opens through the
//! dialogs' own commands, each model kind and a project, and a file that
//! cannot be opened is refused by name with the project left empty.
//!
//! Gate map (M4.4): covers ui::launch_file, ui::cli_options (files).
//! Provenance: our own tests.

use std::path::{Path, PathBuf};

use camcraft_app::{model, project};

use crate::cli_options::{files, help_text, FILE_HELP};
use crate::launch_file::{command, queue};
use crate::state::{CommandKind, State};

fn argv(a: &[&str]) -> Vec<String> {
    a.iter().map(|s| (*s).to_owned()).collect()
}

fn testdata(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/models").join(name)
}

fn text(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/// Start a new state with `args`, as the window does, and run its first
/// command to the end.
fn started(args: &[String]) -> State {
    let mut state = State::new();
    queue(&mut state, &files(args));
    state.drain_actions();
    while state.import_in_progress() {
        state.drain_actions();
        std::thread::yield_now();
    }
    state
}

#[test]
fn the_files_are_the_arguments_that_are_no_option_and_no_value() {
    let args = argv(&["--gpu", "nvidia", "part.sc", "--headless", "--mcp-port=7000"]);
    assert_eq!(files(&args), ["part.sc"]);
    assert_eq!(files(&argv(&["--ui", "egui", "a.stl", "b.stl"])), ["a.stl", "b.stl"]);
    assert!(files(&argv(&["-", "--headless"])).is_empty(), "a bare dash is no file");
    assert!(files(&argv(&["--bogus", "a.stl"])).is_empty(), "refused arguments open nothing");
}

#[test]
fn help_shows_the_file_argument() {
    let text = help_text();
    assert!(text.contains("Usage: camcraft-ui [options] [file]"), "{text}");
    assert!(text.contains(FILE_HELP), "{text}");
}

#[test]
fn no_file_queues_nothing() {
    assert!(matches!(command(&[]), Ok(None)));
    let state = started(&argv(&["--headless"]));
    assert!(state.pending.is_empty());
    assert!(model::model_entities(&state.world).is_empty());
}

#[test]
fn each_model_kind_is_imported_through_the_import_command() {
    for (file, name) in [("cube.stl", "cube"), ("box.obj", "box"), ("box.step", "box")] {
        let path = testdata(file);
        assert!(matches!(command(&[text(&path)]), Ok(Some(CommandKind::ImportFile(p))) if p == path), "{file}");
        let state = started(&[text(&path)]);
        assert!(model::find_by_name(&state.world, name).is_some(), "{file}: {}", state.status);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn a_solidcraft_design_is_imported() {
    let path = testdata("solidcraft_for_beginners.sc");
    assert!(matches!(command(&[text(&path)]), Ok(Some(CommandKind::ImportFile(_)))));
    let state = started(&[text(&path)]);
    assert_eq!(model::model_entities(&state.world).len(), 1, "{}", state.status);
}

#[test]
fn a_project_is_opened_through_the_open_command() {
    // A saved project with one model in it.
    let author = started(&[text(&testdata("cube.stl"))]);
    let dir = std::env::temp_dir().join(format!("camcraft-2462-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let file = dir.join("job.CAMPROJ");
    project::save(&author.world, &file).expect("save");

    assert!(matches!(command(&[text(&file)]), Ok(Some(CommandKind::OpenProjectFile(p))) if p == file));
    let state = started(&[text(&file)]);
    std::fs::remove_dir_all(&dir).expect("remove temp dir");
    assert!(model::find_by_name(&state.world, "cube").is_some(), "{}", state.status);
    assert_eq!(state.project_path.as_deref(), Some(file.as_path()));
}

#[test]
fn a_file_that_cannot_be_opened_is_refused_by_name() {
    let cube = text(&testdata("cube.stl"));
    let missing = text(&testdata("no-such-part.stl"));
    let cases = [
        (vec![cube.clone(), cube], "open refused: 2 files were given on the command line; give one model or project file"),
        (vec![String::from("notes.txt")], "open refused: notes.txt is not a model or project file"),
        (vec![String::from("README")], "open refused: README is not a model or project file"),
        (vec![missing], "open refused: no-such-part.stl is not a readable file"),
        (vec![text(&testdata(""))], "is not a model or project file"),
    ];
    for (args, want) in cases {
        let err = command(&args).err().unwrap_or_default();
        assert!(err.contains(want), "{args:?}: {err}");
        let state = started(&args);
        assert!(state.status.contains(want), "{args:?}: {}", state.status);
        assert!(state.pending.is_empty(), "{args:?}");
        assert!(model::model_entities(&state.world).is_empty(), "{args:?}");
        assert!(!state.history.is_dirty(), "{args:?}: nothing reached the history");
    }
}
