use crate::token::Token;
use std::any::Any;

pub struct LetStatement {
    pub token: Token,
    pub name: Identifier,
    pub value: Expression,
}

pub struct ReturnStatement {
    pub token: Token,
    pub return_value: Expression,
}

pub struct ExpressionStatement {
    pub token: Token,
    pub expression: Expression,
}

#[derive(Clone)]
pub struct PrefixExpression {
    pub token: Token,
    pub operator: String,
    pub right: Expression,
}

#[derive(Clone)]
pub struct InfixExpression {
    pub token: Token,
    pub left: Expression,
    pub operator: String,
    pub right: Expression,
}

#[derive(Clone)]
pub struct Identifier {
    pub token: Token,
    pub value: String,
}

#[derive(Clone)]
pub struct IntegerLiteral {
    pub token: Token,
    pub value: isize,
}

pub struct Program {
    pub statements: Vec<Statement>,
}

pub trait Node: Any {
    fn token_literal(&self) -> String;
    fn string(&self) -> String;
}

impl Node for LetStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        let mut s = String::new();
        let stmt_str = self.token_literal() + " " + &self.name.value + " = ";
        s.push_str(&stmt_str[..]);

        // TODO: check value whether it's Some or None

        s.push_str("; \n");

        s
    }
}

impl Node for ReturnStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        let mut s = String::new();
        let stmt_str = self.token_literal() + " ";

        // TODO: check return value whether it's Some or None

        s.push_str(&stmt_str[..]);
        s.push_str("; \n");
        s
    }
}

impl Node for ExpressionStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        self.expression.string()
    }
}

impl Node for PrefixExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        let mut s = String::new();
        s += "(";
        s += &self.operator[..];
        s += &self.right.string();
        s += ")";
        s
    }
}

impl Node for InfixExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        let mut s = String::new();

        s += "(";
        s += &self.left.string();
        s += " ";
        s += &self.operator[..];
        s += " ";
        s += &self.right.string();
        s += ")";
        s
    }
}

impl Node for Identifier {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        self.value.clone()
    }
}

impl Node for IntegerLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        self.value.to_string()
    }
}

impl Node for Statement {
    fn token_literal(&self) -> String {
        match self {
            Statement::Let(stmt) => stmt.token_literal(),
            Statement::Return(stmt) => stmt.token_literal(),
            Statement::Expression(stmt) => stmt.token_literal(),
        }
    }

    fn string(&self) -> String {
        match self {
            Statement::Let(stmt) => stmt.string(),
            Statement::Return(stmt) => stmt.string(),
            Statement::Expression(stmt) => stmt.string(),
        }
    }
}

impl Node for Expression {
    fn token_literal(&self) -> String {
        match self {
            Expression::Ident(i) => i.token_literal(),
            Expression::Int(int) => int.token_literal(),
            Expression::Prefix(pre) => pre.token_literal(),
            Expression::Infix(inf) => inf.token_literal(),
            Expression::Placeholder => String::from("delete this"),
        }
    }

    fn string(&self) -> String {
        match self {
            Expression::Ident(i) => i.string(),
            Expression::Int(int) => int.string(),
            Expression::Prefix(pre) => pre.string(),
            Expression::Infix(inf) => inf.string(),
            Expression::Placeholder => String::from("delete this"),
        }
    }
}

impl Node for Program {
    fn token_literal(&self) -> String {
        if !self.statements.is_empty() {
            self.statements[0].token_literal()
        } else {
            String::new()
        }
    }
    fn string(&self) -> String {
        let mut s = String::new();
        for stmt in &self.statements {
            let stmt_str = stmt.string();
            s.push_str(&stmt_str);
        }
        s
    }
}

pub enum Statement {
    Let(LetStatement),
    Return(ReturnStatement),
    Expression(ExpressionStatement),
}

#[derive(Clone)]
pub enum Expression {
    Ident(Identifier),
    Int(IntegerLiteral),
    Prefix(Box<PrefixExpression>),
    Infix(Box<InfixExpression>),
    Placeholder, // delete this when starting to add expressions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string() {
        let program = Program {
            statements: vec![Statement::Let(LetStatement {
                token: Token {
                    token_type: crate::token::TokenType::Let,
                    literal: "let".to_string(),
                },
                name: Identifier {
                    token: Token {
                        token_type: crate::token::TokenType::Ident,
                        literal: "my_var".to_string(),
                    },
                    value: "my_var".to_string(),
                },
                value: Expression::Ident(Identifier {
                    token: Token {
                        token_type: crate::token::TokenType::Ident,
                        literal: "another_var".to_string(),
                    },
                    value: "another_var".to_string(),
                }),
            })],
        };

        assert_eq!(
            program.string(),
            "let my_var = another_var",
            "wrong string, got={}",
            program.string()
        );
    }
}
