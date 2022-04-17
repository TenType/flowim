use crate::{
    chunk::{Chunk, OpCode, Value},
    lexer::Lexer,
    result::LangError,
    token::{Token, TokenType},
};
use std::collections::HashMap;

#[derive(PartialEq, PartialOrd)]
enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl Precedence {
    fn next(&self) -> Self {
        use Precedence::*;
        match self {
            None => Assignment,
            Assignment => Or,
            Or => And,
            And => Equality,
            Equality => Comparison,
            Comparison => Term,
            Term => Factor,
            Factor => Unary,
            Unary => Call,
            Call => Primary,
            Primary => panic!("No rule higher than Primary"),
        }
    }
}

struct ParseRule {
    prefix: Option<fn(&mut Compiler) -> ()>,
    infix: Option<fn(&mut Compiler) -> ()>,
    precedence: Precedence,
}

struct Compiler {
    chunk: Chunk,
    lexer: Lexer,
    curr: Token,
    prev: Token,
    had_error: bool,
    panic_mode: bool,
    rules: HashMap<TokenType, ParseRule>,
}

impl Compiler {
    fn new(code: &str) -> Self {
        use Precedence as P;
        use TokenType::*;

        let rule = |prefix, infix, precedence| ParseRule {
            prefix,
            infix,
            precedence,
        };
        // let empty = || rule(None, None, P::None);

        let rules = HashMap::from([
            (LeftParen, rule(Some(Self::group), None, P::None)),
            (Minus, rule(Some(Self::unary), Some(Self::binary), P::Term)),
            (Plus, rule(None, Some(Self::binary), P::Term)),
            (Slash, rule(None, Some(Self::binary), P::Factor)),
            (Star, rule(None, Some(Self::binary), P::Factor)),
            (Int, rule(Some(Self::int), None, P::None)),
            (Float, rule(Some(Self::float), None, P::None)),
            (True, rule(Some(Self::literal), None, P::None)),
            (False, rule(Some(Self::literal), None, P::None)),
            (Not, rule(Some(Self::unary), None, P::None)),
            (BangEqual, rule(None, Some(Self::binary), P::Equality)),
            (EqualEqual, rule(None, Some(Self::binary), P::Equality)),
            (Greater, rule(None, Some(Self::binary), P::Comparison)),
            (GreaterEqual, rule(None, Some(Self::binary), P::Comparison)),
            (Less, rule(None, Some(Self::binary), P::Comparison)),
            (LessEqual, rule(None, Some(Self::binary), P::Comparison)),
        ]);

        Self {
            chunk: Chunk::new(),
            lexer: Lexer::new(code),
            curr: Token {
                id: TokenType::Eof,
                lexeme: String::new(),
                line: 1,
            },
            prev: Token {
                id: TokenType::Eof,
                lexeme: String::new(),
                line: 1,
            },
            had_error: false,
            panic_mode: false,
            rules,
        }
    }

    fn compile(&mut self) -> bool {
        self.next();
        self.expression();
        self.end_compile();
        self.eat(TokenType::Eof, "Expected to reach the end of the file");
        !self.had_error
    }

    fn next(&mut self) {
        self.prev = self.curr.clone();

        loop {
            self.curr = self.lexer.lex_token();
            if self.curr.id != TokenType::Error {
                break;
            }
            self.error_curr(&self.curr.lexeme.clone());
        }
    }

    fn eat(&mut self, id: TokenType, message: &str) {
        if self.curr.id == id {
            self.next();
            return;
        }
        self.error_curr(message);
    }

    fn emit(&mut self, op: OpCode) {
        self.chunk.write(op, self.prev.line);
    }

    fn emit_two(&mut self, op1: OpCode, op2: OpCode) {
        self.chunk.write(op1, self.prev.line);
        self.chunk.write(op2, self.prev.line);
    }

    fn emit_constant(&mut self, value: Value) {
        let index = self.chunk.add_constant(value);
        self.emit(index);
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn int(&mut self) {
        let value = self.prev.lexeme.parse::<isize>().unwrap();
        self.emit_constant(Value::Int(value));
    }

    fn float(&mut self) {
        let value = self.prev.lexeme.parse::<f64>().unwrap();
        self.emit_constant(Value::Float(value));
    }

    fn group(&mut self) {
        self.expression();
        self.eat(TokenType::RightParen, "Expected closing parenthesis ')'");
    }

    fn unary(&mut self) {
        let operator_id = self.prev.id;

        self.parse_precedence(Precedence::Unary);

        match operator_id {
            TokenType::Minus => self.emit(OpCode::Negate),
            TokenType::Not => self.emit(OpCode::Not),
            _ => (),
        }
    }

    fn binary(&mut self) {
        let operator_id = self.prev.id;
        let rule = self.get_rule(operator_id).precedence.next();

        self.parse_precedence(rule);

        use OpCode::*;
        use TokenType::*;
        match operator_id {
            Plus => self.emit(Add),
            Minus => self.emit(Subtract),
            Star => self.emit(Multiply),
            Slash => self.emit(Divide),
            BangEqual => self.emit_two(OpCode::Equal, OpCode::Not),
            EqualEqual => self.emit(OpCode::Equal),
            TokenType::Greater => self.emit(OpCode::Greater),
            GreaterEqual => self.emit_two(OpCode::Less, OpCode::Not),
            TokenType::Less => self.emit(OpCode::Less),
            LessEqual => self.emit_two(OpCode::Greater, OpCode::Not),
            _ => (),
        }
    }

    fn literal(&mut self) {
        match self.prev.id {
            TokenType::False => self.emit(OpCode::False),
            TokenType::True => self.emit(OpCode::True),
            _ => (),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.next();
        let prefix_rule = self.get_rule(self.prev.id).prefix;
        if let Some(func) = prefix_rule {
            func(self);
        } else {
            return self.error("Expected expression");
        }

        while precedence <= self.get_rule(self.curr.id).precedence {
            self.next();
            let infix_rule = self.get_rule(self.prev.id).infix;
            if let Some(func) = infix_rule {
                func(self);
            } else {
                return self.error("Unexpected infix rule call");
            }
        }
    }

    fn get_rule(&self, id: TokenType) -> &ParseRule {
        self.rules.get(&id).unwrap_or(&ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        })
        // .unwrap_or_else(|| panic!("Undefined rule {:?}", id))
    }

    fn error_curr(&mut self, msg: &str) {
        self.error_at(self.curr.clone(), msg);
    }

    fn error(&mut self, msg: &str) {
        self.error_at(self.prev.clone(), msg);
    }

    fn error_at(&mut self, token: Token, msg: &str) {
        if self.panic_mode {
            return;
        }
        self.had_error = true;
        self.panic_mode = true;
        eprint!("[line {}] Error", token.line);
        if token.id == TokenType::Eof {
            eprint!(" at end of file");
        } else {
            eprint!(" at `{}`", token.lexeme);
        }
        eprintln!(": {}", msg);
    }

    fn end_compile(&mut self) {
        self.emit(OpCode::Return);

        // if cfg!(debug_assertions) && !self.had_error {
        //     self.chunk.disassemble("Debug code");
        // }
    }
}

pub fn compile(code: &str) -> Result<Chunk, LangError> {
    let mut compiler = Compiler::new(code);
    let passed = compiler.compile();
    if passed {
        Ok(compiler.chunk)
    } else {
        Err(LangError::CompileError)
    }
}
