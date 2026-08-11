use crate::token::Token;
use std::any::Any;
pub struct LetStatement {
    pub token: Token,
    pub name: Identifier,
    pub value: Box<dyn Expression>,
}

pub struct Identifier {
    pub token: Token,
    pub value: String,
}

pub struct Program {
    pub statements: Vec<Box<dyn Statement>>,
}

pub trait Node: Any {
    fn token_literal(&self) -> String;
    fn as_any(&self) -> &dyn Any;
}

impl Node for LetStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Node for Identifier {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn as_any(&self) -> &dyn Any {
        self
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
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub trait Statement: Node {}
pub trait Expression: Node {}
