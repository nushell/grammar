//! Every input under `../corpus`, parsed with nu-parser and compared with the verdicts in
//! `../corpus/expected/<set>.txt`, one `<key>\t<verdict>` line per input.
//!
//! `UPDATE_EXPECTED=1 cargo test` writes the expectation files instead of comparing.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use nu_grammar_harness::{Verdict, engine, verdict};
use nu_protocol::engine::EngineState;
use rayon::prelude::*;
use serde::Deserialize;

static ENGINE: LazyLock<EngineState> = LazyLock::new(engine);

/// How many changed verdicts a failure lists before it only counts them.
const SHOWN_CHANGES: usize = 40;

/// One corpus input: its key in the expectation file and its source.
struct Input {
    key: String,
    source: Vec<u8>,
}

/// An entry of a snippet list (`corpus/snippets/*.json`, `corpus/generated/*/*.json`).
#[derive(Deserialize)]
struct Snippet {
    snippet: String,
}

fn corpus() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../corpus")
}

/// Every file under `dir` with the extension `extension`, sorted, with paths relative to `root`.
fn files_under(root: &Path, dir: &Path, extension: &str) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(current) = pending.pop() {
        for entry in fs::read_dir(&current).expect("corpus directories should be readable") {
            let path = entry.expect("corpus entries should be readable").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|ext| ext == extension) {
                let relative = path
                    .strip_prefix(root)
                    .expect("a file found under the root is relative to it");
                found.push(relative.to_path_buf());
            }
        }
    }
    found.sort();
    found
}

/// A `/`-separated key, the same on every platform.
fn key_of(relative: &Path) -> String {
    relative
        .components()
        .map(|part| part.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// Every `.nu` file under `corpus/<set>`, keyed by its path below that directory.
fn nu_files(set: &str) -> Vec<Input> {
    let root = corpus().join(set);
    files_under(&root, &root, "nu")
        .into_iter()
        .map(|relative| Input {
            source: fs::read(root.join(&relative)).expect("corpus files should be readable"),
            key: key_of(&relative),
        })
        .collect()
}

/// Every snippet of every `.json` list under `corpus/<set>`, keyed `<list path>#<index>`.
fn snippets(set: &str) -> Vec<Input> {
    let root = corpus().join(set);
    files_under(&root, &root, "json")
        .into_iter()
        .flat_map(|relative| {
            let text = fs::read(root.join(&relative)).expect("snippet lists should be readable");
            let list: Vec<Snippet> =
                serde_json::from_slice(&text).expect("a snippet list is a list of {snippet}");
            let list_key = key_of(&relative);
            list.into_iter()
                .enumerate()
                .map(move |(index, entry)| Input {
                    key: format!("{list_key}#{index}"),
                    source: entry.snippet.into_bytes(),
                })
        })
        .collect()
}

/// Parse `inputs` in parallel and compare with `corpus/expected/<set>.txt`, or write that file
/// when `UPDATE_EXPECTED` is set.
fn check(set: &str, inputs: &[Input]) {
    assert!(!inputs.is_empty(), "corpus/{set} has no inputs");
    // An indexed parallel iterator keeps the input order when collected.
    let actual: String = inputs
        .par_iter()
        .map(|input| {
            let result = verdict(&ENGINE, &input.key, &input.source);
            format!("{}\t{result}\n", input.key)
        })
        .collect();
    let path = corpus().join("expected").join(format!("{set}.txt"));
    if std::env::var_os("UPDATE_EXPECTED").is_some() {
        fs::create_dir_all(path.parent().expect("the expected file has a directory"))
            .expect("corpus/expected should be writable");
        fs::write(&path, &actual).expect("the expected file should be writable");
        return;
    }
    let expected = fs::read_to_string(&path).unwrap_or_default();
    if expected == actual {
        return;
    }
    let changes = changed_verdicts(&expected, &actual);
    let shown = changes
        .iter()
        .take(SHOWN_CHANGES)
        .cloned()
        .collect::<Vec<_>>();
    panic!(
        "{} verdicts in corpus/expected/{set}.txt differ from nu-parser's \
         (expected => actual); run `UPDATE_EXPECTED=1 cargo test` if the change is right:\n{}{}",
        changes.len(),
        shown.join("\n"),
        if changes.len() > SHOWN_CHANGES {
            "\n..."
        } else {
            ""
        },
    );
}

/// The keys whose verdict differs between two expectation texts, as `key: old => new`.
fn changed_verdicts(expected: &str, actual: &str) -> Vec<String> {
    let parse = |text: &str| -> BTreeMap<String, String> {
        text.lines()
            .filter_map(|line| line.split_once('\t'))
            .map(|(key, result)| (key.to_owned(), result.to_owned()))
            .collect()
    };
    let (old, new) = (parse(expected), parse(actual));
    let mut keys = old.keys().chain(new.keys()).collect::<Vec<_>>();
    keys.sort();
    keys.dedup();
    keys.into_iter()
        .filter_map(|key| {
            let (before, after) = (old.get(key), new.get(key));
            (before != after).then(|| format!("{key}: {} => {}", shown(before), shown(after)))
        })
        .collect()
}

/// A verdict as a change report shows it; a key missing on one side shows as `(none)`.
fn shown(result: Option<&String>) -> &str {
    result.map_or("(none)", String::as_str)
}

#[test]
fn fixtures() {
    check("fixtures", &nu_files("fixtures"));
}

/// What the grammar's syntactic rules refuse, nu-parser refuses too. (The converse does not
/// hold: nu-parser also rejects accept fixtures for reasons the text alone does not decide,
/// grammar.md section 9.)
#[test]
fn reject_fixtures_fail_in_nu_parser() {
    let accepted = nu_files("fixtures/reject")
        .par_iter()
        .filter(|input| verdict(&ENGINE, &input.key, &input.source) == Verdict::Ok)
        .map(|input| input.key.clone())
        .collect::<Vec<_>>();

    assert!(
        accepted.is_empty(),
        "nu-parser accepts these reject fixtures: {accepted:?}"
    );
}

#[test]
fn files() {
    check("files", &nu_files("files"));
}

#[test]
fn command_examples_and_book() {
    check("snippets", &snippets("snippets"));
}

#[test]
fn generated() {
    check("generated", &snippets("generated"));
}

#[test]
fn changed_verdicts_lists_each_difference() {
    let expected = "a\tok\nb\terror X\nc\tok\n";
    let actual = "a\tok\nb\tok\nd\terror Y\n";

    let changes = changed_verdicts(expected, actual);

    assert_eq!(
        changes,
        [
            "b: error X => ok",
            "c: ok => (none)",
            "d: (none) => error Y"
        ]
    );
}
