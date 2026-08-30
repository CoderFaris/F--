use std::{
    collections::HashMap,
    fmt::{self, Display},
    sync::LazyLock,
};

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub enum TokenType {
    Illegal,
    Eof,
    Placeholder,
    Ident,
    Int,
    Assign,
    Plus,
    Minus,
    Bang,
    Asterisk,
    Slash,
    GT,
    LT,
    EQ,
    NotEq,
    Comma,
    Semicolon,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Function,
    FunctionTy,
    Colon,
    Let,
    True,
    False,
    Integer,
    Boolean,
    If,
    Else,
    Return,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub literal: String,
}

impl Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(Type:{:#?} Literal:{})", self.token_type, self.literal)
    }
}

// code runs lazily on the first access, only declared here but not allocated or populated yet (until we enter the function)
static KEYWORDS: LazyLock<HashMap<String, TokenType>> = LazyLock::new(|| {
    let mut keywords: HashMap<String, TokenType> = HashMap::new();
    keywords.insert(String::from("fn"), TokenType::Function);
    keywords.insert(String::from("let"), TokenType::Let);
    keywords.insert(String::from("if"), TokenType::If);
    keywords.insert(String::from("else"), TokenType::Else);
    keywords.insert(String::from("return"), TokenType::Return);
    keywords.insert(String::from("true"), TokenType::True);
    keywords.insert(String::from("false"), TokenType::False);
    keywords.insert(String::from("int"), TokenType::Integer);
    keywords.insert(String::from("bool"), TokenType::Boolean);
    keywords
});

pub fn lookup_ident(ident: String) -> TokenType {
    match KEYWORDS.get(&ident) {
        Some(v) => *v,
        None => TokenType::Ident,
    }
}
