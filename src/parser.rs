use iota::iota;
use std::collections::HashMap;

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
    prefix_parse_fn: HashMap<TokenType, PrefixParseFn>,
    infix_parse_fn: HashMap<TokenType, InfixParseFn>,
}

type PrefixParseFn = fn(&mut Parser) -> ast::Expression;
type InfixParseFn = fn(&mut Parser, ast::Expression) -> ast::Expression;

iota!(
    const LOWEST: u32 = iota + 1;
    const EQUALS: u32 = iota + 1;
    const LESSGREATER: u32 = iota + 1;
    const SUM: u32 = iota + 1;
    const PRODUCT: u32 = iota + 1;
    const PREFIX: u32 = iota + 1;
    const CALL: u32 = iota + 1;
);

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
            prefix_parse_fn: HashMap::new(),
            infix_parse_fn: HashMap::new(),
        };

        // filling out prefix and infix maps with appropriate functions
        parser
            .prefix_parse_fn
            .insert(TokenType::Ident, Parser::parse_identifier);
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
            TokenType::Return => self.parse_return_statement().map(ast::Statement::Return),
            _ => self
                .parse_expression_statement()
                .map(ast::Statement::Expression),
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

    fn parse_return_statement(&mut self) -> Option<ast::ReturnStatement> {
        let token = self.curr_token.clone();

        while !self.curr_token_is(TokenType::Semicolon) {
            self.next_token();
        }

        Some(ast::ReturnStatement {
            token: token,
            return_value: ast::Expression::Placeholder,
        })
    }

    fn parse_expression_statement(&mut self) -> Option<ast::ExpressionStatement> {
        let token = self.curr_token.clone();
        let expression = self.parse_expression(LOWEST);

        // semicolon required
        if !self.expect_peek(TokenType::Semicolon) {
            return None;
        }

        Some(ast::ExpressionStatement {
            token: token,
            expression: expression.unwrap(),
        })
    }

    fn parse_expression(&mut self, precedence: u32) -> Option<ast::Expression> {
        let prefix = self.prefix_parse_fn[&self.curr_token.token_type];

        let left_exp = prefix(self);
        Some(left_exp)
    }

    fn parse_identifier(&mut self) -> ast::Expression {
        return ast::Expression::Ident(ast::Identifier {
            token: self.curr_token.clone(),
            value: self.curr_token.literal.clone(),
        });
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

    fn register_prefix(&mut self, token_type: TokenType, function: PrefixParseFn) {
        self.prefix_parse_fn.insert(token_type, function);
    }

    fn register_infix(&mut self, token_type: TokenType, function: InfixParseFn) {
        self.infix_parse_fn.insert(token_type, function);
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

    #[test]
    fn test_return_statements() {
        let input = "
        return 5;
        return 10;
        return 838383;
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

        for tt in &program.statements {
            if !test_return_statement(tt) {
                return;
            }
        }
    }

    fn test_let_statement(s: &ast::Statement, name: &'static str) -> bool {
        let let_stmt: Option<_> = match s {
            ast::Statement::Let(stmt) => Some(stmt),
            _ => None,
        };
        if let_stmt.unwrap().name.value != name {
            println!(
                "expected letstatement.name.value to be {}, got={}",
                name,
                let_stmt.unwrap().name.value
            );
            return false;
        }

        if let_stmt.unwrap().name.token_literal() != name {
            println!(
                "expected letstatement.name.token_literal() to be {}, got={}",
                name,
                let_stmt.unwrap().name.token_literal()
            );
            return false;
        }

        true
    }

    fn test_return_statement(s: &ast::Statement) -> bool {
        let ret_statement: Option<_> = match s {
            ast::Statement::Return(stmt) => Some(stmt),
            _ => None,
        };

        if ret_statement.unwrap().token_literal() != "return" {
            println!(
                "expected retstatement.token_literal to be 'return', got={}",
                ret_statement.unwrap().token_literal()
            );
            return false;
        }
        true
    }

    #[test]
    fn test_identifier_expression() {
        let input = "foobar;";

        let lexer = Lexer::new(input);
        let mut p = Parser::new(lexer);
        let program = p.parse();
        check_parser_errors(&p);

        assert_eq!(
            program.statements.len(),
            1,
            "program does not have enough statements, got={}",
            program.statements.len()
        );

        let expression_stmt: Option<_> = match &program.statements[0] {
            ast::Statement::Expression(expr) => Some(expr),
            _ => None,
        };

        let expr: ast::Expression = expression_stmt.unwrap().expression.clone();

        let ident: Option<_> = match expr {
            ast::Expression::Ident(i) => Some(i),
            _ => None,
        };

        let ident_unwrapped = ident.unwrap();

        assert_eq!(
            ident_unwrapped.value, "foobar",
            "unexpected value, got={}",
            ident_unwrapped.value
        );
        assert_eq!(
            ident_unwrapped.token_literal(),
            "foobar",
            "unexpected token_literal, got={}",
            ident_unwrapped.token_literal()
        );
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
