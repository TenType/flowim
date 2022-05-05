#[derive(Clone, Debug)]
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
    Newline,

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
    Bool,

    Or,
    And,
    Not,
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
    Print,
    Do,
    End,

    Error,
    Eof,
}
