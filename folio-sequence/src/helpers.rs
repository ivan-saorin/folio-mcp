//! Helper functions and utility sequence operations
//!
//! Common utilities for extracting inputs and utility sequence functions.

use folio_plugin::prelude::*;
use folio_core::{Number, Value, FolioError};

/// Extract a list from a single argument
pub fn extract_list(arg: &Value) -> Result<Vec<Number>, FolioError> {
    match arg {
        Value::List(list) => {
            let mut numbers = Vec::new();
            for item in list {
                match item {
                    Value::Number(n) => numbers.push(n.clone()),
                    Value::Error(e) => return Err(e.clone()),
                    other => return Err(FolioError::type_error("Number", other.type_name())),
                }
            }
            Ok(numbers)
        }
        Value::Error(e) => Err(e.clone()),
        other => Err(FolioError::type_error("List", other.type_name())),
    }
}

/// Extract a single number from argument
pub fn extract_number(arg: &Value, name: &str) -> Result<Number, FolioError> {
    match arg {
        Value::Number(n) => Ok(n.clone()),
        Value::Error(e) => Err(e.clone()),
        other => Err(FolioError::arg_type("", name, "Number", other.type_name())),
    }
}

/// Extract an optional number argument
pub fn extract_optional_number(args: &[Value], index: usize) -> Result<Option<Number>, FolioError> {
    if index >= args.len() {
        return Ok(None);
    }
    match &args[index] {
        Value::Number(n) => Ok(Some(n.clone())),
        Value::Null => Ok(None),
        Value::Error(e) => Err(e.clone()),
        other => Err(FolioError::type_error("Number", other.type_name())),
    }
}

/// Require a positive integer count parameter
pub fn require_count(n: &Number, func: &str, max: usize) -> Result<usize, FolioError> {
    if !n.is_integer() || n.is_negative() || n.is_zero() {
        return Err(FolioError::domain_error(format!(
            "{}() requires positive integer count",
            func
        )));
    }
    let count = n.to_i64().unwrap_or(0) as usize;
    if count > max {
        return Err(FolioError::domain_error(format!(
            "{}() limited to {} elements for performance",
            func, max
        )));
    }
    Ok(count)
}

/// Require a non-negative integer index parameter
pub fn require_index(n: &Number, func: &str) -> Result<usize, FolioError> {
    if !n.is_integer() || n.is_negative() {
        return Err(FolioError::domain_error(format!(
            "{}() requires non-negative integer index",
            func
        )));
    }
    Ok(n.to_i64().unwrap_or(0) as usize)
}

/// Calculate sum of numbers
pub fn sum(numbers: &[Number]) -> Number {
    numbers
        .iter()
        .fold(Number::from_i64(0), |acc, n| acc.add(n))
}

/// Calculate product of numbers
pub fn product(numbers: &[Number]) -> Number {
    numbers
        .iter()
        .fold(Number::from_i64(1), |acc, n| acc.mul(n))
}

// ============ Utility Functions ============

/// Get nth element of named sequence (more efficient than generating full sequence)
pub struct Nth;

static NTH_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "sequence_name",
        typ: "Text",
        description: "Name of sequence: fibonacci, prime, lucas, etc.",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "n",
        typ: "Number",
        description: "Index (1-based)",
        optional: false,
        default: None,
    },
];

static NTH_EXAMPLES: [&str; 2] = [
    "nth(\"fibonacci\", 10) → 55",
    "nth(\"prime\", 5) → 11",
];

static NTH_RELATED: [&str; 2] = ["fibonacci", "primes"];

impl FunctionPlugin for Nth {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "nth",
            description: "Get nth element of named sequence (more efficient than generating full sequence)",
            usage: "nth(sequence_name, n)",
            args: &NTH_ARGS,
            returns: "Number",
            examples: &NTH_EXAMPLES,
            category: "sequence/utility",
            source: None,
            related: &NTH_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("nth", 2, args.len()));
        }

        let name = match &args[0] {
            Value::Text(s) => s.to_lowercase(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("nth", "sequence_name", "Text", other.type_name())),
        };

        let n = match extract_number(&args[1], "n") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if !n.is_integer() || n.is_negative() {
            return Value::Error(FolioError::domain_error(
                "nth() requires non-negative integer index"
            ));
        }

        let idx = n.to_i64().unwrap_or(0) as usize;

        match name.as_str() {
            "fibonacci" => nth_fibonacci(idx),
            "prime" | "primes" => nth_prime(idx),
            "lucas" => nth_lucas(idx),
            "triangular" => nth_triangular(idx),
            "square" => nth_square(idx),
            "cube" => nth_cube(idx),
            "factorial" => nth_factorial(idx, ctx),
            _ => Value::Error(FolioError::domain_error(format!(
                "Unknown sequence name: {}. Valid: fibonacci, prime, lucas, triangular, square, cube, factorial",
                name
            ))),
        }
    }
}

