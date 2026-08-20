use iota::iota;
use std::{collections::HashMap, hash::Hash, sync::LazyLock};

use crate::{
    ast::{self, BlockStatement, Node},
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
type InfixParseFn = fn(&mut Parser, ast::Expression) -> Option<ast::Expression>;

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
        parser.register_prefix(TokenType::Ident, Parser::parse_identifier);
        parser.register_prefix(TokenType::Int, Parser::parse_integer_literal);
        parser.register_prefix(TokenType::Bang, Parser::parse_prefix_expression);
        parser.register_prefix(TokenType::Minus, Parser::parse_prefix_expression);
        parser.register_prefix(TokenType::True, Parser::parse_boolean);
        parser.register_prefix(TokenType::False, Parser::parse_boolean);
        parser.register_prefix(TokenType::LParen, Parser::parse_grouped_expression);
        parser.register_prefix(TokenType::RParen, Parser::parse_grouped_expression);
        parser.register_prefix(TokenType::If, Parser::parse_if_expression);
        parser.register_infix(TokenType::Plus, Parser::parse_infix_expression);
        parser.register_infix(TokenType::Minus, Parser::parse_infix_expression);
        parser.register_infix(TokenType::Asterisk, Parser::parse_infix_expression);
        parser.register_infix(TokenType::Slash, Parser::parse_infix_expression);
        parser.register_infix(TokenType::EQ, Parser::parse_infix_expression);
        parser.register_infix(TokenType::NotEq, Parser::parse_infix_expression);
        parser.register_infix(TokenType::LT, Parser::parse_infix_expression);
        parser.register_infix(TokenType::GT, Parser::parse_infix_expression);
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
        let expression_right = self.parse_expression(PREFIX);
        Some(ast::Expression::Prefix(Box::new(ast::PrefixExpression {
            token: token,
            operator: operator,
            right: expression_right.unwrap(),
        })))
    }

    fn parse_infix_expression(&mut self, left: ast::Expression) -> Option<ast::Expression> {
        let token = self.curr_token.clone();
        let operator = self.curr_token.literal.clone();

        let precedence = self.curr_precedence();
        self.next_token();
        let right = self.parse_expression(precedence);
        Some(ast::Expression::Infix(Box::new(ast::InfixExpression {
            token: token,
            operator: operator,
            left: left,
            right: right?,
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

        let mut left_exp = prefix.unwrap()(self);

        while !self.peek_token_is(TokenType::Semicolon) && precedence < self.peek_precedence() {
            let infix = self
                .infix_parse_fn
                .get(&self.peek_token.token_type)
                .copied();
            if infix.is_none() {
                return left_exp;
            }
            self.next_token();
            left_exp = infix.unwrap()(self, left_exp?);
        }

        left_exp
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

    fn parse_boolean(&mut self) -> Option<ast::Expression> {
        Some(ast::Expression::Bool(ast::Boolean {
            token: self.curr_token.clone(),
            value: self.curr_token_is(TokenType::True),
        }))
    }

    fn parse_grouped_expression(&mut self) -> Option<ast::Expression> {
        self.next_token();

        let expression = self.parse_expression(LOWEST);

        if !self.expect_peek(TokenType::RParen) {
            return None;
        }

        return expression;
    }

    fn parse_if_expression(&mut self) -> Option<ast::Expression> {
        if !self.expect_peek(TokenType::LParen) {
            return None;
        }

        self.next_token();

        let condition = self.parse_expression(LOWEST);

        if !self.expect_peek(TokenType::RParen) {
            return None;
        }

        if !self.expect_peek(TokenType::LBrace) {
            return None;
        }

        let consequence = self.parse_block_statement();

        let mut alt: Option<BlockStatement> = None;

        if self.peek_token_is(TokenType::Else) {
            self.next_token();

            if !self.expect_peek(TokenType::LBrace) {
                return None;
            }

            alt = Some(self.parse_block_statement());
        }

        Some(ast::Expression::If(Box::new(ast::IfExpression {
            token: self.curr_token.clone(),
            condition: condition?,
            consequence: consequence,
            alternative: if alt.is_some() { alt } else { None },
        })))
    }

    fn parse_block_statement(&mut self) -> ast::BlockStatement {
        let mut statements = Vec::new();

        self.next_token();
        while !self.curr_token_is(TokenType::RBrace) && !self.curr_token_is(TokenType::Eof) {
            let stmt = self.parse_statement();
            if stmt.is_some() {
                statements.push(stmt.unwrap());
            }
            self.next_token();
        }

        ast::BlockStatement {
            token: self.curr_token.clone(),
            statements: statements,
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

    enum TestLiteral {
        Int(isize),
        Bool(bool),
        Ident(String),
    }

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
    fn test_boolean_expression() {
        let input = "true;";
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

        let expression_stmt = match &program.statements[0] {
            ast::Statement::Expression(expr) => Some(expr),
            _ => None,
        };

        let expr: ast::Expression = expression_stmt.unwrap().expression.clone();

        let bool_lit = match expr {
            ast::Expression::Bool(b) => Some(b),
            _ => None,
        };

        let boolean_literal = bool_lit.unwrap();

        assert_eq!(
            boolean_literal.token.literal, "true",
            "unexpected token_literal, got={}",
            boolean_literal.token.literal
        );

        assert_eq!(
            boolean_literal.value, true,
            "unexpected token.value, got={}",
            boolean_literal.value
        );
    }

    #[test]
    fn test_parsing_prefix_expressions() {
        struct TestType {
            input: &'static str,
            operator: &'static str,
            value: TestLiteral,
        };

        let testcases = [
            TestType {
                input: "!5;",
                operator: "!",
                value: TestLiteral::Int(5),
            },
            TestType {
                input: "-15;",
                operator: "-",
                value: TestLiteral::Int(15),
            },
            TestType {
                input: "!true;",
                operator: "!",
                value: TestLiteral::Bool(true),
            },
            TestType {
                input: "!false;",
                operator: "!",
                value: TestLiteral::Bool(false),
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

            test_prefix_expression(
                expression_stmt.unwrap().expression.clone(),
                tt.operator.to_string(),
                &tt.value,
            );
        }
    }

    #[test]
    fn test_parsing_infix_expressions() {
        struct TestType {
            input: &'static str,
            left_value: TestLiteral,
            operator: &'static str,
            right_value: TestLiteral,
        };

        let testcases = [
            TestType {
                input: "5 + 5;",
                left_value: TestLiteral::Int(5),
                operator: "+",
                right_value: TestLiteral::Int(5),
            },
            TestType {
                input: "5 - 5;",
                left_value: TestLiteral::Int(5),
                operator: "-",
                right_value: TestLiteral::Int(5),
            },
            TestType {
                input: "5 * 5;",
                left_value: TestLiteral::Int(5),
                operator: "*",
                right_value: TestLiteral::Int(5),
            },
            TestType {
                input: "5 / 5;",
                left_value: TestLiteral::Int(5),
                operator: "/",
                right_value: TestLiteral::Int(5),
            },
            TestType {
                input: "5 > 5;",
                left_value: TestLiteral::Int(5),
                operator: ">",
                right_value: TestLiteral::Int(5),
            },
            TestType {
                input: "5 < 5;",
                left_value: TestLiteral::Int(5),
                operator: "<",
                right_value: TestLiteral::Int(5),
            },
            TestType {
                input: "5 == 5;",
                left_value: TestLiteral::Int(5),
                operator: "==",
                right_value: TestLiteral::Int(5),
            },
            TestType {
                input: "5 != 5;",
                left_value: TestLiteral::Int(5),
                operator: "!=",
                right_value: TestLiteral::Int(5),
            },
            TestType {
                input: "true == true;",
                left_value: TestLiteral::Bool(true),
                operator: "==",
                right_value: TestLiteral::Bool(true),
            },
            TestType {
                input: "true != false;",
                left_value: TestLiteral::Bool(true),
                operator: "!=",
                right_value: TestLiteral::Bool(false),
            },
            TestType {
                input: "false == false;",
                left_value: TestLiteral::Bool(false),
                operator: "==",
                right_value: TestLiteral::Bool(false),
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

            test_infix_expression(
                expression_stmt.unwrap().expression.clone(),
                &tt.left_value,
                tt.operator.to_string(),
                &tt.right_value,
            );
        }
    }

    #[test]
    fn test_operator_precedence_parsing() {
        struct TestType {
            input: String,
            expected: String,
        }

        let testcases = [
            TestType {
                input: "-a * b;".to_string(),
                expected: "((-a) * b)".to_string(),
            },
            TestType {
                input: "!-a;".to_string(),
                expected: "(!(-a))".to_string(),
            },
            TestType {
                input: "a + b + c;".to_string(),
                expected: "((a + b) + c)".to_string(),
            },
            TestType {
                input: "a + b - c;".to_string(),
                expected: "((a + b) - c)".to_string(),
            },
            TestType {
                input: "a * b * c;".to_string(),
                expected: "((a * b) * c)".to_string(),
            },
            TestType {
                input: "a * b / c;".to_string(),
                expected: "((a * b) / c)".to_string(),
            },
            TestType {
                input: "a + b / c;".to_string(),
                expected: "(a + (b / c))".to_string(),
            },
            TestType {
                input: "3 + 4; -5 * 5;".to_string(),
                expected: "(3 + 4)((-5) * 5)".to_string(),
            },
            TestType {
                input: "5 > 4 == 3 < 4;".to_string(),
                expected: "((5 > 4) == (3 < 4))".to_string(),
            },
            TestType {
                input: "5 < 4 != 3 > 4;".to_string(),
                expected: "((5 < 4) != (3 > 4))".to_string(),
            },
            TestType {
                input: "3 + 4 * 5 == 3 * 1 + 4 * 5;".to_string(),
                expected: "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))".to_string(),
            },
            TestType {
                input: "a + b * c + d / e - f;".to_string(),
                expected: "(((a + (b * c)) + (d / e)) - f)".to_string(),
            },
            TestType {
                input: "1 + (2 + 3) + 4;".to_string(),
                expected: "((1 + (2 + 3)) + 4)".to_string(),
            },
            TestType {
                input: "(5 + 5) * 2;".to_string(),
                expected: "((5 + 5) * 2)".to_string(),
            },
            TestType {
                input: "2 / (5 + 5);".to_string(),
                expected: "(2 / (5 + 5))".to_string(),
            },
            TestType {
                input: "-(5 + 5);".to_string(),
                expected: "(-(5 + 5))".to_string(),
            },
            TestType {
                input: "!(true == true);".to_string(),
                expected: "(!(true == true))".to_string(),
            },
            TestType {
                input: "true;".to_string(),
                expected: "true".to_string(),
            },
            TestType {
                input: "false;".to_string(),
                expected: "false".to_string(),
            },
            TestType {
                input: "3 > 5 == false;".to_string(),
                expected: "((3 > 5) == false)".to_string(),
            },
            TestType {
                input: "3 < 5 == true;".to_string(),
                expected: "((3 < 5) == true)".to_string(),
            },
        ];

        for tt in &testcases {
            let l = Lexer::new(&tt.input[..]);
            let mut p = Parser::new(l);
            let program = p.parse();
            check_parser_errors(&p);

            let actual = program.string();
            assert_eq!(
                actual, tt.expected,
                "expected={}, got={}",
                tt.expected, actual
            );
        }
    }
    #[test]
    fn test_if_expression() {
        let input = "if (x < y) { x; };";
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

        let expr: Option<_> = match &program.statements[0] {
            ast::Statement::Expression(expr) => Some(expr),
            _ => None,
        };

        let ifexpr: Option<_> = match &expr.unwrap().expression {
            ast::Expression::If(ifexpr) => Some(ifexpr),
            _ => None,
        };

        let if_expression = ifexpr.unwrap();

        if !test_infix_expression(
            if_expression.condition.clone(),
            &TestLiteral::Ident("x".to_string()),
            "<".to_string(),
            &TestLiteral::Ident("y".to_string()),
        ) {
            return;
        };

        assert_eq!(
            if_expression.consequence.statements.len(),
            1,
            "consequence has more than 1 statement, got={}",
            if_expression.consequence.statements.len()
        );

        let con = match &if_expression.consequence.statements[0] {
            ast::Statement::Expression(expr) => Some(expr),
            _ => None,
        };

        let consequence = con.unwrap();

        if !test_identifier(consequence.expression.clone(), "x".to_string()) {
            return;
        }

        if if_expression.alternative.is_some() {
            panic!(
                "if_expression.alternative.statements was not nil, got {:#?}",
                if_expression.alternative
            );
        }
    }

    #[test]
    fn test_if_else_expression() {
        let input = "if (x < y) { x; } else { y; };";
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

        let expr: Option<_> = match &program.statements[0] {
            ast::Statement::Expression(expr) => Some(expr),
            _ => None,
        };

        let ifexpr: Option<_> = match &expr.unwrap().expression {
            ast::Expression::If(ifexpr) => Some(ifexpr),
            _ => None,
        };

        let if_expression = ifexpr.unwrap();

        if !test_infix_expression(
            if_expression.condition.clone(),
            &TestLiteral::Ident("x".to_string()),
            "<".to_string(),
            &TestLiteral::Ident("y".to_string()),
        ) {
            return;
        };

        assert_eq!(
            if_expression.consequence.statements.len(),
            1,
            "consequence has more than 1 statement, got={}",
            if_expression.consequence.statements.len()
        );

        let con = match &if_expression.consequence.statements[0] {
            ast::Statement::Expression(expr) => Some(expr),
            _ => None,
        };

        let consequence = con.unwrap();

        if !test_identifier(consequence.expression.clone(), "x".to_string()) {
            return;
        }

        if if_expression.alternative.is_some() {
            let alternative = if_expression.alternative.clone().unwrap();
            let expr = match &alternative.statements[0] {
                ast::Statement::Expression(expr) => Some(expr),
                _ => None,
            };
            let expr_stmt = expr.unwrap();

            if !test_identifier(expr_stmt.expression.clone(), "y".to_string()) {
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

    fn test_identifier(expression: ast::Expression, value: String) -> bool {
        let ident = match expression {
            ast::Expression::Ident(i) => Some(i),
            _ => None,
        };

        let ident_unwrapped = ident.unwrap();
        assert_eq!(
            ident_unwrapped.value, value,
            "ident.value is not={}, got={}",
            value, ident_unwrapped.value
        );
        assert_eq!(
            ident_unwrapped.token_literal(),
            value,
            "ident.token_literal is not={}, got={}",
            value,
            ident_unwrapped.token_literal()
        );
        return true;
    }

    fn test_boolean_literal(expression: ast::Expression, value: bool) -> bool {
        let bool = match expression {
            ast::Expression::Bool(b) => Some(b),
            _ => None,
        };
        let bool_unwrapped = bool.unwrap();

        assert_eq!(
            bool_unwrapped.value, value,
            "bool value not={}, got={}",
            value, bool_unwrapped.value
        );

        assert_eq!(
            bool_unwrapped.token_literal(),
            value.to_string(),
            "bool token literal not={}, got={}",
            value.to_string(),
            bool_unwrapped.token_literal()
        );
        return true;
    }

    fn test_literal_expression(expression: ast::Expression, expected: &TestLiteral) -> bool {
        match expected {
            TestLiteral::Int(value) => test_integer_literal(expression, *value),

            TestLiteral::Bool(value) => test_boolean_literal(expression, *value),

            TestLiteral::Ident(value) => test_identifier(expression, value.to_string()),
        }
    }

    fn test_infix_expression(
        expression: ast::Expression,
        left: &TestLiteral,
        operator: String,
        right: &TestLiteral,
    ) -> bool {
        let expr = match expression {
            ast::Expression::Infix(inf) => Some(inf),
            _ => None,
        };

        let expr_unwrapped = expr.unwrap();

        if !test_literal_expression(expr_unwrapped.left, left) {
            return false;
        }

        assert_eq!(
            expr_unwrapped.operator, operator,
            "infix_expr.operator is not={}, got={}",
            operator, expr_unwrapped.operator
        );

        if !test_literal_expression(expr_unwrapped.right, right) {
            return false;
        }

        return true;
    }

    fn test_prefix_expression(
        expression: ast::Expression,
        operator: String,
        right: &TestLiteral,
    ) -> bool {
        let expr = match expression {
            ast::Expression::Prefix(pre) => Some(pre),
            _ => None,
        };

        let expr_unwrapped = expr.unwrap();

        assert_eq!(
            expr_unwrapped.operator, operator,
            "prefix_expr.operator is not={}, got={}",
            operator, expr_unwrapped.operator
        );

        if !test_literal_expression(expr_unwrapped.right, right) {
            return false;
        }

        return true;
    }
}
