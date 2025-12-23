//! Abstract Syntax Tree

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Document {
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub name: String,
    pub attributes: HashMap<String, String>,
    pub table: Table,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Table {
    pub columns: Vec<String>,
    pub rows: Vec<Row>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    pub cells: Vec<Cell>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub name: String,
    pub formula: Option<Expr>,
    pub raw_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    Number(String),
    StringLiteral(String),
    Variable(Vec<String>),
    BinaryOp(Box<Expr>, BinOp, Box<Expr>),
    UnaryOp(UnaryOp, Box<Expr>),
    FunctionCall(String, Vec<Expr>),
    /// List literal: [a, b, c]
    List(Vec<Expr>),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BinOp { Add, Sub, Mul, Div, Pow }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum UnaryOp { Neg }
