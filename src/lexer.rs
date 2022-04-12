use crate::{
    result::LangError,
    token::{Token, TokenType},
};
use std::collections::HashMap;

pub struct Lexer {
    // code: String,
    chars: Vec<char>,
    start: usize,
    curr: usize,
    line: usize,
    keywords: HashMap<&'static str, TokenType>,
}

impl Lexer {
    pub fn new(code: String) -> Self {
        use TokenType::*;
        let keywords = HashMap::from([
            ("or", Or),
            ("and", And),
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
            ("true", True),
            ("false", False),
            ("print", Print),
        ]);

        let chars: Vec<char> = code.chars().collect();
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
                '\n' => {
                    self.line += 1;
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

    fn make_string(&mut self, quote: char) -> Token {
        while self.peek() != quote && !self.at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.next();
        }

        if self.at_end() {
            return self.make_error("Unterminated string".to_string());
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

pub fn _lex(code: String) -> Result<(), LangError> {
    let mut lexer = Lexer::new(code);
    let mut line = 0;
    loop {
        let token = lexer.lex_token();
        if token.line != line {
            print!("{:>4} ", token.line);
            line = token.line;
        } else {
            print!("   | ");
        }
        println!("{:?} {}", token.id, token.lexeme);
        if token.id == TokenType::Eof {
            break;
        }
    }
    Ok(())
}
