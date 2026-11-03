use crate::{ast, token::Token, types::Type};
use std::any::Any;

#[derive(Clone, Debug)]
pub struct LetStatement {
    pub token: Token,
    pub name: Identifier,
    pub value: Expression,
}

#[derive(Clone, Debug)]
pub struct ReturnStatement {
    pub token: Token,
    pub return_value: Expression,
}

#[derive(Clone, Debug)]
pub struct ExpressionStatement {
    pub token: Token,
    pub expression: Expression,
}

#[derive(Clone, Debug)]
pub struct BlockStatement {
    pub token: Token,
    pub statements: Vec<Statement>,
}

#[derive(Clone, Debug)]
pub struct FunctionLiteral {
    pub token: Token,
    pub parameters: Vec<Identifier>,
    pub return_type: Type,
    pub body: BlockStatement,
    pub ty: Type,
}

#[derive(Clone, Debug)]
pub struct PrefixExpression {
    pub token: Token,
    pub operator: String,
    pub right: Expression,
    pub ty: Type,
}

#[derive(Clone, Debug)]
pub struct InfixExpression {
    pub token: Token,
    pub left: Expression,
    pub operator: String,
    pub right: Expression,
    pub ty: Type,
}

#[derive(Clone, Debug)]
pub struct IfExpression {
    pub token: Token,
    pub condition: Expression,
    pub consequence: BlockStatement,
    pub alternative: Option<BlockStatement>,
    pub ty: Type,
}

#[derive(Clone, Debug)]
pub struct WhileExpression {
    pub token: Token,
    pub condition: Expression,
    pub body: BlockStatement,
    pub ty: Type,
}

#[derive(Clone, Debug)]
pub struct CallExpression {
    pub token: Token,
    pub function: Expression,
    pub arguments: Vec<Expression>,
    pub ty: Type,
}

#[derive(Clone, Debug)]
pub struct Identifier {
    pub token: Token,
    pub value: String,
    pub ty: Type,
}

#[derive(Clone, Debug)]
pub struct IntegerLiteral {
    pub token: Token,
    pub value: isize,
    pub ty: Type,
}

#[derive(Clone, Debug)]
pub struct Boolean {
    pub token: Token,
    pub value: bool,
    pub ty: Type,
}

#[derive(Clone, Debug)]
pub struct ArrayLiteral {
    pub token: Token,
    pub elements: Vec<Expression>,
    pub ty: Type,
}
#[derive(Clone, Debug)]
pub struct IndexExpression {
    pub token: Token,
    pub left: Expression,
    pub index: Expression,
    pub ty: Type,
}

#[derive(Clone, Debug)]
pub struct AssignExpression {
    pub token: Token,
    pub target: Identifier,
    pub value: Expression,
    pub ty: Type,
}

#[derive(Debug)]
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

        s.push_str(&self.value.string());

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

        s.push_str(&stmt_str[..]);
        s.push_str(&self.return_value.string());
        s.push_str("; ");
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

impl Node for BlockStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        let mut s = String::new();
        for stmt in &self.statements {
            s += &stmt.string()[..];
        }
        s
    }
}

impl Node for AssignExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        format!("{:#?} = {:#?}", self.target, self.value)
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

impl Node for IfExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        let mut s = String::new();

        s += "if";
        s += &self.condition.string();
        s += " ";
        s += &self.consequence.string();
        if self.alternative.is_some() {
            s += "else ";
            s += &self.alternative.clone().unwrap().string()[..];
        }

        s
    }
}

impl Node for WhileExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        format!("while {} {}", self.condition.string(), self.body.string())
    }
}

impl Node for CallExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        let mut s = String::new();
        let mut arguments = Vec::new();
        for arg in &self.arguments {
            arguments.push(arg.string());
        }

        s += &self.function.string();
        s += "(";
        s += &arguments.join(", ");
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

impl Node for Boolean {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        self.token.literal.clone()
    }
}

impl Node for ArrayLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        let mut elements = Vec::new();
        for el in self.elements.iter() {
            elements.push(el.string());
        }
        format!("[{}]", elements.join(", "))
    }
}

impl Node for IndexExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        format!("({}[{}])", self.left.string(), self.index.string())
    }
}

impl Node for FunctionLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        let mut s = String::new();
        let mut params = Vec::new();

        for param in &self.parameters {
            params.push(param.string());
        }

        s += &self.token_literal();
        s += " (";
        s += &params.join(", ");
        s += ") ";
        s += "{ ";
        s += &self.body.string();
        s += "}";
        s
    }
}

