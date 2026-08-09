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
    keywords.insert(String::from("if"), IF);
    keywords.insert(String::from("else"), ELSE);
    keywords.insert(String::from("return"), RETURN);
    keywords.insert(String::from("true"), TRUE);
    keywords.insert(String::from("false"), FALSE);
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
pub const MINUS: TokenType = "-";
pub const BANG: TokenType = "!";
pub const ASTERISK: TokenType = "*";
pub const SLASH: TokenType = "/";
pub const LT: TokenType = "<";
pub const GT: TokenType = ">";
pub const EQ: TokenType = "==";
pub const NOT_EQ: TokenType = "!=";
pub const COMMA: TokenType = ",";
pub const SEMICOLON: TokenType = ";";
pub const LPAREN: TokenType = "(";
pub const RPAREN: TokenType = ")";
pub const LBRACE: TokenType = "{";
pub const RBRACE: TokenType = "}";
pub const FUNCTION: TokenType = "FUNCTION";
pub const LET: TokenType = "LET";
pub const TRUE: TokenType = "TRUE";
pub const FALSE: TokenType = "FALSE";
pub const IF: TokenType = "IF";
pub const ELSE: TokenType = "ELSE";
pub const RETURN: TokenType = "RETURN";
