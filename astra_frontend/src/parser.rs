use astra_ir::ir::*;
use bls12_381::Scalar;

use crate::error::ParseError;

pub fn parse(input: &str) -> Result<Program, ParseError> {
    let mut p = Parser {
        chars: input.chars().collect(),
        pos: 0,
    };
    p.parse_program()
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == '/' && self.chars.get(self.pos + 1) == Some(&'/') {
                self.pos += 2;
                while let Some(c) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    self.pos += 1;
                }
            } else if c == '/' && self.chars.get(self.pos + 1) == Some(&'*') {
                self.pos += 2;
                while let Some(c) = self.peek() {
                    if c == '*' && self.chars.get(self.pos + 1) == Some(&'/') {
                        self.pos += 2;
                        break;
                    }
                    self.pos += 1;
                }
            } else if c.is_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn expect(&mut self, ch: char) -> Result<(), ParseError> {
        self.skip_whitespace();
        if self.peek() != Some(ch) {
            return Err(ParseError::at(format!("expected '{}'", ch), self.pos));
        }
        self.pos += 1;
        Ok(())
    }

    fn parse_ident(&mut self) -> Result<String, ParseError> {
        self.skip_whitespace();
        let start = self.pos;
        if let Some(c) = self.peek() {
            if !c.is_ascii_alphabetic() && c != '_' {
                return Err(ParseError::at(
                    format!("expected identifier at pos {}", self.pos),
                    self.pos,
                ));
            }
            self.pos += 1;
        }
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.pos += 1;
            } else {
                break;
            }
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        Ok(s)
    }

    fn parse_number(&mut self) -> Result<Scalar, ParseError> {
        self.skip_whitespace();
        let start = self.pos;
        if self.peek() == Some('-') {
            self.pos += 1;
        }
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.pos += 1;
            } else {
                break;
            }
        }
        if start == self.pos {
            return Err(ParseError::at(
                format!("expected number at pos {}", self.pos),
                self.pos,
            ));
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        astra_ir::types::scalar_from_dec_str(&s).map_err(|e| ParseError::at(e, start))
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_add_expr()
    }

    fn parse_add_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_mul_expr()?;
        loop {
            self.skip_whitespace();
            match self.peek() {
                Some('+') => {
                    self.pos += 1;
                    let right = self.parse_mul_expr()?;
                    left = Expr::Binary(BinaryOp::Add, Box::new(left), Box::new(right));
                }
                Some('-') => {
                    self.pos += 1;
                    let right = self.parse_mul_expr()?;
                    left = Expr::Binary(BinaryOp::Sub, Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_mul_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_primary_expr()?;
        loop {
            self.skip_whitespace();
            match self.peek() {
                Some('*') => {
                    self.pos += 1;
                    let right = self.parse_primary_expr()?;
                    left = Expr::Binary(BinaryOp::Mul, Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_primary_expr(&mut self) -> Result<Expr, ParseError> {
        self.skip_whitespace();
        match self.peek() {
            Some('(') => {
                self.pos += 1;
                let expr = self.parse_expr()?;
                self.expect(')')?;
                Ok(expr)
            }
            Some(c) if c.is_ascii_digit() => {
                let n = self.parse_number()?;
                Ok(Expr::Number(n))
            }
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                let name = self.parse_ident()?;
                Ok(Expr::Variable(name))
            }
            Some(c) => Err(ParseError::at(
                format!("unexpected token '{}'", c),
                self.pos,
            )),
            None => Err(ParseError::at("unexpected end of input", self.pos)),
        }
    }

    fn parse_def(&mut self) -> Result<Func, ParseError> {
        self.skip_whitespace();
        let name = self.parse_ident()?;
        self.expect('(')?;
        let mut params = Vec::new();
        loop {
            self.skip_whitespace();
            if self.peek() == Some(')') {
                break;
            }
            let ty = self.parse_ident()?;
            if ty != "field" {
                return Err(ParseError::at(
                    format!("unsupported type '{}'", ty),
                    self.pos,
                ));
            }
            self.skip_whitespace();
            let is_private = if self.peek() == Some('p') {
                let kw = self.parse_ident()?;
                kw == "private"
            } else {
                false
            };
            let name = self.parse_ident()?;
            params.push((name, is_private));
            self.skip_whitespace();
            if self.peek() == Some(',') {
                self.pos += 1;
            }
        }
        self.expect(')')?;
        self.skip_whitespace();
        if self.peek() == Some('-') {
            self.pos += 2;
            let _ret_ty = self.parse_ident()?;
        }
        self.expect('{')?;
        let body = self.parse_block()?;
        self.expect('}')?;
        Ok(Func { name, params, body })
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();
        loop {
            self.skip_whitespace();
            if self.peek() == Some('}') || self.peek().is_none() {
                break;
            }
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.skip_whitespace();
        match self.peek() {
            Some('i') => {
                let kw = self.parse_ident()?;
                if kw == "if" {
                    let cond = self.parse_expr()?;
                    self.expect('{')?;
                    let then = self.parse_block()?;
                    self.expect('}')?;
                    return Ok(Stmt::If { cond, body: then });
                }
                Err(ParseError::at(
                    format!("unknown keyword '{}'", kw),
                    self.pos,
                ))
            }
            Some('r') => {
                let kw = self.parse_ident()?;
                if kw == "return" {
                    let val = self.parse_expr()?;
                    self.expect(';')?;
                    return Ok(Stmt::Return(val));
                }
                Err(ParseError::at(
                    format!("unknown keyword '{}'", kw),
                    self.pos,
                ))
            }
            Some('a') => {
                let kw = self.parse_ident()?;
                if kw == "assert" {
                    self.expect('(')?;
                    let left = self.parse_expr()?;
                    self.skip_whitespace();
                    if self.peek() == Some('=') {
                        self.pos += 1;
                        if self.peek() == Some('=') {
                            self.pos += 1;
                        } else {
                            self.expect('=')?;
                        }
                    }
                    let right = self.parse_expr()?;
                    self.expect(')')?;
                    self.expect(';')?;
                    return Ok(Stmt::Constrain { left, right });
                }
                Err(ParseError::at(
                    format!("unknown keyword '{}'", kw),
                    self.pos,
                ))
            }
            Some('f') => {
                let _ty = self.parse_ident()?;
                let name = self.parse_ident()?;
                self.skip_whitespace();
                if self.peek() == Some('=') {
                    self.pos += 1;
                    let init = self.parse_expr()?;
                    self.expect(';')?;
                    Ok(Stmt::Declare {
                        name,
                        init: Some(init),
                    })
                } else if self.peek() == Some(';') {
                    self.pos += 1;
                    Ok(Stmt::Declare { name, init: None })
                } else {
                    Err(ParseError::at(
                        format!("expected '=' or ';' after '{}'", name),
                        self.pos,
                    ))
                }
            }
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                let name = self.parse_ident()?;
                self.skip_whitespace();
                if self.peek() == Some('=') {
                    self.pos += 1;
                    let init = self.parse_expr()?;
                    self.expect(';')?;
                    Ok(Stmt::Declare {
                        name,
                        init: Some(init),
                    })
                } else {
                    Err(ParseError::at(
                        format!("expected '=' after '{}'", name),
                        self.pos,
                    ))
                }
            }
            Some(_) => Err(ParseError::at("unexpected token in statement", self.pos)),
            None => Err(ParseError::at("unexpected end of input", self.pos)),
        }
    }

    fn parse_program(&mut self) -> Result<Program, ParseError> {
        self.skip_whitespace();
        let kw = self.parse_ident()?;
        if kw != "def" {
            return Err(ParseError::at("expected 'def'", self.pos));
        }
        let main = self.parse_def()?;
        Ok(Program { main })
    }
}
