use crate::{
    ast,
    lexer::Lexer,
    token::{PLACEHOLDER, Token},
};

pub struct Parser {
    lexer: Lexer,
    curr_token: Token,
    peek_token: Token,
}

impl Parser {
    fn new(lxr: Lexer) -> Parser {
        let mut parser = Parser {
            lexer: lxr,
            curr_token: Token {
                token_type: PLACEHOLDER,
                literal: String::from("PLACEHOLDER"),
            },
            peek_token: Token {
                token_type: PLACEHOLDER,
                literal: String::from("PLACEHOLDER"),
            },
        };
        // to set both curr and peek
        parser.next_token();
        parser.next_token();

        parser
    }

    fn next_token(&mut self) {
        self.curr_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    fn parse(&self) -> ast::Program {}
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_let_statements() {
        let input = "
        let x = 5;
        let y = 10;
        let foobar = 838383;
        ";

        let lexer = Lexer::new(input);
        let parser: Parser = Parser::new(lexer);

        let program = parser.parse();

        assert_eq!(
            program.statements.len(),
            3,
            "program.statements does not have 3 statements; statements={}",
            program.statements.len()
        );

        struct TestType {
            expected_identifier: &'static str,
        }

        let tests = [
            TestType {
                expected_identifier: "x",
            },
            TestType {
                expected_identifier: "y",
            },
            TestType {
                expected_identifier: "foobar",
            },
        ];

        for tt in &tests {}
    }

    fn test_let_statement<T: ast::Statement>(s: &T, name: &'static str) -> bool {
        if s.token_literal() != "let" {
            println!(
                "expected token_literal to be 'let', got={}",
                s.token_literal()
            );
            false;
        }

        let let_stmt = if let Some(let_statement) = s.as_any().downcast_ref::<ast::LetStatement>() {
            let_statement
        } else {
            panic!("current statement not a let statement");
        };
        if let_stmt.name.value != name {
            println!(
                "expected letstatement.name.value to be {}, got={}",
                name, let_stmt.name.value
            );
            false;
        }

        if let_stmt.name.token_literal() != name {
            println!(
                "expected letstatement.name.token_literal() to be {}, got={}",
                name,
                let_stmt.name.token_literal()
            )
        }

        true
    }
}
