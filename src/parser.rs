use iota::iota;
use std::{collections::HashMap, hash::Hash, sync::LazyLock};

use crate::{
    ast::{self, Node},
    lexer::Lexer,
    token::{self, Token, TokenType},
};

static PRECEDENDES: LazyLock<HashMap<TokenType, isize>> = LazyLock::new(|| {
    let mut precedences: HashMap<TokenType, isize> = HashMap::new();
    precedences.insert(TokenType::EQ, EQUALS);
    precedences.insert(TokenType::NotEq, EQUALS);
    precedences.insert(TokenType::LT, LESSGREATER);
    precedences.insert(TokenType::GT, LESSGREATER);
    precedences.insert(TokenType::Plus, SUM);
    precedences.insert(TokenType::Minus, SUM);
    precedences.insert(TokenType::Asterisk, PRODUCT);
    precedences.insert(TokenType::Slash, PRODUCT);

    precedences
});

pub struct Parser {
    lexer: Lexer,
    curr_token: Token,
    peek_token: Token,
    errors: Vec<String>,
    prefix_parse_fn: HashMap<TokenType, PrefixParseFn>,
    infix_parse_fn: HashMap<TokenType, InfixParseFn>,
}

type PrefixParseFn = fn(&mut Parser) -> Option<ast::Expression>;
type InfixParseFn = fn(&mut Parser, ast::Expression) -> ast::Expression;