impl Node for Statement {
    fn token_literal(&self) -> String {
        match self {
            Statement::Let(stmt) => stmt.token_literal(),
            Statement::Return(stmt) => stmt.token_literal(),
            Statement::Expression(stmt) => stmt.token_literal(),
            Statement::Block(stmt) => stmt.token_literal(),
        }
    }

    fn string(&self) -> String {
        match self {
            Statement::Let(stmt) => stmt.string(),
            Statement::Return(stmt) => stmt.string(),
            Statement::Expression(stmt) => stmt.string(),
            Statement::Block(stmt) => stmt.string(),
        }
    }
}

impl Node for Expression {
    fn token_literal(&self) -> String {
        match self {
            Expression::Ident(i) => i.token_literal(),
            Expression::Int(int) => int.token_literal(),
            Expression::Bool(b) => b.token_literal(),
            Expression::Prefix(pre) => pre.token_literal(),
            Expression::Infix(inf) => inf.token_literal(),
            Expression::If(ifexp) => ifexp.token_literal(),
            Expression::While(w) => w.token_literal(),
            Expression::Function(fun) => fun.token_literal(),
            Expression::Call(c) => c.token_literal(),
            Expression::Array(a) => a.token_literal(),
            Expression::Index(index) => index.token_literal(),
            Expression::Assign(expr) => expr.token_literal(),
            Expression::Placeholder => String::from("delete this"),
        }
    }

    fn string(&self) -> String {
        match self {
            Expression::Ident(i) => i.string(),
            Expression::Int(int) => int.string(),
            Expression::Bool(b) => b.string(),
            Expression::Prefix(pre) => pre.string(),
            Expression::Infix(inf) => inf.string(),
            Expression::If(ifexp) => ifexp.string(),
            Expression::While(w) => w.string(),
            Expression::Function(fun) => fun.string(),
            Expression::Call(c) => c.string(),
            Expression::Array(a) => a.string(),
            Expression::Index(index) => index.string(),
            Expression::Assign(expr) => expr.string(),
            Expression::Placeholder => String::from("delete this"),
        }
    }
}

impl Expression {
    pub fn ty(&self) -> &Type {
        match self {
            Expression::Int(int) => &int.ty,
            Expression::Bool(bool) => &bool.ty,
            Expression::Prefix(pre) => &pre.ty,
            Expression::Infix(inf) => &inf.ty,
            Expression::Call(c) => &c.ty,
            Expression::If(iff) => &iff.ty,
            Expression::While(w) => &w.ty,
            Expression::Ident(ident) => &ident.ty,
            Expression::Function(f) => &f.ty,
            Expression::Array(a) => &a.ty,
            Expression::Index(index) => &index.ty,
            Expression::Assign(expr) => &expr.ty,
            Expression::Placeholder => &Type::Unknown,
        }
    }

    pub fn set_ty(&mut self, ty: Type) {
        match self {
            Expression::Int(int) => int.ty = ty,
            Expression::Bool(bool) => bool.ty = ty,
            Expression::Prefix(pre) => pre.ty = ty,
            Expression::Infix(inf) => inf.ty = ty,
            Expression::Call(c) => c.ty = ty,
            Expression::If(iff) => iff.ty = ty,
            Expression::While(w) => w.ty = ty,
            Expression::Ident(ident) => ident.ty = ty,
            Expression::Function(f) => f.ty = ty,
            Expression::Array(a) => a.ty = ty,
            Expression::Index(index) => index.ty = ty,
            Expression::Assign(expr) => expr.ty = ty,
            Expression::Placeholder => panic!("this won't be reached :)"),
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

#[derive(Clone, Debug)]
pub enum Statement {
    Let(LetStatement),
    Return(ReturnStatement),
    Expression(ExpressionStatement),
    Block(BlockStatement),
}

#[derive(Clone, Debug)]
pub enum Expression {
    Ident(Identifier),
    Int(IntegerLiteral),
    Bool(Boolean),
    Prefix(Box<PrefixExpression>),
    Infix(Box<InfixExpression>),
    If(Box<IfExpression>),
    While(Box<WhileExpression>),
    Function(FunctionLiteral),
    Call(Box<CallExpression>),
    Array(ArrayLiteral),
    Index(Box<IndexExpression>),
    Assign(Box<AssignExpression>),
    Placeholder,
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
                    ty: Type::Unknown,
                },
                value: Expression::Ident(Identifier {
                    token: Token {
                        token_type: crate::token::TokenType::Ident,
                        literal: "another_var".to_string(),
                    },
                    value: "another_var".to_string(),
                    ty: Type::Unknown,
                }),
            })],
        };

        assert_eq!(
            program.string(),
            "let my_var = another_var; \n",
            "wrong string, got={}",
            program.string()
        );
    }
}
