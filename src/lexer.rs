use crate::token::*;

pub struct Lexer {
    input: Vec<u8>,
    position: usize,
    read_position: usize,
    ch: u8,
}

impl Lexer {
    pub fn new(input: &str) -> Lexer {
        let mut l = Lexer {
            input: input.as_bytes().to_vec(),
            position: 0,
            read_position: 0,
            ch: 0,
        };
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
        // we don't care about whitespace
        self.consume_whitespace();

        let tok = match self.ch {
            b'=' => new_token(ASSIGN, self.ch),
            b';' => new_token(SEMICOLON, self.ch),
            b'(' => new_token(LPAREN, self.ch),
            b')' => new_token(RPAREN, self.ch),
            b'{' => new_token(LBRACE, self.ch),
            b'}' => new_token(RBRACE, self.ch),
            b'+' => new_token(PLUS, self.ch),
            b',' => new_token(COMMA, self.ch),
            0 => Token {
                token_type: EOF,
                literal: String::from(""),
            },
            0_u8..=60_u8 | 62_u8..=u8::MAX => {
                if is_letter(self.ch) {
                    let lit = self.read_identifier();
                    let str_lit = match str::from_utf8(lit) {
                        Ok(v) => v,
                        Err(e) => panic!("Invalid UTF-8 sequence, {}", e),
                    };
                    let tok_type = lookup_ident(str_lit.to_string());
                    // early return so that it does not get other letters, it won't make sense ;)
                    return Token {
                        token_type: tok_type,
                        literal: str_lit.to_string(),
                    };
                    // only supports integers for now
                } else if is_digit(self.ch) {
                    let tok_type = INT;
                    let lit = self.read_number();
                    let str_lit = match str::from_utf8(lit) {
                        Ok(v) => v,
                        Err(e) => panic!("Invalid UTF-8 sequence, {}", e),
                    };
                    return Token {
                        token_type: tok_type,
                        literal: str_lit.to_string(),
                    };
                } else {
                    Token {
                        token_type: ILLEGAL,
                        literal: String::from(""),
                    }
                }
            }
        };

        self.read_char();
        return tok;
    }

    fn read_identifier(&mut self) -> &[u8] {
        let position = self.position;
        while is_letter(self.ch) {
            self.read_char();
        }
        &self.input[position..self.position]
    }

    fn read_number(&mut self) -> &[u8] {
        let position = self.position;

        while is_digit(self.ch) {
            self.read_char();
        }

        &self.input[position..self.position]
    }

    fn consume_whitespace(&mut self) {
        while self.ch == b' ' || self.ch == b'\t' || self.ch == b'\n' || self.ch == b'\r' {
            self.read_char();
        }
    }
}

fn new_token(tokenType: &'static str, ch: u8) -> Token {
    Token {
        token_type: tokenType,
        literal: (ch as char).to_string(),
    }
}

fn is_letter(ch: u8) -> bool {
    (b'a' <= ch && ch <= b'z') || (b'A' <= ch && ch <= b'Z') || ch == b'_'
}

fn is_digit(ch: u8) -> bool {
    b'0' <= ch && ch <= b'9'
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
            expected_literal: &'static str,
        }

        let tests = [
            TestType {
                expected_type: LET,
                expected_literal: "let",
            },
            TestType {
                expected_type: IDENT,
                expected_literal: "five",
            },
            TestType {
                expected_type: ASSIGN,
                expected_literal: "=",
            },
            TestType {
                expected_type: INT,
                expected_literal: "5",
            },
            TestType {
                expected_type: SEMICOLON,
                expected_literal: ";",
            },
            TestType {
                expected_type: LET,
                expected_literal: "let",
            },
            TestType {
                expected_type: IDENT,
                expected_literal: "ten",
            },
            TestType {
                expected_type: ASSIGN,
                expected_literal: "=",
            },
            TestType {
                expected_type: INT,
                expected_literal: "10",
            },
            TestType {
                expected_type: SEMICOLON,
                expected_literal: ";",
            },
            TestType {
                expected_type: LET,
                expected_literal: "let",
            },
            TestType {
                expected_type: IDENT,
                expected_literal: "add",
            },
            TestType {
                expected_type: ASSIGN,
                expected_literal: "=",
            },
            TestType {
                expected_type: FUNCTION,
                expected_literal: "fn",
            },
            TestType {
                expected_type: LPAREN,
                expected_literal: "(",
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

            assert_eq!(
                tt.expected_type, token.token_type,
                "tokentype wrong. expected={}, got={}",
                tt.expected_type, token.token_type
            );
            assert_eq!(
                tt.expected_literal.to_string(),
                token.literal,
                "tokentype wrong. expected={}, got={}",
                tt.expected_literal,
                token.literal
            );
        }
    }
}
