//! Abstract syntax tree for the Zara circuit language.

#[derive(Debug, Clone)]
pub struct Program {
    pub main: Func,
}

#[derive(Debug, Clone)]
pub struct Func {
    pub name: String,
    pub params: Vec<(String, bool)>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Declare { name: String, init: Option<Expr> },
    Constrain { left: Expr, right: Expr },
    If { cond: Expr, body: Vec<Stmt> },
    Return(Expr),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(bls12_381::Scalar),
    Variable(String),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
}
