use colored::Colorize;
use compiler::ast::Node;
use compiler::codegen::Compiler;
use compiler::lexer::Lexer;
use compiler::parser::Parser;
use compiler::token::TokenType;
use inkwell::context::Context;
use std::env;
use std::fs::File;
use std::io::Read;

fn read_from_file() -> () {
    let args: Vec<_> = env::args().skip(1).collect();
    let show_tokens = args.contains(&String::from("--tokens"));
    let show_ast = args.contains(&String::from("--ast"));
    let show_ir = args.contains(&String::from("--ir"));
    let text = match args.first() {
        Some(path) => {
            let mut file = File::open(path).expect("File not found");
            let mut text = String::new();
            file.read_to_string(&mut text)
                .expect("Could not read the file");
            text
        }
        None => {
            panic!("Expected the path to the file as the first argument");
        }
    };
    let mut l = Lexer::new(&text.to_string());
    if show_tokens {
        // go through the tokens in the lexer
        let mut tok = l.next_token();
        while tok.token_type != TokenType::Eof {
            println!("{}", tok);
            tok = l.next_token();
        }
        println!("{}", tok); // printing the EOF
        return;
    }
    let mut p = Parser::new(l);
    let program = p.parse();
    if show_ast {
        if p.errors().len() > 0 {
            print_parser_errors(&p.errors());
            return;
        }

        println!("{} \n", program.string());
        return;
    }
    let context = Context::create();
    let mut compiler = Compiler::new(&context);

    let result = compiler.compile_program(program);
    match result {
        Ok(_) => {}
        Err(e) => println!("{}", e),
    };

    if show_ir {
        println!("{}", compiler.module.print_to_string().to_string());
        return;
    }

    println!("No flag meaning compilation, although it's nothing for now");
}

fn print_parser_errors(errors: &Vec<String>) {
    println!("{}", "Parsing errors".red());
    for err in errors {
        eprintln!("\t {} \n", err);
    }
}

fn main() {
    read_from_file();
}
