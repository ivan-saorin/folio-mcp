//! Mini-expression parser for recurrence relations
//!
//! Supports simple arithmetic expressions with variables a, b, c, d (previous values)
//! and n (current index).

use folio_core::{Number, FolioError};
use std::collections::HashMap;

/// Token types for the expression parser
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(Number),
    Variable(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    LParen,
    RParen,
    Func(String),
}

/// AST node for expressions
#[derive(Debug, Clone)]
pub enum Expr {
    Num(Number),
    Var(String),
    BinOp(Box<Expr>, Op, Box<Expr>),
    UnaryMinus(Box<Expr>),
    FuncCall(String, Box<Expr>),
}

#[derive(Debug, Clone, Copy)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

/// Tokenize expression string
fn tokenize(input: &str) -> Result<Vec<Token>, FolioError> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' | '\n' => {
                chars.next();
            }
            '+' => {
                tokens.push(Token::Plus);
                chars.next();
            }
            '-' => {
                tokens.push(Token::Minus);
                chars.next();
            }
            '*' => {
                tokens.push(Token::Star);
                chars.next();
            }
            '/' => {
                tokens.push(Token::Slash);
                chars.next();
            }
            '^' => {
                tokens.push(Token::Caret);
                chars.next();
            }
            '(' => {
                tokens.push(Token::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RParen);
                chars.next();
            }
            '0'..='9' | '.' => {
                let mut num_str = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() || c == '.' {
                        num_str.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                match Number::from_str(&num_str) {
                    Ok(n) => tokens.push(Token::Number(n)),
                    Err(_) => return Err(FolioError::new("PARSE_ERROR", format!("Invalid number: {}", num_str))),
                }
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        ident.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let lower = ident.to_lowercase();
                // Check if it's a function
                if matches!(lower.as_str(), "abs" | "sqrt" | "floor" | "ceil") {
                    tokens.push(Token::Func(lower));
                } else {
                    tokens.push(Token::Variable(lower));
                }
            }
            _ => {
                return Err(FolioError::new("PARSE_ERROR", format!("Unexpected character: {}", ch)));
            }
        }
    }

    Ok(tokens)
}

