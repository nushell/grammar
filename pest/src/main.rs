mod diags;

use diags::diagnostics::{help, parse_command_line_args, print_pair};
use nu_protocol::{
    ast::{Expr, Expression, Operator},
    Span, Type,
};
use pest::{iterators::Pair, Parser};
use std::error::Error;

#[macro_use]
extern crate pest_derive;

#[derive(Parser)]
#[grammar = "../nushell.pest"]
struct NuParser;

fn main() -> Result<(), Box<dyn Error>> {
    let args = parse_command_line_args()?;

    let unparsed_input = if args.file_mode {
        std::fs::read_to_string(args.file_name)?
    } else if args.string_mode {
        args.string
    } else {
        help();
        return Err("No input file or string")?;
    };

    let pairs = NuParser::parse(args.rule, &unparsed_input)?;

    if args.expression_mode {
        for pair in pairs {
            let output = convert_to_nu_expression(pair);
            println!("{:?}", output);
        }
    } else if args.diagnostic_mode {
        for pair in pairs {
            print_pair(pair, 0);
        }
    } else {
        help();
        return Err("No output mode specified. Please supply -e or -d")?;
    }

    Ok(())
}

fn convert_to_nu_expression(pair: Pair<Rule>) -> Box<Expression> {
    let span = pair.as_span();
    let token = pair.as_rule();

    let span = Span {
        start: span.start(),
        end: span.end(),
    };

    let string = pair.as_str().to_string();

    let mut v = vec![];
    for pair in pair.into_inner() {
        let operand = convert_to_nu_expression(pair);
        v.push(operand);
    }

    match token {
        Rule::plus_expr => {
            if v.len() == 1 {
                return v.pop().unwrap();
            }
            //FIXME: remove clones
            let expr = Expr::BinaryOp(v[0].clone(), v[1].clone(), v[2].clone());

            Box::new(Expression {
                expr,
                span,
                custom_completion: None,
                ty: Type::Any,
            })
        }
        Rule::mul_expr => {
            if v.len() == 1 {
                return v.pop().unwrap();
            }

            //FIXME: remove clones
            let expr = Expr::BinaryOp(v[0].clone(), v[1].clone(), v[2].clone());

            Box::new(Expression {
                expr,
                span,
                custom_completion: None,
                ty: Type::Any,
            })
        }
        Rule::pow_expr => {
            if v.len() == 1 {
                return v.pop().unwrap();
            }

            //FIXME: remove clones
            let expr = Expr::BinaryOp(v[0].clone(), v[1].clone(), v[2].clone());

            Box::new(Expression {
                expr,
                span,
                custom_completion: None,
                ty: Type::Any,
            })
        }
        Rule::int => {
            //FIXME: remove clones
            let int_val = string.parse::<i64>().unwrap();
            let expr = Expr::Int(int_val);

            Box::new(Expression {
                expr,
                span,
                custom_completion: None,
                ty: Type::Any,
            })
        }
        Rule::float => {
            //FIXME: remove clones
            let float_val = string.parse::<f64>().unwrap();
            let expr = Expr::Float(float_val);

            Box::new(Expression {
                expr,
                span,
                custom_completion: None,
                ty: Type::Any,
            })
        }
        Rule::dec_int => {
            //FIXME: remove clones
            let int_val = string.parse::<i64>().unwrap();
            let expr = Expr::Int(int_val);

            Box::new(Expression {
                expr,
                span,
                custom_completion: None,
                ty: Type::Any,
            })
        }
        Rule::plus_op => {
            if string == "+" {
                Box::new(Expression {
                    expr: Expr::Operator(Operator::Plus),
                    span,
                    custom_completion: None,
                    ty: Type::Any,
                })
            } else if string == "-" {
                Box::new(Expression {
                    expr: Expr::Operator(Operator::Minus),
                    span,
                    custom_completion: None,
                    ty: Type::Any,
                })
            } else {
                panic!("internal compiler error: operator not of supported set")
            }
        }
        Rule::mul_op => {
            if string == "*" {
                Box::new(Expression {
                    expr: Expr::Operator(Operator::Multiply),
                    span,
                    custom_completion: None,
                    ty: Type::Any,
                })
            } else if string == "/" {
                Box::new(Expression {
                    expr: Expr::Operator(Operator::Divide),
                    span,
                    custom_completion: None,
                    ty: Type::Any,
                })
            } else if string == "//" {
                Box::new(Expression {
                    expr: Expr::Operator(Operator::FloorDivision),
                    span,
                    custom_completion: None,
                    ty: Type::Any,
                })
            } else {
                panic!("internal compiler error: operator not of supported set")
            }
        }
        Rule::double_quote_string_inner => {
            // TODO: unescape the string
            Box::new(Expression {
                expr: Expr::String(string),
                span,
                custom_completion: None,
                ty: Type::String,
            })
        }
        Rule::string => {
            if v.len() == 1 {
                return v.pop().unwrap();
            }
            panic!("internal compiler error: internal string incomplete")
        }
        Rule::double_quote_string => {
            if v.len() == 1 {
                return v.pop().unwrap();
            }
            panic!("internal compiler error: internal string incomplete")
        }
        x => {
            panic!("UNMATCHED: {:#?}", x)
        }
    }

    // Box::new(Expression {
    //     expr: Expr::Nothing,
    //     span,
    //     custom_completion: None,
    //     ty: Type::Nothing,
    // })
}