iota!(
    const LOWEST: isize = iota + 1;
    const EQUALS: isize = iota + 1;
    const LESSGREATER: isize = iota + 1;
    const SUM: isize = iota + 1;
    const PRODUCT: isize = iota + 1;
    const PREFIX: isize = iota + 1;
    const CALL: isize = iota + 1;
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

        parser
            .prefix_parse_fn
            .insert(TokenType::Int, Parser::parse_integer_literal);
        parser
            .prefix_parse_fn
            .insert(TokenType::Bang, Parser::parse_prefix_expression);
        parser
            .prefix_parse_fn
            .insert(TokenType::Minus, Parser::parse_prefix_expression);
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

    fn parse_prefix_expression(&mut self) -> Option<ast::Expression> {
        let token = self.curr_token.clone();
        let operator = self.curr_token.literal.clone();
        self.next_token();
        let expression_right = self.parse_expression(LOWEST);
        Some(ast::Expression::Prefix(Box::new(ast::PrefixExpression {
            token: token,
            operator: operator,
            right: expression_right.unwrap(),
        })))
    }

    fn parse_expression(&mut self, precedence: isize) -> Option<ast::Expression> {
        let prefix = self
            .prefix_parse_fn
            .get(&self.curr_token.token_type)
            .copied();

        if prefix.is_none() {
            self.no_prefix_parse_fn(self.curr_token.token_type);
            return None;
        }

        let left_exp = prefix.unwrap()(self);
        match left_exp {
            Some(expr) => Some(expr),
            None => {
                self.errors.push("error when parsing left_exp".to_string());
                None
            }
        }
    }

    fn parse_identifier(&mut self) -> Option<ast::Expression> {
        return Some(ast::Expression::Ident(ast::Identifier {
            token: self.curr_token.clone(),
            value: self.curr_token.literal.clone(),
        }));
    }

    fn parse_integer_literal(&mut self) -> Option<ast::Expression> {
        match self.curr_token.literal.parse::<isize>() {
            Ok(v) => {
                return Some(ast::Expression::Int(ast::IntegerLiteral {
                    token: self.curr_token.clone(),
                    value: v,
                }));
            }
            Err(e) => {
                self.errors.push(e.to_string());
                None
            }
        }
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

    fn peek_precedence(&self) -> isize {
        let precedence = PRECEDENDES.get(&self.peek_token.token_type);
        if precedence.is_some() {
            *precedence.unwrap()
        } else {
            LOWEST
        }
    }

    fn curr_precedence(&self) -> isize {
        let precedence = PRECEDENDES.get(&self.curr_token.token_type);
        if precedence.is_some() {
            *precedence.unwrap()
        } else {
            LOWEST
        }
    }

    fn no_prefix_parse_fn(&mut self, token: TokenType) {
        let err_msg = format!("No prefix parse function for {:#?} found", token);
        self.errors.push(err_msg);
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

    #[test]
    fn test_integer_literal_expression() {
        let input = "5;";
        let l = Lexer::new(input);
        let mut p = Parser::new(l);
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

        let int_lit: Option<_> = match expr {
            ast::Expression::Int(int) => Some(int),
            _ => None,
        };

        let integer_literal = int_lit.unwrap();

        assert_eq!(
            integer_literal.token.literal, "5",
            "unexpected token_literal, got={}",
            integer_literal.token.literal
        );
        assert_eq!(
            integer_literal.value, 5,
            "unexpected value, got={}",
            integer_literal.value,
        );
    }

    #[test]
    fn test_parsing_prefix_expressions() {
        struct TestType {
            input: &'static str,
            operator: &'static str,
            integer_val: isize,
        };

        let testcases = [
            TestType {
                input: "!5;",
                operator: "!",
                integer_val: 5,
            },
            TestType {
                input: "-15;",
                operator: "-",
                integer_val: 15,
            },
        ];

        for tt in &testcases {
            let l = Lexer::new(tt.input);
            let mut p = Parser::new(l);
            let program = p.parse();
            check_parser_errors(&p);

            assert_eq!(
                program.statements.len(),
                1,
                "program does not have enough statements, got={}",
                program.statements.len()
            );

            let expression_stmt = match &program.statements[0] {
                ast::Statement::Expression(expr) => Some(expr),
                _ => None,
            };

            let prefix_exp = match &expression_stmt.unwrap().expression {
                ast::Expression::Prefix(pre) => Some(pre),
                _ => None,
            };

            let prefix_unwrapped = prefix_exp.unwrap();

            assert_eq!(
                prefix_unwrapped.operator, tt.operator,
                "prefix_operator is not={}, got={}",
                tt.operator, prefix_unwrapped.operator
            );

            if !test_integer_literal(prefix_unwrapped.right.clone(), tt.integer_val) {
                return;
            }
        }
    }

    #[test]
    fn test_parsing_infix_expressions() {
        struct TestType {
            input: &'static str,
            left_value: isize,
            operator: &'static str,
            right_value: isize,
        };

        let testcases = [
            TestType {
                input: "5 + 5;",
                left_value: 5,
                operator: "+",
                right_value: 5,
            },
            TestType {
                input: "5 - 5;",
                left_value: 5,
                operator: "-",
                right_value: 5,
            },
            TestType {
                input: "5 * 5;",
                left_value: 5,
                operator: "*",
                right_value: 5,
            },
            TestType {
                input: "5 / 5;",
                left_value: 5,
                operator: "/",
                right_value: 5,
            },
            TestType {
                input: "5 > 5;",
                left_value: 5,
                operator: ">",
                right_value: 5,
            },
            TestType {
                input: "5 < 5;",
                left_value: 5,
                operator: "<",
                right_value: 5,
            },
            TestType {
                input: "5 == 5;",
                left_value: 5,
                operator: "==",
                right_value: 5,
            },
            TestType {
                input: "5 != 5;",
                left_value: 5,
                operator: "!=",
                right_value: 5,
            },
        ];

        for tt in &testcases {
            let l = Lexer::new(tt.input);
            let mut p = Parser::new(l);
            let program = p.parse();
            check_parser_errors(&p);

            assert_eq!(
                program.statements.len(),
                1,
                "program does not have enough statements, got={}",
                program.statements.len()
            );

            let expression_stmt = match &program.statements[0] {
                ast::Statement::Expression(expr) => Some(expr),
                _ => None,
            };

            let infix_expr = match &expression_stmt.unwrap().expression {
                ast::Expression::Infix(inf) => Some(inf),
                _ => None,
            };

            let infix_unwrapped = infix_expr.unwrap();

            if !test_integer_literal(infix_unwrapped.left.clone(), tt.left_value) {
                return;
            }

            assert_eq!(
                infix_unwrapped.operator, tt.operator,
                "prefix_operator is not={}, got={}",
                tt.operator, infix_unwrapped.operator
            );

            if !test_integer_literal(infix_unwrapped.right.clone(), tt.right_value) {
                return;
            }
        }
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

    fn test_integer_literal(integer_literal: ast::Expression, value: isize) -> bool {
        let int_literal = match integer_literal {
            ast::Expression::Int(i) => Some(i),
            _ => {
                eprintln!("integer_literal is not an integer literal");
                return false;
            }
        };
        let i_lit = int_literal.unwrap();
        assert_eq!(
            i_lit.value, value,
            "integer_literal value not={}, got={}",
            value, i_lit.value
        );
        assert_eq!(
            i_lit.token.literal,
            value.to_string(),
            "integer_literal token literal not={}, got={}",
            value.to_string(),
            i_lit.token.literal
        );
        return true;
    }
}
