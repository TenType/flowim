#[derive(Clone)]
pub struct Token {
    pub id: TokenType,
    pub lexeme: String,
    pub line: usize,
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Plus,
    Minus,
    Semicolon,
    Slash,
    Star,

    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    Identifier,
    Str,
    Int,
    Float,

    Or,
    And,
    If,
    Else,
    While,
    For,
    Var,
    Let,
    Fn,
    Return,
    Class,
    Super,
    SelfKw,
    True,
    False,
    Print,

    Error,
    Eof,
}