fn nth_fibonacci(n: usize) -> Value {
    if n == 0 {
        return Value::Number(Number::from_i64(0));
    }
    let (mut a, mut b) = (Number::from_i64(0), Number::from_i64(1));
    for _ in 1..n {
        let next = a.add(&b);
        a = b;
        b = next;
    }
    Value::Number(b)
}

fn nth_lucas(n: usize) -> Value {
    if n == 0 {
        return Value::Number(Number::from_i64(2));
    }
    let (mut a, mut b) = (Number::from_i64(2), Number::from_i64(1));
    for _ in 1..n {
        let next = a.add(&b);
        a = b;
        b = next;
    }
    Value::Number(b)
}

fn nth_prime(n: usize) -> Value {
    if n == 0 {
        return Value::Error(FolioError::domain_error("Prime index must be >= 1"));
    }
    if n > 10000 {
        return Value::Error(FolioError::domain_error("nth(prime) limited to n <= 10000"));
    }

    let mut count = 0;
    let mut candidate = 2u64;

    while count < n {
        if is_prime(candidate) {
            count += 1;
            if count == n {
                return Value::Number(Number::from_i64(candidate as i64));
            }
        }
        candidate += 1;
    }

    Value::Number(Number::from_i64(candidate as i64))
}

fn nth_triangular(n: usize) -> Value {
    // T(n) = n(n+1)/2
    let n = Number::from_i64(n as i64);
    let result = n.mul(&n.add(&Number::from_i64(1)));
    match result.checked_div(&Number::from_i64(2)) {
        Ok(r) => Value::Number(r),
        Err(e) => Value::Error(e.into()),
    }
}

fn nth_square(n: usize) -> Value {
    let n = Number::from_i64(n as i64);
    Value::Number(n.mul(&n))
}

fn nth_cube(n: usize) -> Value {
    let n = Number::from_i64(n as i64);
    Value::Number(n.mul(&n).mul(&n))
}

fn nth_factorial(n: usize, _ctx: &EvalContext) -> Value {
    if n > 170 {
        return Value::Error(FolioError::domain_error("nth(factorial) limited to n <= 170"));
    }
    let mut result = Number::from_i64(1);
    for i in 2..=n {
        result = result.mul(&Number::from_i64(i as i64));
    }
    Value::Number(result)
}

/// Check if a number is prime
pub fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }
    let sqrt_n = (n as f64).sqrt() as u64;
    for i in (3..=sqrt_n).step_by(2) {
        if n % i == 0 {
            return false;
        }
    }
    true
}

// ============ IndexOfSeq ============

pub struct IndexOfSeq;

static INDEX_OF_SEQ_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List<Number>",
        description: "List to search in",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "value",
        typ: "Number",
        description: "Value to find",
        optional: false,
        default: None,
    },
];

static INDEX_OF_SEQ_EXAMPLES: [&str; 1] = [
    "index_of_seq(fibonacci(20), 55) → 10",
];

static INDEX_OF_SEQ_RELATED: [&str; 1] = ["is_in_sequence"];

impl FunctionPlugin for IndexOfSeq {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "index_of_seq",
            description: "Find index of value in sequence (returns -1 if not found)",
            usage: "index_of_seq(list, value)",
            args: &INDEX_OF_SEQ_ARGS,
            returns: "Number",
            examples: &INDEX_OF_SEQ_EXAMPLES,
            category: "sequence/utility",
            source: None,
            related: &INDEX_OF_SEQ_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("index_of_seq", 2, args.len()));
        }

        let list = match extract_list(&args[0]) {
            Ok(l) => l,
            Err(e) => return Value::Error(e),
        };

        let value = match extract_number(&args[1], "value") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        for (i, item) in list.iter().enumerate() {
            if item == &value {
                return Value::Number(Number::from_i64((i + 1) as i64)); // 1-indexed
            }
        }

        Value::Number(Number::from_i64(-1))
    }
}

// ============ IsInSequence ============

pub struct IsInSequence;

