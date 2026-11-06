use crate::token::*;

pub struct Lexer {
    input: Vec<u8>,
    position: usize,
    read_position: usize,
    ch: u8,
    lines: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Lexer {
        let mut l = Lexer {
            input: input.as_bytes().to_vec(),
            position: 0,
            read_position: 0,
            ch: 0,
            lines: 0,
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
        // we don't care about whitespace/comments
        self.consume_whitespace_and_comments();

        let tok = match self.ch {
            b'=' => {
                if self.peek_char() == b'=' {
                    let ch = self.ch;
                    self.read_char();
                    let lit = String::from_utf8(vec![ch, self.ch]).expect("Invalid UTF-8 sequence");
                    Token {
                        token_type: TokenType::EQ,
                        literal: lit,
                        line: self.lines,
                    }
                } else {
                    new_token(TokenType::Assign, self.ch, self.lines)
                }
            }
            b';' => new_token(TokenType::Semicolon, self.ch, self.lines),
            b'(' => new_token(TokenType::LParen, self.ch, self.lines),
            b')' => new_token(TokenType::RParen, self.ch, self.lines),
            b'{' => new_token(TokenType::LBrace, self.ch, self.lines),
            b'}' => new_token(TokenType::RBrace, self.ch, self.lines),
            b'[' => new_token(TokenType::LBracket, self.ch, self.lines),
            b']' => new_token(TokenType::RBracket, self.ch, self.lines),
            b'+' => new_token(TokenType::Plus, self.ch, self.lines),
            b'-' => {
                if self.peek_char() == b'>' {
                    let ch = self.ch;
                    self.read_char();
                    let lit = String::from_utf8(vec![ch, self.ch]).expect("Invalid UTF-8 sequence");
                    Token {
                        token_type: TokenType::FunctionTy,
                        literal: lit,
                        line: self.lines,
                    }
                } else {
                    new_token(TokenType::Minus, self.ch, self.lines)
                }
            }
            b'*' => new_token(TokenType::Asterisk, self.ch, self.lines),
            b'/' => new_token(TokenType::Slash, self.ch, self.lines),
            b'<' => new_token(TokenType::LT, self.ch, self.lines),
            b'>' => new_token(TokenType::GT, self.ch, self.lines),
            b'!' => {
                if self.peek_char() == b'=' {
                    let ch = self.ch;
                    self.read_char();
                    let lit = String::from_utf8(vec![ch, self.ch]).expect("Invalid UTF-8 sequence");
                    Token {
                        token_type: TokenType::NotEq,
                        literal: lit,
                        line: self.lines,
                    }
                } else {
                    new_token(TokenType::Bang, self.ch, self.lines)
                }
            }
            b',' => new_token(TokenType::Comma, self.ch, self.lines),
            b':' => new_token(TokenType::Colon, self.ch, self.lines),
            b'\n' => {
                self.lines += 1;
                new_token(TokenType::Newline, self.ch, self.lines)
            }
            0 => Token {
                token_type: TokenType::Eof,
                literal: String::from(""),
                line: self.lines,
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
                        line: self.lines,
                    };
                    // only supports integers for now
                } else if is_digit(self.ch) {
                    let tok_type = TokenType::Int;
                    let lit = self.read_number();
                    let str_lit = match str::from_utf8(lit) {
                        Ok(v) => v,
                        Err(e) => panic!("Invalid UTF-8 sequence, {}", e),
                    };
                    return Token {
                        token_type: tok_type,
                        literal: str_lit.to_string(),
                        line: self.lines,
                    };
                } else {
                    Token {
                        token_type: TokenType::Illegal,
                        literal: String::from(""),
                        line: self.lines,
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

    fn peek_char(&mut self) -> u8 {
        if self.read_position >= self.input.len() {
            0
        } else {
            self.input[self.read_position]
        }
    }

    fn consume_whitespace_and_comments(&mut self) {
        loop {
            while self.ch == b' ' || self.ch == b'\t' || self.ch == b'\r' {
                self.read_char();
            }

            if self.ch == b'/' && self.peek_char() == b'/' {
                while self.ch != b'\n' && self.ch != 0 {
                    self.read_char();
                }
                continue;
            }
            break;
        }
    }
}

fn new_token(tokenType: TokenType, ch: u8, line: usize) -> Token {
    Token {
        token_type: tokenType,
        literal: (ch as char).to_string(),
        line: line,
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
        !-/*5;
        5 < 10 > 5;
        if(5 < 10) {
            return true;
        } else {
            return false;
        }
        10 == 10;
        10 != 9;
        [1, 2];
        ";

        struct TestType {
            expected_type: TokenType,
            expected_literal: &'static str,
        }

        let tests = [
            TestType {
                expected_type: TokenType::Let,
                expected_literal: "let",
            },
            TestType {
                expected_type: TokenType::Ident,
                expected_literal: "five",
            },
            TestType {
                expected_type: TokenType::Assign,
                expected_literal: "=",
            },
            TestType {
                expected_type: TokenType::Int,
                expected_literal: "5",
            },
            TestType {
                expected_type: TokenType::Semicolon,
                expected_literal: ";",
            },
            TestType {
                expected_type: TokenType::Newline,
                expected_literal: "\n",
            },
            TestType {
                expected_type: TokenType::Let,
                expected_literal: "let",
            },
            TestType {
                expected_type: TokenType::Ident,
                expected_literal: "ten",
            },
            TestType {
                expected_type: TokenType::Assign,
                expected_literal: "=",
            },
            TestType {
                expected_type: TokenType::Int,
                expected_literal: "10",
            },
            TestType {
                expected_type: TokenType::Semicolon,
                expected_literal: ";",
            },
            TestType {
                expected_type: TokenType::Newline,
                expected_literal: "\n",
            },
            TestType {
                expected_type: TokenType::Let,
                expected_literal: "let",
            },
            TestType {
                expected_type: TokenType::Ident,
                expected_literal: "add",
            },
            TestType {
                expected_type: TokenType::Assign,
                expected_literal: "=",
            },
            TestType {
                expected_type: TokenType::Function,
                expected_literal: "fn",
            },
            TestType {
                expected_type: TokenType::LParen,
                expected_literal: "(",
            },
            TestType {
                expected_type: TokenType::Ident,
                expected_literal: "x",
            },
            TestType {
                expected_type: TokenType::Comma,
                expected_literal: ",",
            },
            TestType {
                expected_type: TokenType::Ident,
                expected_literal: "y",
            },
            TestType {
                expected_type: TokenType::RParen,
                expected_literal: ")",
            },
            TestType {
                expected_type: TokenType::LBrace,
                expected_literal: "{",
            },
            TestType {
                expected_type: TokenType::Newline,
                expected_literal: "\n",
            },
            TestType {
                expected_type: TokenType::Ident,
                expected_literal: "x",
            },
            TestType {
                expected_type: TokenType::Plus,
                expected_literal: "+",
            },
            TestType {
                expected_type: TokenType::Ident,
                expected_literal: "y",
            },
            TestType {
                expected_type: TokenType::Semicolon,
                expected_literal: ";",
            },
            TestType {
                expected_type: TokenType::Newline,
                expected_literal: "\n",
            },
            TestType {
                expected_type: TokenType::RBrace,
                expected_literal: "}",
            },
            TestType {
                expected_type: TokenType::Semicolon,
                expected_literal: ";",
            },
            TestType {
                expected_type: TokenType::Newline,
                expected_literal: "\n",
            },
            TestType {
                expected_type: TokenType::Let,
                expected_literal: "let",
            },
            TestType {
                expected_type: TokenType::Ident,
                expected_literal: "result",
            },
            TestType {
                expected_type: TokenType::Assign,
                expected_literal: "=",
            },
            TestType {
                expected_type: TokenType::Ident,
                expected_literal: "add",
            },
            TestType {
                expected_type: TokenType::LParen,
                expected_literal: "(",
            },
            TestType {
                expected_type: TokenType::Ident,
                expected_literal: "five",
            },
            TestType {
                expected_type: TokenType::Comma,
                expected_literal: ",",
            },
            TestType {
                expected_type: TokenType::Ident,
                expected_literal: "ten",
            },
            TestType {
                expected_type: TokenType::RParen,
                expected_literal: ")",
            },
            TestType {
                expected_type: TokenType::Semicolon,
                expected_literal: ";",
            },
            TestType {
                expected_type: TokenType::Newline,
                expected_literal: "\n",
            },
            TestType {
                expected_type: TokenType::Bang,
                expected_literal: "!",
            },
            TestType {
                expected_type: TokenType::Minus,
                expected_literal: "-",
            },
            TestType {
                expected_type: TokenType::Slash,
                expected_literal: "/",
            },
            TestType {
                expected_type: TokenType::Asterisk,
                expected_literal: "*",
            },
            TestType {
                expected_type: TokenType::Int,
                expected_literal: "5",
            },
            TestType {
                expected_type: TokenType::Semicolon,
                expected_literal: ";",
            },
            TestType {
                expected_type: TokenType::Newline,
                expected_literal: "\n",
            },
            TestType {
                expected_type: TokenType::Int,
                expected_literal: "5",
            },
            TestType {
                expected_type: TokenType::LT,
                expected_literal: "<",
            },
            TestType {
                expected_type: TokenType::Int,
                expected_literal: "10",
            },
            TestType {
                expected_type: TokenType::GT,
                expected_literal: ">",
            },
            TestType {
                expected_type: TokenType::Int,
                expected_literal: "5",
            },
            TestType {
                expected_type: TokenType::Semicolon,
                expected_literal: ";",
            },
            TestType {
                expected_type: TokenType::Newline,
                expected_literal: "\n",
            },
            TestType {
                expected_type: TokenType::If,
                expected_literal: "if",
            },
            TestType {
                expected_type: TokenType::LParen,
                expected_literal: "(",
            },
            TestType {
                expected_type: TokenType::Int,
                expected_literal: "5",
            },
            TestType {
                expected_type: TokenType::LT,
                expected_literal: "<",
            },
            TestType {
                expected_type: TokenType::Int,
                expected_literal: "10",
            },
            TestType {
                expected_type: TokenType::RParen,
                expected_literal: ")",
            },
            TestType {
                expected_type: TokenType::LBrace,
                expected_literal: "{",
            },
            TestType {
                expected_type: TokenType::Newline,
                expected_literal: "\n",
            },
            TestType {
                expected_type: TokenType::Return,
                expected_literal: "return",
            },
            TestType {
                expected_type: TokenType::True,
                expected_literal: "true",
            },
            TestType {
                expected_type: TokenType::Semicolon,
                expected_literal: ";",
            },
            TestType {
                expected_type: TokenType::Newline,
                expected_literal: "\n",
            },
            TestType {
                expected_type: TokenType::RBrace,
                expected_literal: "}",
            },
            TestType {
                expected_type: TokenType::Else,
                expected_literal: "else",
            },
            TestType {
                expected_type: TokenType::LBrace,
                expected_literal: "{",
            },
            TestType {
                expected_type: TokenType::Newline,
                expected_literal: "\n",
            },
            TestType {
                expected_type: TokenType::Return,
                expected_literal: "return",
            },
            TestType {
                expected_type: TokenType::False,
                expected_literal: "false",
            },
            TestType {
                expected_type: TokenType::Semicolon,
                expected_literal: ";",
            },
            TestType {
                expected_type: TokenType::Newline,
                expected_literal: "\n",
            },
            TestType {
                expected_type: TokenType::RBrace,
                expected_literal: "}",
            },
            TestType {
                expected_type: TokenType::Newline,
                expected_literal: "\n",
            },
            TestType {
                expected_type: TokenType::Int,
                expected_literal: "10",
            },
            TestType {
                expected_type: TokenType::EQ,
                expected_literal: "==",
            },
            TestType {
                expected_type: TokenType::Int,
                expected_literal: "10",
            },
            TestType {
                expected_type: TokenType::Semicolon,
                expected_literal: ";",
            },
            TestType {
                expected_type: TokenType::Newline,
                expected_literal: "\n",
            },
            TestType {
                expected_type: TokenType::Int,
                expected_literal: "10",
            },
            TestType {
                expected_type: TokenType::NotEq,
                expected_literal: "!=",
            },
            TestType {
                expected_type: TokenType::Int,
                expected_literal: "9",
            },
            TestType {
                expected_type: TokenType::Semicolon,
                expected_literal: ";",
            },
            TestType {
                expected_type: TokenType::Newline,
                expected_literal: "\n",
            },
            TestType {
                expected_type: TokenType::LBracket,
                expected_literal: "[",
            },
            TestType {
                expected_type: TokenType::Int,
                expected_literal: "1",
            },
            TestType {
                expected_type: TokenType::Comma,
                expected_literal: ",",
            },
            TestType {
                expected_type: TokenType::Int,
                expected_literal: "2",
            },
            TestType {
                expected_type: TokenType::RBracket,
                expected_literal: "]",
            },
            TestType {
                expected_type: TokenType::Semicolon,
                expected_literal: ";",
            },
            TestType {
                expected_type: TokenType::Newline,
                expected_literal: "\n",
            },
            TestType {
                expected_type: TokenType::Eof,
                expected_literal: "",
            },
        ];

        let mut l = Lexer::new(input);

        for tt in &tests {
            let token = l.next_token();

            assert_eq!(
                tt.expected_type, token.token_type,
                "tokentype wrong. expected={:#?}, got={:#?}",
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
