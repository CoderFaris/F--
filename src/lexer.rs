use crate::token::*;


pub struct Lexer {
    input: Vec<u8>,
    position: usize,
    read_position: usize,
    ch: u8,
}

impl Lexer {
    pub fn new(input: &str) -> Lexer {
        let mut l = Lexer{input: input.as_bytes().to_vec(), position: 0, read_position: 0, ch: 0}; // a possible issue here with initializing pos, read_pos and ch
        l.read_char();
        return l;   
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = 0;
        } else {
            self.ch = self.input[self.read_position];
        }

        self.position = self.read_position;
        self.read_position += 1;

    }

    pub fn next_token(&mut self) -> Token {
        

        let tok = match self.ch {
            b'=' => new_token(ASSIGN, self.ch),
            b';' => new_token(SEMICOLON, self.ch),
            b'(' => new_token(LPAREN, self.ch),
            b')' => new_token(RPAREN, self.ch),
            b'{' => new_token(LBRACE, self.ch),
            b'}' => new_token(RBRACE, self.ch),
            b'+' => new_token(PLUS, self.ch),
            b',' => new_token(COMMA, self.ch),
            0 => Token{token_type: EOF, literal: String::from("")},
            0_u8..=60_u8 | 62_u8..=u8::MAX => todo!("These are not needed"), // feels like a weird solution but it works ig
        };

        self.read_char();
        return tok;
    }
}

fn new_token(tokenType: &'static str, ch: u8) -> Token {
    Token { token_type: tokenType, literal:  (ch as char).to_string()}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_token() {
        let input = "let five = 5;
        let ten = 10;

        let add = fn(x, y) {
            x + y;
        };
        let result = add(five, ten);
        ";

        struct TestType {
            expected_type: TokenType,
            expected_literal: &'static str
        }

        let tests = [
            TestType{
                expected_type: LET,
                expected_literal: "let"
            },
            TestType{
                expected_type: IDENT,
                expected_literal: "five"
            },
            TestType{
                expected_type: ASSIGN,
                expected_literal: "="
            },
            TestType{
                expected_type: INT,
                expected_literal: "5"
            },
            TestType{
                expected_type: SEMICOLON,
                expected_literal: ";"
            },
            TestType{
                expected_type: LET,
                expected_literal: "let"
            },
            TestType{
                expected_type: IDENT,
                expected_literal: "ten"
            },
            TestType{
                expected_type: ASSIGN,
                expected_literal: "="
            },
            TestType{
                expected_type: INT,
                expected_literal: "10"
            },
            TestType{
                expected_type: SEMICOLON,
                expected_literal: ";"
            },
            TestType{
                expected_type: LET,
                expected_literal: "let"
            },
            TestType{
                expected_type: IDENT,
                expected_literal: "add"
            },
            TestType{
                expected_type: ASSIGN,
                expected_literal: "="
            },
            TestType{
                expected_type: FUNCTION,
                expected_literal: "fn"
            },
            TestType{
                expected_type: LPAREN,
                expected_literal: "("
            },
            TestType {
                expected_type: IDENT,
                expected_literal: "x",
            },
            TestType {
                expected_type: COMMA,
                expected_literal: ",",
            },
            TestType {
                expected_type: IDENT,
                expected_literal: "y",
            },
            TestType {
                expected_type: RPAREN,
                expected_literal: ")",
            },
            TestType {
                expected_type: LBRACE,
                expected_literal: "{",
            },
            TestType {
                expected_type: IDENT,
                expected_literal: "x",
            },
            TestType {
                expected_type: PLUS,
                expected_literal: "+",
            },
            TestType {
                expected_type: IDENT,
                expected_literal: "y",
            },
            TestType {
                expected_type: SEMICOLON,
                expected_literal: ";",
            },
            TestType {
                expected_type: RBRACE,
                expected_literal: "}",
            },
            TestType {
                expected_type: SEMICOLON,
                expected_literal: ";",
            },
            TestType {
                expected_type: LET,
                expected_literal: "let",
            },
        ];

        let mut l = Lexer::new(input);

        for tt in &tests {
            let token = l.next_token();

            assert_eq!(tt.expected_type, token.token_type, "tokentype wrong. expected={}, got={}", tt.expected_type, token.token_type);
            assert_eq!(tt.expected_literal.to_string(), token.literal, "tokentype wrong. expected={}, got={}", tt.expected_literal, token.literal);
        }

    }

}