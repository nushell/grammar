//! nu-parser's verdict on a piece of Nushell source, parsed in the engine the `nu` binary
//! parses with.
//!
//! The corpus test (`tests/corpus.rs`) runs every input under `../corpus` through
//! [`verdict`] and compares the results with `../corpus/expected`.

use std::fmt;

use miette::Diagnostic;
use nu_protocol::{
    ParseError, Span, Type, Value,
    engine::{EngineState, StateWorkingSet},
};

/// What nu-parser makes of one input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// The input parses with no error.
    Ok,
    /// The input has at least one parse error; this describes the first, as `nu-check` reports
    /// it.
    Error {
        /// The `ParseError` variant, such as `UnknownCommand`.
        variant: String,
        /// The text of the error's first label, such as `expected bool`, when it has one.
        label: Option<String>,
    },
}

impl Verdict {
    /// Describe a parse error without its spans, which move whenever the engine's own files
    /// (the standard library) change.
    fn from_error(error: &ParseError) -> Self {
        // `Debug` starts with the variant name: `Expected("bool", Span { .. })`.
        let variant = format!("{error:?}")
            .chars()
            .take_while(char::is_ascii_alphanumeric)
            .collect();
        let label = error
            .labels()
            .and_then(|mut labels| labels.next())
            .and_then(|labeled| labeled.label().map(|text| text.replace('\n', "\\n")));
        Self::Error { variant, label }
    }
}

/// One line of an expectation file: `ok`, `error Variant` or `error Variant: label`.
impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ok => f.write_str("ok"),
            Self::Error {
                variant,
                label: None,
            } => write!(f, "error {variant}"),
            Self::Error {
                variant,
                label: Some(label),
            } => write!(f, "error {variant}: {label}"),
        }
    }
}

/// Build the engine state the `nu` binary parses with.
///
/// It has the commands of nushell's `src/command_context.rs`, added in the same order, then the
/// `$nu` and `$NU_LIB_DIRS` constants and the standard library with its prelude, so every
/// command `nu` knows is known here. No plugin is registered, as with `nu -n`.
///
/// Every experimental option is first set to its default, whatever `NU_EXPERIMENTAL_OPTIONS`
/// says, so an exported option cannot change the verdicts. Call this once, before anything is
/// parsed.
///
/// # Returns
///
/// The engine state, ready for [`verdict`].
pub fn engine() -> EngineState {
    for option in nu_experimental::ALL {
        // An option that is never set reads `NU_EXPERIMENTAL_OPTIONS`; this is the value it has
        // without that variable.
        let default = matches!(option.status(), nu_experimental::Status::OptOut);
        // SAFETY: `set` is unsafe because options are global and read without
        // synchronisation. The caller sets them once, before the engine exists and before
        // any parse reads them.
        unsafe { option.set(default) };
    }
    let engine_state = nu_cmd_lang::create_default_context();
    let engine_state = nu_cmd_plugin::add_plugin_command_context(engine_state);
    let engine_state = nu_command::add_shell_command_context(engine_state);
    let engine_state = nu_cmd_extra::add_extra_command_context(engine_state);
    let engine_state = nu_cli::add_cli_context(engine_state);
    let engine_state = nu_explore::add_explore_context(engine_state);
    let mut engine_state = nu_tui::add_tui_context(engine_state);
    // `$nu` must exist before the standard library is parsed, as in the `nu` binary.
    engine_state.generate_nu_constant();
    add_lib_dirs_constant(&mut engine_state);
    nu_std::load_standard_library(&mut engine_state)
        .expect("the standard library of the pinned nushell revision should load");
    engine_state
}

/// Define `$NU_LIB_DIRS`, which `use`, `overlay use`, `source-env` and `nu-check` read while
/// parsing. It is empty, so no module is found outside the input itself.
fn add_lib_dirs_constant(engine_state: &mut EngineState) {
    let mut working_set = StateWorkingSet::new(engine_state);
    let var_id = working_set.add_variable(
        b"$NU_LIB_DIRS".into(),
        Span::unknown(),
        Type::List(Box::new(Type::String)),
        false,
    );
    working_set.set_variable_const_val(var_id, Value::list(vec![], Span::unknown()));
    let delta = working_set.render();
    engine_state
        .merge_delta(delta)
        .expect("a delta holding one constant should merge");
}

/// Parse `source` as a script, as `nu-check` does, and report the first parse error.
///
/// # Arguments
///
/// * `engine_state` - The engine from [`engine`]; it is only read, so one engine can serve
///   many threads.
/// * `name` - The file name the parser records for the source.
/// * `source` - The Nushell source.
///
/// # Returns
///
/// [`Verdict::Ok`] when nu-parser reports no error, otherwise the first error.
///
/// # Examples
///
/// ```no_run
/// let engine_state = nu_grammar_harness::engine();
/// let verdict = nu_grammar_harness::verdict(&engine_state, "x.nu", b"ls | length");
/// assert_eq!(verdict.to_string(), "ok");
/// ```
pub fn verdict(engine_state: &EngineState, name: &str, source: &[u8]) -> Verdict {
    let mut working_set = StateWorkingSet::new(engine_state);
    nu_parser::parse(&mut working_set, Some(name), source, false);
    working_set
        .parse_errors
        .first()
        .map_or(Verdict::Ok, Verdict::from_error)
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use super::*;

    static ENGINE: LazyLock<EngineState> = LazyLock::new(engine);

    #[test]
    fn clean_source_is_ok() {
        let result = verdict(&ENGINE, "ok.nu", b"ls | where size > 1kb | length");

        assert_eq!(result, Verdict::Ok);
    }

    #[test]
    fn first_error_names_its_variant_and_label() {
        let result = verdict(&ENGINE, "err.nu", b"let x = 1; $x = 2");

        assert_eq!(
            result.to_string(),
            "error AssignmentRequiresMutableVar: needs to be a mutable variable"
        );
    }

    #[test]
    fn display_without_a_label() {
        let result = Verdict::Error {
            variant: "UnexpectedEof".into(),
            label: None,
        };

        assert_eq!(result.to_string(), "error UnexpectedEof");
    }
}