static IS_IN_SEQUENCE_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "value",
        typ: "Number",
        description: "Value to check",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "sequence_name",
        typ: "Text",
        description: "Name of sequence: fibonacci, prime, etc.",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "max_check",
        typ: "Number",
        description: "Maximum index to check",
        optional: true,
        default: Some("1000"),
    },
];

static IS_IN_SEQUENCE_EXAMPLES: [&str; 3] = [
    "is_in_sequence(55, \"fibonacci\") → true",
    "is_in_sequence(17, \"prime\") → true",
    "is_in_sequence(15, \"prime\") → false",
];

static IS_IN_SEQUENCE_RELATED: [&str; 2] = ["nth", "index_of_seq"];

impl FunctionPlugin for IsInSequence {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "is_in_sequence",
            description: "Check if value is in named sequence",
            usage: "is_in_sequence(value, sequence_name, [max_check])",
            args: &IS_IN_SEQUENCE_ARGS,
            returns: "Bool",
            examples: &IS_IN_SEQUENCE_EXAMPLES,
            category: "sequence/utility",
            source: None,
            related: &IS_IN_SEQUENCE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 3 {
            return Value::Error(FolioError::arg_count("is_in_sequence", 2, args.len()));
        }

        let value = match extract_number(&args[0], "value") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let name = match &args[1] {
            Value::Text(s) => s.to_lowercase(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("is_in_sequence", "sequence_name", "Text", other.type_name())),
        };

        let max_check = match extract_optional_number(args, 2) {
            Ok(Some(n)) => n.to_i64().unwrap_or(1000) as usize,
            Ok(None) => 1000,
            Err(e) => return Value::Error(e),
        };

        let max_check = max_check.min(10000);

        match name.as_str() {
            "fibonacci" => is_in_fibonacci(&value, max_check),
            "prime" | "primes" => is_in_primes(&value),
            "lucas" => is_in_lucas(&value, max_check),
            "triangular" => is_in_triangular(&value),
            "square" => is_in_square(&value),
            "cube" => is_in_cube(&value),
            _ => Value::Error(FolioError::domain_error(format!(
                "Unknown sequence name: {}",
                name
            ))),
        }
    }
}

fn is_in_fibonacci(value: &Number, max_check: usize) -> Value {
    let (mut a, mut b) = (Number::from_i64(0), Number::from_i64(1));
    for _ in 0..max_check {
        if &a == value {
            return Value::Bool(true);
        }
        if a.sub(value).is_negative() == false && !a.sub(value).is_zero() {
            return Value::Bool(false);
        }
        let next = a.add(&b);
        a = b;
        b = next;
    }
    Value::Bool(false)
}

fn is_in_lucas(value: &Number, max_check: usize) -> Value {
    let (mut a, mut b) = (Number::from_i64(2), Number::from_i64(1));
    for _ in 0..max_check {
        if &a == value {
            return Value::Bool(true);
        }
        if a.sub(value).is_negative() == false && !a.sub(value).is_zero() {
            return Value::Bool(false);
        }
        let next = a.add(&b);
        a = b;
        b = next;
    }
    Value::Bool(false)
}

fn is_in_primes(value: &Number) -> Value {
    if !value.is_integer() || value.is_negative() {
        return Value::Bool(false);
    }
    let n = value.to_i64().unwrap_or(0);
    if n < 2 {
        return Value::Bool(false);
    }
    Value::Bool(is_prime(n as u64))
}

fn is_in_triangular(value: &Number) -> Value {
    // T(n) = n(n+1)/2, so 8*T + 1 should be a perfect square
    if !value.is_integer() || value.is_negative() {
        return Value::Bool(false);
    }
    let eight = Number::from_i64(8);
    let one = Number::from_i64(1);
    let discriminant = eight.mul(value).add(&one);
    // Check if discriminant is a perfect square
    if let Some(d) = discriminant.to_i64() {
        let sqrt_d = (d as f64).sqrt() as i64;
        if sqrt_d * sqrt_d == d && sqrt_d % 2 == 1 {
            return Value::Bool(true);
        }
    }
    Value::Bool(false)
}

fn is_in_square(value: &Number) -> Value {
    if !value.is_integer() || value.is_negative() {
        return Value::Bool(false);
    }
    if let Some(n) = value.to_i64() {
        let sqrt_n = (n as f64).sqrt() as i64;
        return Value::Bool(sqrt_n * sqrt_n == n);
    }
    Value::Bool(false)
}

