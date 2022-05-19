use crate::{
    chunk::{Chunk, OpCode, Value},
    lexer::Lexer,
    result::LangError,
    token::{Token, TokenType},
};
use std::collections::HashMap;

const JUMP_PLACEHOLDER: usize = usize::MAX;

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

type ParseFn = Option<fn(&mut Compiler, can_assign: bool) -> ()>;

struct ParseRule {
    prefix: ParseFn,
    infix: ParseFn,
    precedence: Precedence,
}

struct Local {
    name: Token,
    depth: Option<usize>,
}

struct Compiler {
    chunk: Chunk,
    lexer: Lexer,
    curr: Token,
    prev: Token,
    had_error: bool,
    panic_mode: bool,
    rules: HashMap<TokenType, ParseRule>,
    locals: Vec<Local>,
    scope_depth: usize,
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
            (Bool, rule(Some(Self::bool), None, P::None)),
            (Int, rule(Some(Self::int), None, P::None)),
            (Float, rule(Some(Self::float), None, P::None)),
            (Str, rule(Some(Self::string), None, P::None)),
            (Not, rule(Some(Self::unary), None, P::None)),
            (And, rule(None, Some(Self::and_op), P::And)),
            (Or, rule(None, Some(Self::or_op), P::Or)),
            (BangEqual, rule(None, Some(Self::binary), P::Equality)),
            (EqualEqual, rule(None, Some(Self::binary), P::Equality)),
            (Greater, rule(None, Some(Self::binary), P::Comparison)),
            (GreaterEqual, rule(None, Some(Self::binary), P::Comparison)),
            (Less, rule(None, Some(Self::binary), P::Comparison)),
            (LessEqual, rule(None, Some(Self::binary), P::Comparison)),
            (Identifier, rule(Some(Self::variable), None, P::None)),
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
            locals: Vec::new(),
            scope_depth: 0,
        }
    }

    fn compile(&mut self) -> bool {
        self.next();
        while !self.matches(TokenType::Eof) {
            self.declaration();
        }
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

    fn eat_delimit(&mut self) {
        while self.curr.id == TokenType::Semicolon || self.curr.id == TokenType::Newline {
            self.next();
        }
    }

    fn emit(&mut self, op: OpCode) {
        self.chunk.write(op, self.prev.line);
    }

    fn emit_with_index(&mut self, op: OpCode) -> usize {
        self.chunk.write(op, self.prev.line);
        self.chunk.code.len() - 1
    }

    fn emit_two(&mut self, op1: OpCode, op2: OpCode) {
        self.chunk.write(op1, self.prev.line);
        self.chunk.write(op2, self.prev.line);
    }

    fn emit_constant(&mut self, value: Value) {
        let index = self.chunk.add_constant(value);
        self.emit(index);
    }

    fn patch_jump(&mut self, index: usize) {
        let jump = self.chunk.code.len() - index - 1;
        match self.chunk.code[index] {
            OpCode::Jump(ref mut x) => *x = jump,
            OpCode::JumpIfFalse(ref mut x) => *x = jump,
            _ => unreachable!(),
        }
    }

    fn declaration(&mut self) {
        if self.matches(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn var_declaration(&mut self) {
        let index = self.parse_variable("Expected a variable name");

        if self.matches(TokenType::Equal) {
            self.expression();
        } else {
            self.error_curr("Expected an expression");
        }
        self.eat_delimit();
        self.define_variable(index);
    }

    fn statement(&mut self) {
        if self.matches(TokenType::Print) {
            self.print_statement();
        } else if self.matches(TokenType::If) {
            self.if_statement();
        } else if self.matches(TokenType::While) {
            self.while_statement();
        } else if self.matches(TokenType::Do) {
            self.eat_delimit();
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
        self.eat_delimit();
    }

    fn print_statement(&mut self) {
        self.expression();
        self.emit(OpCode::Print);
    }

    fn if_statement(&mut self) {
        self.expression();
        self.eat_delimit();

        let then_index = self.emit_with_index(OpCode::JumpIfFalse(JUMP_PLACEHOLDER));
        self.emit(OpCode::Pop);

        self.if_block();

        let else_index = self.emit_with_index(OpCode::Jump(JUMP_PLACEHOLDER));
        self.patch_jump(then_index);
        self.emit(OpCode::Pop);

        if self.matches(TokenType::Else) {
            self.if_block();
        }
        self.patch_jump(else_index);
    }

    fn while_statement(&mut self) {
        let start = self.chunk.code.len() - 1;

        self.expression();
        self.eat_delimit();

        let exit_index = self.emit_with_index(OpCode::JumpIfFalse(JUMP_PLACEHOLDER));
        self.emit(OpCode::Pop);

        self.begin_scope();
        self.block();
        self.end_scope();

        let back_index = self.chunk.code.len() - start;
        self.emit(OpCode::JumpBack(back_index));

        self.patch_jump(exit_index);
        self.emit(OpCode::Pop);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.emit(OpCode::Pop);
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn block(&mut self) {
        while !self.check(TokenType::End) && !self.check(TokenType::Eof) {
            self.declaration();
        }
        self.eat(TokenType::End, "Expected 'end' after block");
    }

    fn if_block(&mut self) {
        self.eat_delimit();
        self.begin_scope();

        while !self.check(TokenType::Else)
            && !self.check(TokenType::End)
            && !self.check(TokenType::Eof)
        {
            self.declaration();
        }
        if self.curr.id != TokenType::Else {
            self.eat(TokenType::End, "Expected 'end' after if block'");
        }

        self.end_scope();
    }

    fn bool(&mut self, _can_assign: bool) {
        let value = self.prev.lexeme.parse::<bool>().unwrap();
        self.emit_constant(Value::Bool(value));
    }

    fn int(&mut self, _can_assign: bool) {
        let value = match self.prev.lexeme.parse::<isize>() {
            Ok(v) => v,
            Err(_) => {
                return self.error(&format!(
                    "Integer is out of the range {}..{}",
                    isize::MIN + 1,
                    isize::MAX
                ))
            }
        };

        self.emit_constant(Value::Int(value));
    }

    fn float(&mut self, _can_assign: bool) {
        let value = self.prev.lexeme.parse::<f64>().unwrap();
        self.emit_constant(Value::Float(value));
    }

    fn string(&mut self, _can_assign: bool) {
        let lexeme = self.prev.lexeme.clone();
        self.emit_constant(Value::Str(lexeme[1..lexeme.len() - 1].to_string()));
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(self.prev.clone(), can_assign);
    }

    fn named_variable(&mut self, name: Token, can_assign: bool) {
        let get_op;
        let set_op;

        if let Some(index) = self.resolve_local(&name) {
            get_op = OpCode::GetLocal(index);
            set_op = OpCode::SetLocal(index);
        } else {
            let index = self.identifier_constant(name);
            get_op = OpCode::GetGlobal(index);
            set_op = OpCode::SetGlobal(index);
        }

        if can_assign && self.matches(TokenType::Equal) {
            self.expression();
            self.emit(set_op);
        } else {
            self.emit(get_op);
        }
    }

    fn define_variable(&mut self, index: usize) {
        if self.scope_depth > 0 {
            self.mark_initialized();
            return;
        }
        self.emit(OpCode::DefineGlobal(index));
    }

    fn group(&mut self, _can_assign: bool) {
        self.expression();
        self.eat(TokenType::RightParen, "Expected closing parenthesis ')'");
    }

    fn unary(&mut self, _can_assign: bool) {
        let operator_id = self.prev.id;

        self.parse_precedence(Precedence::Unary);

        match operator_id {
            TokenType::Minus => self.emit(OpCode::Negate),
            TokenType::Not => self.emit(OpCode::Not),
            _ => (),
        }
    }

    fn binary(&mut self, _can_assign: bool) {
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

    fn and_op(&mut self, _can_assign: bool) {
        let index = self.emit_with_index(OpCode::JumpIfFalse(JUMP_PLACEHOLDER));

        self.emit(OpCode::Pop);
        self.parse_precedence(Precedence::And);

        self.patch_jump(index);
    }

    fn or_op(&mut self, _can_assign: bool) {
        let else_index = self.emit_with_index(OpCode::JumpIfFalse(JUMP_PLACEHOLDER));
        let end_index = self.emit_with_index(OpCode::Jump(JUMP_PLACEHOLDER));

        self.patch_jump(else_index);
        self.emit(OpCode::Pop);

        self.parse_precedence(Precedence::Or);
        self.patch_jump(end_index);
    }

    fn _literal(&mut self) {
        // match self.prev.id {
        //     _ => (),
        // }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.next();
        let prefix_rule = self.get_rule(self.prev.id).prefix;

        let prefix_rule = match prefix_rule {
            Some(rule) => rule,
            None => return self.error("Expected expression"),
        };

        let can_assign = precedence <= Precedence::Assignment;
        prefix_rule(self, can_assign);

        while precedence <= self.get_rule(self.curr.id).precedence {
            self.next();
            let infix_rule = self.get_rule(self.prev.id).infix.unwrap();
            infix_rule(self, can_assign);

            if can_assign && self.matches(TokenType::Equal) {
                self.error("Invalid assignment target");
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

    fn parse_variable(&mut self, message: &str) -> usize {
        self.eat(TokenType::Identifier, message);
        self.declare_variable();
        if self.scope_depth > 0 {
            return 0;
        }
        self.identifier_constant(self.prev.clone())
    }

    fn mark_initialized(&mut self) {
        self.locals.last_mut().unwrap().depth = Some(self.scope_depth);
    }

    fn identifier_constant(&mut self, token: Token) -> usize {
        let name = Value::Str(token.lexeme);
        self.chunk.add_constant(name);
        self.chunk.constants.len() - 1
    }

    fn declare_variable(&mut self) {
        if self.scope_depth == 0 {
            return;
        }

        let name = self.prev.clone();
        if self.search_locals(&name) {
            self.error("Cannot redeclare variable in this scope");
        }
        self.add_local(name);
    }

    fn search_locals(&self, name: &Token) -> bool {
        for local in self.locals.iter().rev() {
            if local.depth.is_some() && local.depth.unwrap() < self.scope_depth {
                return false;
            }
            if local.name.lexeme == name.lexeme {
                return true;
            }
        }
        false
    }

    fn resolve_local(&mut self, name: &Token) -> Option<usize> {
        for (index, local) in self.locals.iter().enumerate().rev() {
            if name.lexeme == local.name.lexeme {
                if local.depth.is_none() {
                    self.error("Cannot read local variable in its own initializer");
                }
                return Some(index);
            }
        }
        None
    }

    fn add_local(&mut self, name: Token) {
        self.locals.push(Local { name, depth: None });
    }

    fn matches(&mut self, id: TokenType) -> bool {
        if !self.check(id) {
            false
        } else {
            self.next();
            true
        }
    }

    fn check(&self, id: TokenType) -> bool {
        self.curr.id == id
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

    fn synchronize(&mut self) {
        use TokenType::*;
        self.panic_mode = false;

        while self.curr.id != Eof {
            if self.prev.id == Semicolon || self.prev.id == Newline {
                return;
            }

            match self.curr.id {
                Class | Fn | Var | For | If | While | Print | Return => return,
                _ => (),
            }
            self.next();
        }
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;

        for i in (0..self.locals.len()).rev() {
            if self.locals[i].depth.unwrap() > self.scope_depth {
                self.emit(OpCode::Pop);
                self.locals.pop();
            }
        }
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
