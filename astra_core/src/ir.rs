use bls12_381::Scalar;
use ff::PrimeField;

pub fn scalar_from_dec_str(s: &str) -> Result<Scalar, String> {
    if s.starts_with('-') {
        let pos = scalar_from_dec_str(&s[1..])?;
        return Ok(-pos);
    }
    let n = num_bigint::BigUint::parse_bytes(s.as_bytes(), 10)
        .ok_or_else(|| format!("invalid number: {}", s))?;
    let bytes = n.to_bytes_le();
    let mut repr = <Scalar as PrimeField>::Repr::default();
    let len = bytes.len().min(repr.as_ref().len());
    repr.as_mut()[..len].copy_from_slice(&bytes[..len]);
    Option::from(Scalar::from_repr(repr))
        .ok_or_else(|| format!("invalid field element: {}", s))
}

pub fn scalar_display(s: &Scalar) -> String {
    let bytes = s.to_repr();
    let n = num_bigint::BigUint::from_bytes_le(bytes.as_ref());
    n.to_string()
}

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