fn is_in_cube(value: &Number) -> Value {
    if !value.is_integer() {
        return Value::Bool(false);
    }
    if let Some(n) = value.to_i64() {
        let cbrt = if n >= 0 {
            (n as f64).cbrt() as i64
        } else {
            -((-n as f64).cbrt() as i64)
        };
        // Check nearby values due to floating point issues
        for candidate in (cbrt - 1)..=(cbrt + 1) {
            if candidate * candidate * candidate == n {
                return Value::Bool(true);
            }
        }
    }
    Value::Bool(false)
}

// ============ ReverseSeq ============

pub struct ReverseSeq;

static REVERSE_SEQ_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "list",
    typ: "List<Number>",
    description: "List to reverse",
    optional: false,
    default: None,
}];

static REVERSE_SEQ_EXAMPLES: [&str; 1] = [
    "reverse_seq([1, 2, 3, 4, 5]) → [5, 4, 3, 2, 1]",
];

static REVERSE_SEQ_RELATED: [&str; 2] = ["take_seq", "drop_seq"];

impl FunctionPlugin for ReverseSeq {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "reverse_seq",
            description: "Reverse a sequence",
            usage: "reverse_seq(list)",
            args: &REVERSE_SEQ_ARGS,
            returns: "List<Number>",
            examples: &REVERSE_SEQ_EXAMPLES,
            category: "sequence/utility",
            source: None,
            related: &REVERSE_SEQ_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("reverse_seq", 1, args.len()));
        }

        let list = match extract_list(&args[0]) {
            Ok(l) => l,
            Err(e) => return Value::Error(e),
        };

        let reversed: Vec<Value> = list.into_iter().rev().map(Value::Number).collect();
        Value::List(reversed)
    }
}

// ============ Interleave ============

pub struct Interleave;

static INTERLEAVE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list1",
        typ: "List<Number>",
        description: "First list",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "list2",
        typ: "List<Number>",
        description: "Second list",
        optional: false,
        default: None,
    },
];

static INTERLEAVE_EXAMPLES: [&str; 1] = [
    "interleave([1, 3, 5], [2, 4, 6]) → [1, 2, 3, 4, 5, 6]",
];

static INTERLEAVE_RELATED: [&str; 1] = ["zip_seq"];

impl FunctionPlugin for Interleave {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "interleave",
            description: "Interleave two sequences",
            usage: "interleave(list1, list2)",
            args: &INTERLEAVE_ARGS,
            returns: "List<Number>",
            examples: &INTERLEAVE_EXAMPLES,
            category: "sequence/utility",
            source: None,
            related: &INTERLEAVE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("interleave", 2, args.len()));
        }

        let list1 = match extract_list(&args[0]) {
            Ok(l) => l,
            Err(e) => return Value::Error(e),
        };

        let list2 = match extract_list(&args[1]) {
            Ok(l) => l,
            Err(e) => return Value::Error(e),
        };

        let mut result = Vec::new();
        let max_len = list1.len().max(list2.len());

        for i in 0..max_len {
            if i < list1.len() {
                result.push(Value::Number(list1[i].clone()));
            }
            if i < list2.len() {
                result.push(Value::Number(list2[i].clone()));
            }
        }

        Value::List(result)
    }
}

// ============ ZipSeq ============

pub struct ZipSeq;

static ZIP_SEQ_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list1",
        typ: "List",
        description: "First list",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "list2",
        typ: "List",
        description: "Second list",
        optional: false,
        default: None,
    },
];

static ZIP_SEQ_EXAMPLES: [&str; 1] = [
    "zip_seq([1, 2, 3], [\"a\", \"b\", \"c\"]) → [[1, \"a\"], [2, \"b\"], [3, \"c\"]]",
];

static ZIP_SEQ_RELATED: [&str; 1] = ["interleave"];

impl FunctionPlugin for ZipSeq {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "zip_seq",
            description: "Zip two sequences into pairs",
            usage: "zip_seq(list1, list2)",
            args: &ZIP_SEQ_ARGS,
            returns: "List<List>",
            examples: &ZIP_SEQ_EXAMPLES,
            category: "sequence/utility",
            source: None,
            related: &ZIP_SEQ_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("zip_seq", 2, args.len()));
        }

        let list1 = match &args[0] {
            Value::List(l) => l.clone(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("zip_seq", "list1", "List", other.type_name())),
        };

        let list2 = match &args[1] {
            Value::List(l) => l.clone(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("zip_seq", "list2", "List", other.type_name())),
        };

        let min_len = list1.len().min(list2.len());
        let mut result = Vec::with_capacity(min_len);

        for i in 0..min_len {
            result.push(Value::List(vec![list1[i].clone(), list2[i].clone()]));
        }

        Value::List(result)
    }
}

