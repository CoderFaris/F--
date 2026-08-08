use std::{collections::HashMap, sync::LazyLock};

pub type TokenType = &'static str;

pub struct Token {
    pub token_type: TokenType,
    pub literal: String,
}

// code runs lazily on the first access, only declared here but not allocated or populated yet
static KEYWORDS: LazyLock<HashMap<String, TokenType>> = LazyLock::new(|| {
    let mut keywords: HashMap<String, TokenType> = HashMap::new();
    keywords.insert(String::from("fn"), FUNCTION);
    keywords.insert(String::from("let"), LET);
    keywords
});

pub fn lookup_ident(ident: String) -> TokenType {
    match KEYWORDS.get(&ident) {
        Some(v) => v,
        None => IDENT,
    }
}

pub const ILLEGAL: TokenType = "ILLEGAL";
pub const EOF: TokenType = "EOF";
pub const IDENT: TokenType = "IDENT";
pub const INT: TokenType = "INT";
pub const ASSIGN: TokenType = "=";
pub const PLUS: TokenType = "+";
pub const COMMA: TokenType = ",";
pub const SEMICOLON: TokenType = ";";
pub const LPAREN: TokenType = "(";
pub const RPAREN: TokenType = ")";
pub const LBRACE: TokenType = "{";
pub const RBRACE: TokenType = "}";
pub const FUNCTION: TokenType = "FUNCTION";
pub const LET: TokenType = "LET";
