use pest::Parser;

#[macro_use]
extern crate pest_derive;

#[derive(Parser)]
#[grammar = "../nushell.pest"]
struct NuParser;

fn main() {
    let source = "let a = 1 + 2";

    let res = NuParser::parse(Rule::program, source);

    println!("{:#?}", res);
}
