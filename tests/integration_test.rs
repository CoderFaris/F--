use compiler::token::*;
use compiler::lexer::Lexer;

#[test]
fn test_next_token() {
    let input = "=+(){},;";

    struct TestType {
        expected_type: TokenType,
        expected_literal: &'static str
    }

    let tests = [
        TestType{
            expected_type: ASSIGN,
            expected_literal: "="
        },
        TestType{
            expected_type: PLUS,
            expected_literal: "+"
        },
        TestType{
            expected_type: LPAREN,
            expected_literal: "("
        },
        TestType{
            expected_type: RPAREN,
            expected_literal: ")"
        },
        TestType{
            expected_type: LBRACE,
            expected_literal: "{"
        },
        TestType{
            expected_type: RBRACE,
            expected_literal: "}"
        },
        TestType{
            expected_type: COMMA,
            expected_literal: ","
        },
        TestType{
            expected_type: SEMICOLON,
            expected_literal: ";"
        },
        TestType{
            expected_type: EOF,
            expected_literal: ""
        },

    ];

    let l = Lexer::new(input);

    for tt in tests.iter() {
        let token = l.next_token();

        // assert_eq!(token.token_type, tt.expected_type, "tokentype wrong. expected={}, got={token.token_type}", tt.expected_type);
        // assert_eq!(token.literal, tt.expected_literal, "tokentype wrong. expected={}, got={token.literal}", tt.expected_literal);
    }

}