/// Parse tokens into AST
struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.pos);
        self.pos += 1;
        token
    }

    fn parse(&mut self) -> Result<Expr, FolioError> {
        self.parse_expr()
    }

    // expr = term (('+' | '-') term)*
    fn parse_expr(&mut self) -> Result<Expr, FolioError> {
        let mut left = self.parse_term()?;

        loop {
            match self.peek() {
                Some(Token::Plus) => {
                    self.advance();
                    let right = self.parse_term()?;
                    left = Expr::BinOp(Box::new(left), Op::Add, Box::new(right));
                }
                Some(Token::Minus) => {
                    self.advance();
                    let right = self.parse_term()?;
                    left = Expr::BinOp(Box::new(left), Op::Sub, Box::new(right));
                }
                _ => break,
            }
        }

        Ok(left)
    }

    // term = power (('*' | '/') power)*
    fn parse_term(&mut self) -> Result<Expr, FolioError> {
        let mut left = self.parse_power()?;

        loop {
            match self.peek() {
                Some(Token::Star) => {
                    self.advance();
                    let right = self.parse_power()?;
                    left = Expr::BinOp(Box::new(left), Op::Mul, Box::new(right));
                }
                Some(Token::Slash) => {
                    self.advance();
                    let right = self.parse_power()?;
                    left = Expr::BinOp(Box::new(left), Op::Div, Box::new(right));
                }
                _ => break,
            }
        }

        Ok(left)
    }

    // power = unary ('^' power)?  (right associative)
    fn parse_power(&mut self) -> Result<Expr, FolioError> {
        let left = self.parse_unary()?;

        if matches!(self.peek(), Some(Token::Caret)) {
            self.advance();
            let right = self.parse_power()?;  // Right associative
            return Ok(Expr::BinOp(Box::new(left), Op::Pow, Box::new(right)));
        }

        Ok(left)
    }

    // unary = '-' unary | primary
    fn parse_unary(&mut self) -> Result<Expr, FolioError> {
        if matches!(self.peek(), Some(Token::Minus)) {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expr::UnaryMinus(Box::new(expr)));
        }
        self.parse_primary()
    }

    // primary = number | variable | func '(' expr ')' | '(' expr ')'
    fn parse_primary(&mut self) -> Result<Expr, FolioError> {
        match self.peek().cloned() {
            Some(Token::Number(n)) => {
                self.advance();
                Ok(Expr::Num(n))
            }
            Some(Token::Variable(name)) => {
                self.advance();
                Ok(Expr::Var(name))
            }
            Some(Token::Func(name)) => {
                self.advance();
                if !matches!(self.peek(), Some(Token::LParen)) {
                    return Err(FolioError::new("PARSE_ERROR", format!("Expected '(' after function {}", name)));
                }
                self.advance(); // consume '('
                let arg = self.parse_expr()?;
                if !matches!(self.peek(), Some(Token::RParen)) {
                    return Err(FolioError::new("PARSE_ERROR", "Expected ')' after function argument".to_string()));
                }
                self.advance(); // consume ')'
                Ok(Expr::FuncCall(name, Box::new(arg)))
            }
            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_expr()?;
                if !matches!(self.peek(), Some(Token::RParen)) {
                    return Err(FolioError::new("PARSE_ERROR", "Expected closing ')'".to_string()));
                }
                self.advance();
                Ok(expr)
            }
            Some(token) => {
                Err(FolioError::new("PARSE_ERROR", format!("Unexpected token: {:?}", token)))
            }
            None => {
                Err(FolioError::new("PARSE_ERROR", "Unexpected end of expression".to_string()))
            }
        }
    }
}

/// Parse an expression string into an AST
pub fn parse_expr(input: &str) -> Result<Expr, FolioError> {
    let tokens = tokenize(input)?;
    if tokens.is_empty() {
        return Err(FolioError::new("PARSE_ERROR", "Empty expression".to_string()));
    }
    let mut parser = Parser::new(tokens);
    let expr = parser.parse()?;

    // Check that all tokens were consumed
    if parser.pos < parser.tokens.len() {
        return Err(FolioError::new("PARSE_ERROR", "Unexpected tokens at end of expression".to_string()));
    }

    Ok(expr)
}

/// Evaluate an expression with the given variable context
pub fn eval_expr(expr: &Expr, ctx: &HashMap<String, Number>, precision: u32) -> Result<Number, FolioError> {
    match expr {
        Expr::Num(n) => Ok(n.clone()),
        Expr::Var(name) => {
            ctx.get(name)
                .cloned()
                .ok_or_else(|| FolioError::new("UNDEFINED_VAR", format!("Unknown variable: {}", name)))
        }
        Expr::BinOp(left, op, right) => {
            let l = eval_expr(left, ctx, precision)?;
            let r = eval_expr(right, ctx, precision)?;
            match op {
                Op::Add => Ok(l.add(&r)),
                Op::Sub => Ok(l.sub(&r)),
                Op::Mul => Ok(l.mul(&r)),
                Op::Div => l.checked_div(&r).map_err(|e| e.into()),
                Op::Pow => {
                    // Check if exponent is an integer
                    if r.is_integer() {
                        if let Some(exp) = r.to_i64() {
                            if exp >= i32::MIN as i64 && exp <= i32::MAX as i64 {
                                return Ok(l.pow(exp as i32));
                            }
                        }
                    }
                    Ok(l.pow_real(&r, precision))
                }
            }
        }
        Expr::UnaryMinus(inner) => {
            let val = eval_expr(inner, ctx, precision)?;
            Ok(Number::from_i64(0).sub(&val))
        }
        Expr::FuncCall(name, arg) => {
            let val = eval_expr(arg, ctx, precision)?;
            match name.as_str() {
                "abs" => Ok(val.abs()),
                "sqrt" => val.sqrt(precision).map_err(|e| e.into()),
                "floor" => Ok(val.floor()),
                "ceil" => Ok(val.ceil()),
                _ => Err(FolioError::new("UNDEFINED_FUNC", format!("Unknown function: {}", name))),
            }
        }
    }
}

