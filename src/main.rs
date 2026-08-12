use compiler::lexer::Lexer;
use compiler::token::TokenType;
use std::env;
use std::fs::File;
use std::io::Read;

fn driver() -> () {
    let args: Vec<_> = env::args().skip(1).collect();
    let show_tokens = args.contains(&String::from("--tokens"));
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
    if show_tokens {
        let mut l = Lexer::new(&text.to_string());
        // go through the tokens in the lexer
        let mut tok = l.next_token();
        while tok.token_type != TokenType::Eof {
            println!("{}", tok);
            tok = l.next_token();
        }
        println!("{}", tok); // printing the EOF
    } else {
        println!("No flag meaning compilation, although it's nothing for now");
    }
}

fn main() {
    driver();
}
