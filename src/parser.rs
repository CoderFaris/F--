use crate::{
    ast::{self, Node},
    lexer::Lexer,
    token::{self, Token, TokenType},
};

pub struct Parser {
    lexer: Lexer,
    curr_token: Token,
    peek_token: Token,
    errors: Vec<String>,
}

impl Parser {
    fn new(lxr: Lexer) -> Parser {
        let mut parser = Parser {
            lexer: lxr,
            curr_token: Token {
                token_type: TokenType::Placeholder,
                literal: String::from("PLACEHOLDER"),
            },
            peek_token: Token {
                token_type: TokenType::Placeholder,
                literal: String::from("PLACEHOLDER"),
            },
            errors: Vec::new(),
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

    fn parse(&mut self) -> ast::Program {
        let mut program = ast::Program {
            statements: Vec::new(),
        };

        while !self.curr_token_is(TokenType::Eof) {
            let stmt = self.parse_statement();

            if stmt.is_some() {
                program
                    .statements
                    .push(stmt.expect("statement should be added"));
            }

            self.next_token();
        }
        program
    }

    fn parse_statement(&mut self) -> Option<ast::Statement> {
        match self.curr_token.token_type {
            TokenType::Let => self.parse_let_statement().map(ast::Statement::Let),
            _ => None,
        }
    }

    fn parse_let_statement(&mut self) -> Option<ast::LetStatement> {
        let token = self.curr_token.clone();

        if !self.expect_peek(TokenType::Ident) {
            return None;
        }

        let name = ast::Identifier {
            token: self.curr_token.clone(),
            value: self.curr_token.literal.clone(),
        };

        if !self.expect_peek(TokenType::Assign) {
            return None;
        }

        // skipping the expression for now until we encounter a semicolon
        while !self.curr_token_is(TokenType::Semicolon) {
            self.next_token();
        }

        Some(ast::LetStatement {
            token: token,
            name: name,
            value: ast::Expression::Placeholder,
        })
    }

    fn curr_token_is(&self, token: TokenType) -> bool {
        return self.curr_token.token_type == token;
    }

    fn peek_token_is(&self, token: TokenType) -> bool {
        return self.peek_token.token_type == token;
    }

    fn expect_peek(&mut self, token: TokenType) -> bool {
        if self.peek_token_is(token) {
            self.next_token();
            return true;
        } else {
            self.peek_error(token);
            return false;
        }
    }

    fn peek_error(&mut self, token: TokenType) {
        let error_message = format!(
            "Expected next token to be: {:#?}, got {:#?}",
            token, self.peek_token.token_type
        );

        self.errors.push(error_message);
    }

    fn errors(&self) -> Vec<String> {
        self.errors.clone()
    }
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
        let mut parser: Parser = Parser::new(lexer);

        let program = parser.parse();

        check_parser_errors(&parser);

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

        for (i, tt) in tests.iter().enumerate() {
            if !test_let_statement(&program.statements[i], tt.expected_identifier) {
                return;
            }
        }
    }

    fn test_let_statement(s: &ast::Statement, name: &'static str) -> bool {
        let let_stmt = match s {
            ast::Statement::Let(stmt) => stmt,
        };
        if let_stmt.name.value != name {
            println!(
                "expected letstatement.name.value to be {}, got={}",
                name, let_stmt.name.value
            );
            return false;
        }

        if let_stmt.name.token_literal() != name {
            println!(
                "expected letstatement.name.token_literal() to be {}, got={}",
                name,
                let_stmt.name.token_literal()
            );
            return false;
        }

        true
    }

    fn check_parser_errors(parser: &Parser) {
        let errors = parser.errors();
        if errors.is_empty() {
            return;
        }
        eprintln!("parser has {:#?} errors", errors.len());
        for e in errors {
            eprintln!("parser error: {}", e);
        }
        panic!("parser errors");
    }
}
