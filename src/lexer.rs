use crate::token::{Token, TokenType};
use std::collections::HashMap;

pub struct Lexer {
    chars: Vec<char>,
    start: usize,
    curr: usize,
    line: usize,
    keywords: HashMap<&'static str, TokenType>,
}

impl Lexer {
    pub fn new(code: &str) -> Self {
        use TokenType::*;

        // Update the `keywords` unit test after changing any keywords
        let keywords = HashMap::from([
            ("or", Or),
            ("and", And),
            ("not", Not),
            ("if", If),
            ("else", Else),
            ("while", While),
            ("for", For),
            ("var", Var),
            ("let", Let),
            ("fn", Fn),
            ("return", Return),
            ("class", Class),
            ("super", Super),
            ("self", SelfKw),
            ("true", Bool),
            ("false", Bool),
            ("print", Print),
            ("do", Do),
            ("end", End),
        ]);

        let mut chars: Vec<char> = code.chars().collect();
        chars.push('\0');

        Lexer {
            chars,
            start: 0,
            curr: 0,
            line: 1,
            keywords,
        }
    }
    pub fn lex_token(&mut self) -> Token {
        use TokenType::*;

        self.skip_whitespace();
        self.start = self.curr;
        if self.at_end() {
            return self.make_token(Eof);
        }

        let curr = self.next();

        let token = match curr {
            '\n' => return self.make_newline(),
            '(' => LeftParen,
            ')' => RightParen,
            '{' => LeftBrace,
            '}' => RightBrace,
            ';' => Semicolon,
            ',' => Comma,
            '.' => Dot,
            '+' => Plus,
            '-' => Minus,
            '*' => Star,
            '/' => Slash,
            '!' => self.if_eq(BangEqual, Bang),
            '=' => self.if_eq(EqualEqual, Equal),
            '<' => self.if_eq(LessEqual, Less),
            '>' => self.if_eq(GreaterEqual, Greater),
            '"' | '\'' => return self.make_string(curr),
            curr if curr.is_digit(10) => return self.make_number(),
            curr if curr.is_alphabetic() || curr == '_' => return self.make_identifier(),
            _ => return self.make_error(format!("Unexpected character: {}", curr)),
        };

        self.make_token(token)
    }

    fn make_token(&self, id: TokenType) -> Token {
        Token {
            id,
            lexeme: self.make_lexeme(),
            line: self.line,
        }
    }

    fn make_error(&self, message: String) -> Token {
        Token {
            id: TokenType::Error,
            lexeme: message,
            line: self.line,
        }
    }

    fn make_lexeme(&self) -> String {
        self.chars[self.start..self.curr].iter().collect()
    }

    fn skip_whitespace(&mut self) {
        while !self.at_end() {
            match self.peek() {
                ' ' | '\r' | '\t' => {
                    self.next();
                }
                '/' if self.peek_next() == '/' => {
                    while self.peek() != '\n' && !self.at_end() {
                        self.next();
                    }
                }
                _ => return,
            }
        }
    }

    fn make_newline(&mut self) -> Token {
        self.line += 1;
        self.make_token(TokenType::Newline)
    }

    fn make_string(&mut self, quote: char) -> Token {
        while self.peek() != quote && !self.at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.next();
        }