/// Evaluate a recurrence expression with previous values and current index
pub fn eval_recurrence_expr(
    expr_str: &str,
    prev: &[Number],
    n: i64,
    precision: u32,
) -> Result<Number, FolioError> {
    let expr = parse_expr(expr_str)?;

    let mut ctx = HashMap::new();

    // a = most recent (n-1)
    if let Some(val) = prev.last() {
        ctx.insert("a".to_string(), val.clone());
    }
    // b = second most recent (n-2)
    if prev.len() >= 2 {
        ctx.insert("b".to_string(), prev[prev.len() - 2].clone());
    }
    // c = third most recent (n-3)
    if prev.len() >= 3 {
        ctx.insert("c".to_string(), prev[prev.len() - 3].clone());
    }
    // d = fourth most recent (n-4)
    if prev.len() >= 4 {
        ctx.insert("d".to_string(), prev[prev.len() - 4].clone());
    }
    // n = current index (1-based)
    ctx.insert("n".to_string(), Number::from_i64(n));

    eval_expr(&expr, &ctx, precision)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let tokens = tokenize("a + b").unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::Variable(_)));
        assert!(matches!(tokens[1], Token::Plus));
        assert!(matches!(tokens[2], Token::Variable(_)));
    }

    #[test]
    fn test_parse_simple() {
        let expr = parse_expr("a + b").unwrap();
        assert!(matches!(expr, Expr::BinOp(_, Op::Add, _)));
    }

    #[test]
    fn test_eval_addition() {
        let expr = parse_expr("a + b").unwrap();
        let mut ctx = HashMap::new();
        ctx.insert("a".to_string(), Number::from_i64(3));
        ctx.insert("b".to_string(), Number::from_i64(5));
        let result = eval_expr(&expr, &ctx, 50).unwrap();
        assert_eq!(result.to_i64(), Some(8));
    }

    #[test]
    fn test_eval_multiplication() {
        let expr = parse_expr("2 * a").unwrap();
        let mut ctx = HashMap::new();
        ctx.insert("a".to_string(), Number::from_i64(7));
        let result = eval_expr(&expr, &ctx, 50).unwrap();
        assert_eq!(result.to_i64(), Some(14));
    }

    #[test]
    fn test_eval_power() {
        let expr = parse_expr("a ^ 2").unwrap();
        let mut ctx = HashMap::new();
        ctx.insert("a".to_string(), Number::from_i64(3));
        let result = eval_expr(&expr, &ctx, 50).unwrap();
        assert_eq!(result.to_i64(), Some(9));
    }

    #[test]
    fn test_eval_complex() {
        let expr = parse_expr("2*a + b").unwrap();
        let mut ctx = HashMap::new();
        ctx.insert("a".to_string(), Number::from_i64(5));
        ctx.insert("b".to_string(), Number::from_i64(3));
        let result = eval_expr(&expr, &ctx, 50).unwrap();
        assert_eq!(result.to_i64(), Some(13));
    }

    #[test]
    fn test_eval_recurrence() {
        // Fibonacci: a + b
        let prev = vec![Number::from_i64(5), Number::from_i64(8)];
        let result = eval_recurrence_expr("a + b", &prev, 7, 50).unwrap();
        assert_eq!(result.to_i64(), Some(13));
    }

    #[test]
    fn test_eval_factorial_recurrence() {
        // Factorial: a * n
        let prev = vec![Number::from_i64(24)]; // 4!
        let result = eval_recurrence_expr("a * n", &prev, 5, 50).unwrap();
        assert_eq!(result.to_i64(), Some(120)); // 5!
    }
}
