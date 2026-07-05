use bls12_381::Scalar;

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
    Number(u64),
    Variable(String),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
}

#[derive(Debug, Clone)]
pub struct ConstraintSystem {
    pub num_public: usize,
    pub num_private: usize,
    pub num_variables: usize,
    pub num_constraints: usize,
    pub a: Vec<Vec<(usize, Scalar)>>,
    pub b: Vec<Vec<(usize, Scalar)>>,
    pub c: Vec<Vec<(usize, Scalar)>>,
    pub witness: Vec<Scalar>,
}

#[derive(Debug, Clone)]
pub struct R1cs {
    pub a: Vec<Vec<(usize, Scalar)>>,
    pub b: Vec<Vec<(usize, Scalar)>>,
    pub c: Vec<Vec<(usize, Scalar)>>,
}
