use crate::Rule;
use pest::iterators::Pair;
use std::error::Error;

#[derive(Debug)]
pub struct Args {
    pub file_mode: bool,
    pub file_name: String,
    pub string_mode: bool,
    pub string: String,
    pub diagnostic_mode: bool,
    pub expression_mode: bool,
    pub rule_mode: bool,
    pub rule: Rule,
}

pub fn parse_command_line_args() -> Result<Args, Box<dyn Error>> {
    // Let's parse the command line arguments
    let mut file_mode = false;
    let mut file_name = String::new();
    let mut string_mode = false;
    let mut string = String::new();
    let mut diagnostic_mode = false;
    let mut expression_mode = false;
    let mut rule_mode = false;
    let mut rule = None;
    for argument in std::env::args().skip(1) {
        // println!("argument: {}", argument);
        if file_mode {
            file_name = argument;
            file_mode = false;
        } else if string_mode {
            string = argument;
            string_mode = false;
        } else if rule_mode {
            rule = get_rule(&argument);
            rule_mode = false;
        } else if argument == "-f" || argument == "--file" {
            file_mode = true;
        } else if argument == "-s" || argument == "--string" {
            string_mode = true;
        } else if argument == "-d" || argument == "--diagnostic" {
            diagnostic_mode = true;
        } else if argument == "-e" || argument == "--expression" {
            expression_mode = true;
        } else if argument == "-r" || argument == "--rule" {
            rule_mode = true;
        } else {
            return Err(format!("Unknown argument: {}", &argument))?;
        }
    }

    file_mode = !file_name.is_empty();
    string_mode = !string.is_empty();
    rule_mode = rule.is_some();

    // A tiny bit of error checking
    if file_mode && string_mode {
        help();
        return Err("Cannot use both file and string mode at the same time.")?;
    }

    if diagnostic_mode && expression_mode {
        help();
        return Err("Cannot use both diagnostic and expression mode at the same time.")?;
    }

    let unwrapped_rule = match rule {
        Some(x) => x,
        None => Rule::program,
    };

    let args = Args {
        file_mode,
        file_name,
        string_mode,
        string,
        diagnostic_mode,
        expression_mode,
        rule_mode,
        rule: unwrapped_rule,
    };

    // println!("args: {:#?}", args);

    Ok(args)
}

pub fn help() {
    println!(
        "\nusage:
    -e - runs this program in nushell expression mode.
    -d - runs this program in diagnostic mode.
    -r <string> - rule to test, specified like 'plus_expr'.
    -f <path/to/file> - file mode: provide a file to parse.
    -s \"<string>\" - string mode: provide a string to parse.\n

    example 1:
      cargo run
      (prints this help)

    example 2:
      cargo run -- -s \"10.4 + 9.6\" -e -r plus_expr
      runs in string mode, with the string \"10.4 + 9.6\", with output in expression mode, and with the rule plus_expr.

    example 3:
      cargo run -- -s \"10.4 + 9.6\" -d -r plus_expr
      runs in string mode, with the string \"10.4 + 9.6\", with output in diagnostic mode, and with the rule plus_expr.

    example 4:
      cargo run -- -s \"10.4 + 9.6\" -d
      runs in string mode, with the string \"10.4 + 9.6\", with output in diagnostic mode, and with the rule program (default).

    example 5:
      cargo r -- -f /path/to/file/example.nu -d
      runs in file mode, with the file example.nu, with the output in diagnostic mode, and with the rule program (default).
      "

    );
}

pub fn print_pair(pair: Pair<Rule>, indent: usize) {
    let span = pair.as_span();
    let rule = pair.as_rule();
    let text = pair.as_str();
    println!(
        "{:indent$}Rule: \x1b[32m{:?}\x1b[0m, Text: \x1b[36m{}\x1b[0m, Span: {{ start: \x1b[35m{}\x1b[0m end: \x1b[35m{}\x1b[0m }}",
        "",
        rule,
        text,
        span.start(),
        span.end(),
        indent = indent
    );

    for pair in pair.into_inner() {
        print_pair(pair, indent + 2);
    }
}

