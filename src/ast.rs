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

pub struct Identifier {
    pub token: Token,
    pub value: String,
}

pub struct Program {
    pub statements: Vec<Statement>,
}

pub trait Node: Any {
    fn token_literal(&self) -> String;
}

impl Node for LetStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}

impl Node for ReturnStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}

impl Node for Identifier {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}

impl Node for Statement {
    fn token_literal(&self) -> String {
        match self {
            Statement::Let(stmt) => stmt.token_literal(),
            Statement::Return(stmt) => stmt.token_literal(),
        }
    }
}

impl Node for Expression {
    fn token_literal(&self) -> String {
        match self {
            Expression::Ident(i) => i.token_literal(),
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
}

pub enum Statement {
    Let(LetStatement),
    Return(ReturnStatement),
}
pub enum Expression {
    Ident(Identifier),
    Placeholder, // delete this when starting to add expressions
}