        if self.at_end() {
            return self.make_error(String::from("Unterminated string"));
        }
        self.next();
        self.make_token(TokenType::Str)
    }

    fn make_number(&mut self) -> Token {
        while self.peek().is_digit(10) {
            self.next();
        }

        let id = if self.peek() == '.' && self.peek_next().is_digit(10) {
            self.next();
            while self.peek().is_digit(10) {
                self.next();
            }
            TokenType::Float
        } else {
            TokenType::Int
        };

        self.make_token(id)
    }

    fn make_identifier(&mut self) -> Token {
        while self.peek().is_alphabetic() || self.peek() == '_' || self.peek().is_digit(10) {
            self.next();
        }

        self.make_token(
            *self
                .keywords
                .get(&self.make_lexeme().as_str())
                .unwrap_or(&TokenType::Identifier),
        )
    }

    fn matches(&mut self, expect: char) -> bool {
        if self.at_end() || self.peek() != expect {
            false
        } else {
            self.curr += 1;
            true
        }
    }

    fn if_eq(&mut self, if_true: TokenType, if_false: TokenType) -> TokenType {
        if self.matches('=') {
            if_true
        } else {
            if_false
        }
    }

    fn next(&mut self) -> char {
        self.curr += 1;
        self.chars[self.curr - 1]
    }

    fn peek(&self) -> char {
        self.chars[self.curr]
    }

    fn peek_next(&self) -> char {
        if self.at_end() {
            '\0'
        } else {
            self.chars[self.curr + 1]
        }
    }

    fn at_end(&self) -> bool {
        self.curr >= self.chars.len() - 1 || self.peek() == '\0'
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Lexer,
        TokenType::{self, *},
    };

    fn lex(code: &str) -> Vec<TokenType> {
        let mut lexer = Lexer::new(code);
        let mut result: Vec<TokenType> = vec![];
        loop {
            let token = lexer.lex_token();
            let id = token.id;
            result.push(token.id);

            if id == Eof {
                break;
            }
        }
        result
    }

    #[test]
    fn unknown_chars() {
        let expected = vec![Identifier, Error, Error, Bang, Identifier, Error, Eof];
        let actual = lex("hello~ @! test &");
        assert_eq!(expected, actual);
    }

    #[test]
    fn numbers() {
        let expected = vec![Int, Float, Float, Eof];
        let actual = lex("345 1.0 1.2");
        assert_eq!(expected, actual);

        let expected = vec![Int, Dot, Int, Dot, Float, Float, Dot, Int, Eof];
        let actual = lex("1. 0. 5.5 0.0.5");
        assert_eq!(expected, actual);

        let expected = vec![Minus, Float, Identifier, Eof];
        let actual = lex("-3.14a");
        assert_eq!(expected, actual);
    }

    #[test]
    fn expressions() {
        let expected = vec![
            Int, Slash, Int, Plus, Int, Star, Int, Minus, Minus, Int, Eof,
        ];
        let actual = lex("22 / 2 + 42 * 1 - -4");
        assert_eq!(expected, actual);

        let expected = vec![Float, Star, LeftParen, Int, Plus, Float, RightParen, Eof];
        let actual = lex("5.5 * (2 + 1.0)");
        assert_eq!(expected, actual);
    }

    #[test]
    fn strings() {
        let expected = vec![Str, Eof];
        let actual = lex("'single'");
        assert_eq!(expected, actual);

        let expected = vec![Str, Eof];
        let actual = lex("\"double\"");
        assert_eq!(expected, actual);
    }

    #[test]
    fn unterminated_strings() {
        let expected = vec![Error, Eof];
        let actual = lex("'whoops");
        assert_eq!(expected, actual);

        let expected = vec![Error, Eof];
        let actual = lex("\"nope");
        assert_eq!(expected, actual);
    }

    #[test]
    fn identifiers() {
        let expected = vec![Identifier, Identifier, Eof];
        let actual = lex("hello world");
        assert_eq!(expected, actual);
    }

    #[test]
    fn booleans() {
        let expected = vec![Bool, Bool, Eof];
        let actual = lex("true false");
        assert_eq!(expected, actual);
    }

    #[test]
    fn keywords() {
        let expected = vec![
            Or, And, Not, If, Else, While, For, Var, Let, Fn, Return, Class, Super, SelfKw, Print,
            Do, End, Eof,
        ];
        let actual =
            lex("or and not if else while for var let fn return class super self print do end");
        assert_eq!(expected, actual);
    }

    #[test]
    fn skip_whitespace() {
        let expected = vec![Int, Int, Eof];
        let actual = lex("1        2");
        assert_eq!(expected, actual);

        let expected = vec![Int, Newline, Newline, Newline, Int, Eof];
        let actual = lex("3\n\n\n4");
        assert_eq!(expected, actual);

        let expected = vec![Int, Int, Eof];
        let actual = lex("5\t\t6");
        assert_eq!(expected, actual);

        let expected = vec![Int, Int, Eof];
        let actual = lex("7\r\r\r\r8");
        assert_eq!(expected, actual);

        let expected = vec![Int, Newline, Int, Newline, Int, Newline, Newline, Eof];
        let actual = lex("9   \n10   \t \n11\n\n");
        assert_eq!(expected, actual);
    }

    #[test]
    fn skip_comments() {
        let expected = vec![Int, Plus, Int, Eof];
        let actual = lex("1 + 2 // this is a comment");
        assert_eq!(expected, actual);
    }
}