fn get_rule(rule_str: &str) -> Option<Rule> {
    match rule_str {
        "and_expr" => Some(Rule::and_expr),
        "arg_list" => Some(Rule::arg_list),
        "array" => Some(Rule::array),
        "assignment" => Some(Rule::assignment),
        "assignment_operator" => Some(Rule::assignment_operator),
        "backtick_string" => Some(Rule::backtick_string),
        "backtick_string_char" => Some(Rule::backtick_string_char),
        "backtick_string_inner" => Some(Rule::backtick_string_inner),
        "bare_char" => Some(Rule::bare_char),
        "bare_follow_char" => Some(Rule::bare_follow_char),
        "bare_string" => Some(Rule::bare_string),
        "bare_value" => Some(Rule::bare_value),
        "bare_word" => Some(Rule::bare_word),
        "bin_int" => Some(Rule::bin_int),
        "binary_data" => Some(Rule::binary_data),
        "binary_data_bin" => Some(Rule::binary_data_bin),
        "binary_data_hex" => Some(Rule::binary_data_hex),
        "binary_data_oct" => Some(Rule::binary_data_oct),
        "bitand_expr" => Some(Rule::bitand_expr),
        "bitor_expr" => Some(Rule::bitor_expr),
        "bitxor_expr" => Some(Rule::bitxor_expr),
        "block" => Some(Rule::block),
        "break_command" => Some(Rule::break_command),
        "closure" => Some(Rule::closure),
        "closure_args" => Some(Rule::closure_args),
        "code_block" => Some(Rule::code_block),
        "command" => Some(Rule::command),
        "commands" => Some(Rule::commands),
        "COMMENT" => Some(Rule::COMMENT),
        "comp_expr" => Some(Rule::comp_expr),
        "comp_op" => Some(Rule::comp_op),
        "comp_op_word" => Some(Rule::comp_op_word),
        "continue_command" => Some(Rule::continue_command),
        "date_fullyear" => Some(Rule::date_fullyear),
        "date_mday" => Some(Rule::date_mday),
        "date_month" => Some(Rule::date_month),
        "date_or_datetime" => Some(Rule::date_or_datetime),
        "date_sigil" => Some(Rule::date_sigil),
        "date_time" => Some(Rule::date_time),
        "dec_int" => Some(Rule::dec_int),
        "def_command" => Some(Rule::def_command),
        "def_env_command" => Some(Rule::def_env_command),
        "double_quote_interpolated_string" => Some(Rule::double_quote_interpolated_string),
        "double_quote_string" => Some(Rule::double_quote_string),
        "double_quote_string_char" => Some(Rule::double_quote_string_char),
        "double_quote_string_inner" => Some(Rule::double_quote_string_inner),
        "duration" => Some(Rule::duration),
        "EOI" => Some(Rule::EOI),
        "expr" => Some(Rule::expr),
        "filesize" => Some(Rule::filesize),
        "flag" => Some(Rule::flag),
        "float" => Some(Rule::float),
        "for_command" => Some(Rule::for_command),
        "full_date" => Some(Rule::full_date),
        "full_time" => Some(Rule::full_time),
        "hex_int" => Some(Rule::hex_int),
        "ident" => Some(Rule::ident),
        "ident_char" => Some(Rule::ident_char),
        "if_command" => Some(Rule::if_command),
        "int" => Some(Rule::int),
        "interpolated_string" => Some(Rule::interpolated_string),
        "label" => Some(Rule::label),
        "let_command" => Some(Rule::let_command),
        "let_env_command" => Some(Rule::let_env_command),
        "local_date_time" => Some(Rule::local_date_time),
        "long_flag" => Some(Rule::long_flag),
        "mul_expr" => Some(Rule::mul_expr),
        "mul_op" => Some(Rule::mul_op),
        "mul_op_word" => Some(Rule::mul_op_word),
        "mut_command" => Some(Rule::mut_command),
        "named_arg" => Some(Rule::named_arg),
        "nl" => Some(Rule::nl),
        "oct_int" => Some(Rule::oct_int),
        "or_expr" => Some(Rule::or_expr),
        "pair" => Some(Rule::pair),
        "param" => Some(Rule::param),
        "params" => Some(Rule::params),
        "paren_expr" => Some(Rule::paren_expr),
        "partial_time" => Some(Rule::partial_time),
        "pathed_value" => Some(Rule::pathed_value),
        "pipeline" => Some(Rule::pipeline),
        "plus_expr" => Some(Rule::plus_expr),
        "plus_op" => Some(Rule::plus_op),
        "pow_expr" => Some(Rule::pow_expr),
        "program" => Some(Rule::program),
        "quotes" => Some(Rule::quotes),
        "range" => Some(Rule::range),
        "range_value" => Some(Rule::range_value),
        "record" => Some(Rule::record),
        "return_command" => Some(Rule::return_command),
        "row_and_expr" => Some(Rule::row_and_expr),
        "row_bitand_expr" => Some(Rule::row_bitand_expr),
        "row_bitor_expr" => Some(Rule::row_bitor_expr),
        "row_bitxor_expr" => Some(Rule::row_bitxor_expr),
        "row_comp_expr" => Some(Rule::row_comp_expr),
        "row_condition" => Some(Rule::row_condition),
        "row_mul_expr" => Some(Rule::row_mul_expr),
        "row_or_expr" => Some(Rule::row_or_expr),
        "row_plus_expr" => Some(Rule::row_plus_expr),
        "row_pow_expr" => Some(Rule::row_pow_expr),
        "row_shift_expr" => Some(Rule::row_shift_expr),
        "row_value" => Some(Rule::row_value),
        "shift_expr" => Some(Rule::shift_expr),
        "shift_op_word" => Some(Rule::shift_op_word),
        "short_flag" => Some(Rule::short_flag),
        "single_quote_interpolated_string" => Some(Rule::single_quote_interpolated_string),
        "single_quote_string" => Some(Rule::single_quote_string),
        "single_quote_string_char" => Some(Rule::single_quote_string_char),
        "single_quote_string_inner" => Some(Rule::single_quote_string_inner),
        "sp" => Some(Rule::sp),
        "string" => Some(Rule::string),
        "table" => Some(Rule::table),
        "time_hour" => Some(Rule::time_hour),
        "time_minute" => Some(Rule::time_minute),
        "time_offset" => Some(Rule::time_offset),
        "time_secfrac" => Some(Rule::time_secfrac),
        "time_second" => Some(Rule::time_second),
        "toplevel" => Some(Rule::toplevel),
        "traditional_call" => Some(Rule::traditional_call),
        "traditional_call_arg" => Some(Rule::traditional_call_arg),
        "unit" => Some(Rule::unit),
        "unnamed_arg" => Some(Rule::unnamed_arg),
        "user_command" => Some(Rule::user_command),
        "value" => Some(Rule::value),
        "variable" => Some(Rule::variable),
        "variable_char" => Some(Rule::variable_char),
        "variable_name" => Some(Rule::variable_name),
        "where_command" => Some(Rule::where_command),
        "while_command" => Some(Rule::while_command),
        "ws" => Some(Rule::ws),
        _ => None,
    }
}