// ============ TakeSeq ============

pub struct TakeSeq;

static TAKE_SEQ_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "List to take from",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "n",
        typ: "Number",
        description: "Number of elements to take",
        optional: false,
        default: None,
    },
];

static TAKE_SEQ_EXAMPLES: [&str; 1] = [
    "take_seq(range(1, 100), 5) → [1, 2, 3, 4, 5]",
];

static TAKE_SEQ_RELATED: [&str; 2] = ["drop_seq", "slice_seq"];

impl FunctionPlugin for TakeSeq {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "take_seq",
            description: "Take first n elements from a sequence",
            usage: "take_seq(list, n)",
            args: &TAKE_SEQ_ARGS,
            returns: "List",
            examples: &TAKE_SEQ_EXAMPLES,
            category: "sequence/utility",
            source: None,
            related: &TAKE_SEQ_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("take_seq", 2, args.len()));
        }

        let list = match &args[0] {
            Value::List(l) => l.clone(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("take_seq", "list", "List", other.type_name())),
        };

        let n = match extract_number(&args[1], "n") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let n = match require_index(&n, "take_seq") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let take_count = n.min(list.len());
        Value::List(list[..take_count].to_vec())
    }
}

// ============ DropSeq ============

pub struct DropSeq;

static DROP_SEQ_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "List to drop from",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "n",
        typ: "Number",
        description: "Number of elements to drop",
        optional: false,
        default: None,
    },
];

static DROP_SEQ_EXAMPLES: [&str; 1] = [
    "drop_seq([1, 2, 3, 4, 5], 2) → [3, 4, 5]",
];

static DROP_SEQ_RELATED: [&str; 2] = ["take_seq", "slice_seq"];

impl FunctionPlugin for DropSeq {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "drop_seq",
            description: "Drop first n elements from a sequence",
            usage: "drop_seq(list, n)",
            args: &DROP_SEQ_ARGS,
            returns: "List",
            examples: &DROP_SEQ_EXAMPLES,
            category: "sequence/utility",
            source: None,
            related: &DROP_SEQ_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("drop_seq", 2, args.len()));
        }

        let list = match &args[0] {
            Value::List(l) => l.clone(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("drop_seq", "list", "List", other.type_name())),
        };

        let n = match extract_number(&args[1], "n") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let n = match require_index(&n, "drop_seq") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if n >= list.len() {
            return Value::List(vec![]);
        }

        Value::List(list[n..].to_vec())
    }
}

// ============ SliceSeq ============

pub struct SliceSeq;

static SLICE_SEQ_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "List to slice",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "start",
        typ: "Number",
        description: "Start index (0-indexed)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "end",
        typ: "Number",
        description: "End index (exclusive)",
        optional: false,
        default: None,
    },
];

static SLICE_SEQ_EXAMPLES: [&str; 1] = [
    "slice_seq([1, 2, 3, 4, 5], 1, 4) → [2, 3, 4]",
];

static SLICE_SEQ_RELATED: [&str; 2] = ["take_seq", "drop_seq"];

impl FunctionPlugin for SliceSeq {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "slice_seq",
            description: "Get a slice of a sequence (0-indexed, end exclusive)",
            usage: "slice_seq(list, start, end)",
            args: &SLICE_SEQ_ARGS,
            returns: "List",
            examples: &SLICE_SEQ_EXAMPLES,
            category: "sequence/utility",
            source: None,
            related: &SLICE_SEQ_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 3 {
            return Value::Error(FolioError::arg_count("slice_seq", 3, args.len()));
        }

        let list = match &args[0] {
            Value::List(l) => l.clone(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("slice_seq", "list", "List", other.type_name())),
        };

        let start = match extract_number(&args[1], "start") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let end = match extract_number(&args[2], "end") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let start = match require_index(&start, "slice_seq") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let end = match require_index(&end, "slice_seq") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if start >= list.len() {
            return Value::List(vec![]);
        }

        let end = end.min(list.len());
        if start >= end {
            return Value::List(vec![]);
        }

        Value::List(list[start..end].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_prime() {
        assert!(is_prime(2));
        assert!(is_prime(3));
        assert!(!is_prime(4));
        assert!(is_prime(5));
        assert!(is_prime(17));
        assert!(!is_prime(15));
        assert!(is_prime(97));
    }

    #[test]
    fn test_extract_list() {
        let arg = Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
        ]);
        let result = extract_list(&arg).unwrap();
        assert_eq!(result.len(), 3);
    }
}
