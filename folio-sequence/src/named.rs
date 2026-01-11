//! Named mathematical sequences
//!
//! fibonacci, lucas, tribonacci, primes, factorial, triangular, etc.

use folio_plugin::prelude::*;
use crate::helpers::{extract_number, extract_optional_number, require_count, is_prime};

// ============ Fibonacci ============

pub struct Fibonacci;

static FIBONACCI_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Number of Fibonacci numbers to generate",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "start",
        typ: "Number",
        description: "Start index (0 = F(0)=0, 1 = F(1)=1, etc.)",
        optional: true,
        default: Some("1"),
    },
];

static FIBONACCI_EXAMPLES: [&str; 3] = [
    "fibonacci(10) → [1, 1, 2, 3, 5, 8, 13, 21, 34, 55]",
    "fibonacci(5, 10) → [55, 89, 144, 233, 377]",
    "fibonacci(5, 0) → [0, 1, 1, 2, 3]",
];

static FIBONACCI_RELATED: [&str; 2] = ["lucas", "tribonacci"];

impl FunctionPlugin for Fibonacci {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "fibonacci",
            description: "Generate Fibonacci sequence",
            usage: "fibonacci(count, [start])",
            args: &FIBONACCI_ARGS,
            returns: "List<Number>",
            examples: &FIBONACCI_EXAMPLES,
            category: "sequence/named",
            source: None,
            related: &FIBONACCI_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("fibonacci", 1, args.len()));
        }

        let count_num = match extract_number(&args[0], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "fibonacci", 10000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let start = match extract_optional_number(args, 1) {
            Ok(Some(n)) => {
                if !n.is_integer() || n.is_negative() {
                    return Value::Error(FolioError::domain_error(
                        "fibonacci() start must be non-negative integer"
                    ));
                }
                n.to_i64().unwrap_or(1) as usize
            }
            Ok(None) => 1,
            Err(e) => return Value::Error(e),
        };

        // Generate Fibonacci numbers
        let total_needed = start + count;
        let mut fibs = Vec::with_capacity(total_needed);

        let (mut a, mut b) = (Number::from_i64(0), Number::from_i64(1));
        for _ in 0..total_needed {
            fibs.push(a.clone());
            let next = a.add(&b);
            a = b;
            b = next;
        }

        let result: Vec<Value> = fibs[start..].iter()
            .take(count)
            .map(|n| Value::Number(n.clone()))
            .collect();

        Value::List(result)
    }
}

// ============ Lucas ============

pub struct Lucas;

static LUCAS_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Number of Lucas numbers to generate",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "start",
        typ: "Number",
        description: "Start index",
        optional: true,
        default: Some("0"),
    },
];

static LUCAS_EXAMPLES: [&str; 1] = [
    "lucas(8) → [2, 1, 3, 4, 7, 11, 18, 29]",
];

static LUCAS_RELATED: [&str; 2] = ["fibonacci", "tribonacci"];

impl FunctionPlugin for Lucas {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "lucas",
            description: "Generate Lucas numbers (2, 1, 3, 4, 7, 11, ...)",
            usage: "lucas(count, [start])",
            args: &LUCAS_ARGS,
            returns: "List<Number>",
            examples: &LUCAS_EXAMPLES,
            category: "sequence/named",
            source: None,
            related: &LUCAS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("lucas", 1, args.len()));
        }

        let count_num = match extract_number(&args[0], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "lucas", 10000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let start = match extract_optional_number(args, 1) {
            Ok(Some(n)) => {
                if !n.is_integer() || n.is_negative() {
                    return Value::Error(FolioError::domain_error(
                        "lucas() start must be non-negative integer"
                    ));
                }
                n.to_i64().unwrap_or(0) as usize
            }
            Ok(None) => 0,
            Err(e) => return Value::Error(e),
        };

        let total_needed = start + count;
        let mut nums = Vec::with_capacity(total_needed);

        // Lucas: L(0)=2, L(1)=1
        let (mut a, mut b) = (Number::from_i64(2), Number::from_i64(1));
        for _ in 0..total_needed {
            nums.push(a.clone());
            let next = a.add(&b);
            a = b;
            b = next;
        }

        let result: Vec<Value> = nums[start..].iter()
            .take(count)
            .map(|n| Value::Number(n.clone()))
            .collect();

        Value::List(result)
    }
}

// ============ Tribonacci ============

pub struct Tribonacci;

static TRIBONACCI_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Number of tribonacci numbers to generate",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "start",
        typ: "Number",
        description: "Start index",
        optional: true,
        default: Some("0"),
    },
];

static TRIBONACCI_EXAMPLES: [&str; 1] = [
    "tribonacci(10) → [0, 0, 1, 1, 2, 4, 7, 13, 24, 44]",
];

static TRIBONACCI_RELATED: [&str; 2] = ["fibonacci", "lucas"];

impl FunctionPlugin for Tribonacci {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "tribonacci",
            description: "Generate tribonacci sequence (each term = sum of previous 3)",
            usage: "tribonacci(count, [start])",
            args: &TRIBONACCI_ARGS,
            returns: "List<Number>",
            examples: &TRIBONACCI_EXAMPLES,
            category: "sequence/named",
            source: None,
            related: &TRIBONACCI_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("tribonacci", 1, args.len()));
        }

        let count_num = match extract_number(&args[0], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "tribonacci", 10000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let start = match extract_optional_number(args, 1) {
            Ok(Some(n)) => {
                if !n.is_integer() || n.is_negative() {
                    return Value::Error(FolioError::domain_error(
                        "tribonacci() start must be non-negative integer"
                    ));
                }
                n.to_i64().unwrap_or(0) as usize
            }
            Ok(None) => 0,
            Err(e) => return Value::Error(e),
        };

        let total_needed = start + count;
        let mut nums = Vec::with_capacity(total_needed);

        // Tribonacci: T(0)=0, T(1)=0, T(2)=1
        let (mut a, mut b, mut c) = (
            Number::from_i64(0),
            Number::from_i64(0),
            Number::from_i64(1),
        );

        for i in 0..total_needed {
            if i == 0 {
                nums.push(a.clone());
            } else if i == 1 {
                nums.push(b.clone());
            } else if i == 2 {
                nums.push(c.clone());
            } else {
                let next = a.add(&b).add(&c);
                a = b;
                b = c;
                c = next;
                nums.push(c.clone());
            }
        }

        let result: Vec<Value> = nums[start..].iter()
            .take(count)
            .map(|n| Value::Number(n.clone()))
            .collect();

        Value::List(result)
    }
}

// ============ Primes ============

pub struct Primes;

static PRIMES_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Number of primes to generate",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "start",
        typ: "Number",
        description: "Start index (1 = first prime = 2)",
        optional: true,
        default: Some("1"),
    },
];

static PRIMES_EXAMPLES: [&str; 2] = [
    "primes(10) → [2, 3, 5, 7, 11, 13, 17, 19, 23, 29]",
    "primes(5, 10) → [29, 31, 37, 41, 43]",
];

static PRIMES_RELATED: [&str; 1] = ["primes_up_to"];

impl FunctionPlugin for Primes {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "primes",
            description: "Generate prime numbers",
            usage: "primes(count, [start])",
            args: &PRIMES_ARGS,
            returns: "List<Number>",
            examples: &PRIMES_EXAMPLES,
            category: "sequence/named",
            source: None,
            related: &PRIMES_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("primes", 1, args.len()));
        }

        let count_num = match extract_number(&args[0], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "primes", 1000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let start = match extract_optional_number(args, 1) {
            Ok(Some(n)) => {
                if !n.is_integer() || n.is_negative() || n.is_zero() {
                    return Value::Error(FolioError::domain_error(
                        "primes() start must be positive integer"
                    ));
                }
                n.to_i64().unwrap_or(1) as usize
            }
            Ok(None) => 1,
            Err(e) => return Value::Error(e),
        };

        let total_needed = start + count - 1;
        let mut primes_list = Vec::with_capacity(total_needed);
        let mut candidate = 2u64;

        while primes_list.len() < total_needed {
            if is_prime(candidate) {
                primes_list.push(Number::from_i64(candidate as i64));
            }
            candidate += 1;
        }

        let result: Vec<Value> = primes_list[(start - 1)..].iter()
            .take(count)
            .map(|n| Value::Number(n.clone()))
            .collect();

        Value::List(result)
    }
}

// ============ PrimesUpTo ============

pub struct PrimesUpTo;

static PRIMES_UP_TO_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "max",
    typ: "Number",
    description: "Maximum value",
    optional: false,
    default: None,
}];

static PRIMES_UP_TO_EXAMPLES: [&str; 1] = [
    "primes_up_to(30) → [2, 3, 5, 7, 11, 13, 17, 19, 23, 29]",
];

static PRIMES_UP_TO_RELATED: [&str; 1] = ["primes"];

impl FunctionPlugin for PrimesUpTo {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "primes_up_to",
            description: "Generate all primes up to max (Sieve of Eratosthenes)",
            usage: "primes_up_to(max)",
            args: &PRIMES_UP_TO_ARGS,
            returns: "List<Number>",
            examples: &PRIMES_UP_TO_EXAMPLES,
            category: "sequence/named",
            source: None,
            related: &PRIMES_UP_TO_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("primes_up_to", 1, args.len()));
        }

        let max_num = match extract_number(&args[0], "max") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if !max_num.is_integer() || max_num.is_negative() {
            return Value::Error(FolioError::domain_error(
                "primes_up_to() requires non-negative integer"
            ));
        }

        let max = max_num.to_i64().unwrap_or(0) as usize;

        if max > 1_000_000 {
            return Value::Error(FolioError::domain_error(
                "primes_up_to() limited to max <= 1,000,000"
            ));
        }

        if max < 2 {
            return Value::List(vec![]);
        }

        // Sieve of Eratosthenes
        let mut sieve = vec![true; max + 1];
        sieve[0] = false;
        sieve[1] = false;

        let sqrt_max = (max as f64).sqrt() as usize;
        for i in 2..=sqrt_max {
            if sieve[i] {
                let mut j = i * i;
                while j <= max {
                    sieve[j] = false;
                    j += i;
                }
            }
        }

        let result: Vec<Value> = sieve.iter().enumerate()
            .filter(|(_, &is_prime)| is_prime)
            .map(|(i, _)| Value::Number(Number::from_i64(i as i64)))
            .collect();

        Value::List(result)
    }
}

// ============ FactorialSeq ============

pub struct FactorialSeq;

static FACTORIAL_SEQ_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "count",
    typ: "Number",
    description: "Number of factorials to generate (starting from 0!)",
    optional: false,
    default: None,
}];

static FACTORIAL_SEQ_EXAMPLES: [&str; 1] = [
    "factorial_seq(7) → [1, 1, 2, 6, 24, 120, 720]",
];

static FACTORIAL_SEQ_RELATED: [&str; 1] = ["triangular"];

impl FunctionPlugin for FactorialSeq {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "factorial_seq",
            description: "Generate sequence of factorials: 0!, 1!, 2!, 3!, ...",
            usage: "factorial_seq(count)",
            args: &FACTORIAL_SEQ_ARGS,
            returns: "List<Number>",
            examples: &FACTORIAL_SEQ_EXAMPLES,
            category: "sequence/named",
            source: None,
            related: &FACTORIAL_SEQ_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("factorial_seq", 1, args.len()));
        }

        let count_num = match extract_number(&args[0], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "factorial_seq", 171) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let mut result = Vec::with_capacity(count);
        let mut factorial = Number::from_i64(1);

        for i in 0..count {
            if i == 0 {
                result.push(Value::Number(Number::from_i64(1)));
            } else {
                factorial = factorial.mul(&Number::from_i64(i as i64));
                result.push(Value::Number(factorial.clone()));
            }
        }

        Value::List(result)
    }
}

// ============ Triangular ============

pub struct Triangular;

static TRIANGULAR_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "count",
    typ: "Number",
    description: "Number of triangular numbers to generate",
    optional: false,
    default: None,
}];

static TRIANGULAR_EXAMPLES: [&str; 1] = [
    "triangular(6) → [1, 3, 6, 10, 15, 21]",
];

static TRIANGULAR_RELATED: [&str; 2] = ["square_numbers", "pentagonal"];

impl FunctionPlugin for Triangular {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "triangular",
            description: "Generate triangular numbers: T(n) = n(n+1)/2",
            usage: "triangular(count)",
            args: &TRIANGULAR_ARGS,
            returns: "List<Number>",
            examples: &TRIANGULAR_EXAMPLES,
            category: "sequence/named",
            source: None,
            related: &TRIANGULAR_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("triangular", 1, args.len()));
        }

        let count_num = match extract_number(&args[0], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "triangular", 100000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let two = Number::from_i64(2);
        let result: Vec<Value> = (1..=count).map(|n| {
            let n = Number::from_i64(n as i64);
            let n_plus_1 = n.add(&Number::from_i64(1));
            let product = n.mul(&n_plus_1);
            match product.checked_div(&two) {
                Ok(val) => Value::Number(val),
                Err(e) => Value::Error(e.into()),
            }
        }).collect();

        Value::List(result)
    }
}

// ============ SquareNumbers ============

pub struct SquareNumbers;

static SQUARE_NUMBERS_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "count",
    typ: "Number",
    description: "Number of square numbers to generate",
    optional: false,
    default: None,
}];

static SQUARE_NUMBERS_EXAMPLES: [&str; 1] = [
    "square_numbers(6) → [1, 4, 9, 16, 25, 36]",
];

static SQUARE_NUMBERS_RELATED: [&str; 2] = ["cube_numbers", "triangular"];

impl FunctionPlugin for SquareNumbers {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "square_numbers",
            description: "Generate perfect squares: 1, 4, 9, 16, 25, ...",
            usage: "square_numbers(count)",
            args: &SQUARE_NUMBERS_ARGS,
            returns: "List<Number>",
            examples: &SQUARE_NUMBERS_EXAMPLES,
            category: "sequence/named",
            source: None,
            related: &SQUARE_NUMBERS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("square_numbers", 1, args.len()));
        }

        let count_num = match extract_number(&args[0], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "square_numbers", 100000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let result: Vec<Value> = (1..=count).map(|n| {
            let n = Number::from_i64(n as i64);
            Value::Number(n.mul(&n))
        }).collect();

        Value::List(result)
    }
}

// ============ CubeNumbers ============

pub struct CubeNumbers;

static CUBE_NUMBERS_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "count",
    typ: "Number",
    description: "Number of cube numbers to generate",
    optional: false,
    default: None,
}];

static CUBE_NUMBERS_EXAMPLES: [&str; 1] = [
    "cube_numbers(5) → [1, 8, 27, 64, 125]",
];

static CUBE_NUMBERS_RELATED: [&str; 2] = ["square_numbers", "powers"];

impl FunctionPlugin for CubeNumbers {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "cube_numbers",
            description: "Generate perfect cubes: 1, 8, 27, 64, 125, ...",
            usage: "cube_numbers(count)",
            args: &CUBE_NUMBERS_ARGS,
            returns: "List<Number>",
            examples: &CUBE_NUMBERS_EXAMPLES,
            category: "sequence/named",
            source: None,
            related: &CUBE_NUMBERS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("cube_numbers", 1, args.len()));
        }

        let count_num = match extract_number(&args[0], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "cube_numbers", 100000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let result: Vec<Value> = (1..=count).map(|n| {
            let n = Number::from_i64(n as i64);
            Value::Number(n.mul(&n).mul(&n))
        }).collect();

        Value::List(result)
    }
}

// ============ Powers ============

pub struct Powers;

static POWERS_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "base",
        typ: "Number",
        description: "Base number",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Number of powers to generate (starting from base^0)",
        optional: false,
        default: None,
    },
];

static POWERS_EXAMPLES: [&str; 1] = [
    "powers(2, 8) → [1, 2, 4, 8, 16, 32, 64, 128]",
];

static POWERS_RELATED: [&str; 2] = ["geometric", "square_numbers"];

impl FunctionPlugin for Powers {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "powers",
            description: "Generate powers of base: base^0, base^1, base^2, ...",
            usage: "powers(base, count)",
            args: &POWERS_ARGS,
            returns: "List<Number>",
            examples: &POWERS_EXAMPLES,
            category: "sequence/named",
            source: None,
            related: &POWERS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("powers", 2, args.len()));
        }

        let base = match extract_number(&args[0], "base") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count_num = match extract_number(&args[1], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "powers", 10000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let mut result = Vec::with_capacity(count);
        let mut current = Number::from_i64(1);

        for _ in 0..count {
            result.push(Value::Number(current.clone()));
            current = current.mul(&base);
        }

        Value::List(result)
    }
}

// ============ Catalan ============

pub struct Catalan;

static CATALAN_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "count",
    typ: "Number",
    description: "Number of Catalan numbers to generate",
    optional: false,
    default: None,
}];

static CATALAN_EXAMPLES: [&str; 1] = [
    "catalan(7) → [1, 1, 2, 5, 14, 42, 132]",
];

static CATALAN_RELATED: [&str; 1] = ["bell"];

impl FunctionPlugin for Catalan {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "catalan",
            description: "Generate Catalan numbers: C(n) = (2n)! / ((n+1)! * n!)",
            usage: "catalan(count)",
            args: &CATALAN_ARGS,
            returns: "List<Number>",
            examples: &CATALAN_EXAMPLES,
            category: "sequence/named",
            source: None,
            related: &CATALAN_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("catalan", 1, args.len()));
        }

        let count_num = match extract_number(&args[0], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "catalan", 100) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        // C(0) = 1, C(n+1) = C(n) * 2(2n+1) / (n+2)
        let mut result = Vec::with_capacity(count);
        let mut c = Number::from_i64(1);

        for n in 0..count {
            result.push(Value::Number(c.clone()));
            if n + 1 < count {
                // C(n+1) = C(n) * 2(2n+1) / (n+2)
                let two_n_plus_1 = Number::from_i64(2 * n as i64 + 1);
                let n_plus_2 = Number::from_i64(n as i64 + 2);
                let two = Number::from_i64(2);
                let numerator = c.mul(&two).mul(&two_n_plus_1);
                c = match numerator.checked_div(&n_plus_2) {
                    Ok(val) => val,
                    Err(e) => return Value::Error(e.into()),
                };
            }
        }

        Value::List(result)
    }
}

// ============ Bell ============

pub struct Bell;

static BELL_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "count",
    typ: "Number",
    description: "Number of Bell numbers to generate",
    optional: false,
    default: None,
}];

static BELL_EXAMPLES: [&str; 1] = [
    "bell(7) → [1, 1, 2, 5, 15, 52, 203]",
];

static BELL_RELATED: [&str; 1] = ["catalan"];

impl FunctionPlugin for Bell {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "bell",
            description: "Generate Bell numbers (number of partitions of a set)",
            usage: "bell(count)",
            args: &BELL_ARGS,
            returns: "List<Number>",
            examples: &BELL_EXAMPLES,
            category: "sequence/named",
            source: None,
            related: &BELL_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("bell", 1, args.len()));
        }

        let count_num = match extract_number(&args[0], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "bell", 100) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        // Use Bell triangle
        // B(0) = 1
        // Build triangle row by row
        let mut result = Vec::with_capacity(count);

        if count == 0 {
            return Value::List(result);
        }

        // Bell triangle: row[0] = previous row's last, row[i] = row[i-1] + prev_row[i-1]
        let mut prev_row = vec![Number::from_i64(1)];
        result.push(Value::Number(Number::from_i64(1)));

        for _ in 1..count {
            let mut row = Vec::with_capacity(prev_row.len() + 1);
            // First element is last element of previous row
            row.push(prev_row.last().unwrap().clone());

            // Build rest of row
            for i in 0..prev_row.len() {
                let next = row[i].add(&prev_row[i]);
                row.push(next);
            }

            result.push(Value::Number(row[0].clone()));
            prev_row = row;
        }

        Value::List(result)
    }
}

// ============ Pentagonal ============

pub struct Pentagonal;

static PENTAGONAL_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "count",
    typ: "Number",
    description: "Number of pentagonal numbers to generate",
    optional: false,
    default: None,
}];

static PENTAGONAL_EXAMPLES: [&str; 1] = [
    "pentagonal(6) → [1, 5, 12, 22, 35, 51]",
];

static PENTAGONAL_RELATED: [&str; 2] = ["triangular", "hexagonal"];

impl FunctionPlugin for Pentagonal {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "pentagonal",
            description: "Generate pentagonal numbers: P(n) = n(3n-1)/2",
            usage: "pentagonal(count)",
            args: &PENTAGONAL_ARGS,
            returns: "List<Number>",
            examples: &PENTAGONAL_EXAMPLES,
            category: "sequence/named",
            source: None,
            related: &PENTAGONAL_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("pentagonal", 1, args.len()));
        }

        let count_num = match extract_number(&args[0], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "pentagonal", 100000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let two = Number::from_i64(2);
        let three = Number::from_i64(3);
        let one = Number::from_i64(1);

        let result: Vec<Value> = (1..=count).map(|n| {
            let n = Number::from_i64(n as i64);
            // P(n) = n(3n-1)/2
            let three_n_minus_1 = three.mul(&n).sub(&one);
            let product = n.mul(&three_n_minus_1);
            match product.checked_div(&two) {
                Ok(val) => Value::Number(val),
                Err(e) => Value::Error(e.into()),
            }
        }).collect();

        Value::List(result)
    }
}

// ============ Hexagonal ============

pub struct Hexagonal;

static HEXAGONAL_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "count",
    typ: "Number",
    description: "Number of hexagonal numbers to generate",
    optional: false,
    default: None,
}];

static HEXAGONAL_EXAMPLES: [&str; 1] = [
    "hexagonal(5) → [1, 6, 15, 28, 45]",
];

static HEXAGONAL_RELATED: [&str; 2] = ["triangular", "pentagonal"];

impl FunctionPlugin for Hexagonal {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "hexagonal",
            description: "Generate hexagonal numbers: H(n) = n(2n-1)",
            usage: "hexagonal(count)",
            args: &HEXAGONAL_ARGS,
            returns: "List<Number>",
            examples: &HEXAGONAL_EXAMPLES,
            category: "sequence/named",
            source: None,
            related: &HEXAGONAL_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("hexagonal", 1, args.len()));
        }

        let count_num = match extract_number(&args[0], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "hexagonal", 100000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let two = Number::from_i64(2);
        let one = Number::from_i64(1);

        let result: Vec<Value> = (1..=count).map(|n| {
            let n = Number::from_i64(n as i64);
            // H(n) = n(2n-1)
            let two_n_minus_1 = two.mul(&n).sub(&one);
            Value::Number(n.mul(&two_n_minus_1))
        }).collect();

        Value::List(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_fibonacci() {
        let fib = Fibonacci;
        let args = vec![Value::Number(Number::from_i64(10))];
        let result = fib.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 10);
        // Default start=1 means F(1), F(2), ... = 1, 1, 2, 3, 5, 8, 13, 21, 34, 55
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(1));
        assert_eq!(list[1].as_number().unwrap().to_i64(), Some(1));
        assert_eq!(list[2].as_number().unwrap().to_i64(), Some(2));
        assert_eq!(list[9].as_number().unwrap().to_i64(), Some(55));
    }

    #[test]
    fn test_primes() {
        let primes = Primes;
        let args = vec![Value::Number(Number::from_i64(5))];
        let result = primes.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 5);
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(2));
        assert_eq!(list[1].as_number().unwrap().to_i64(), Some(3));
        assert_eq!(list[2].as_number().unwrap().to_i64(), Some(5));
        assert_eq!(list[3].as_number().unwrap().to_i64(), Some(7));
        assert_eq!(list[4].as_number().unwrap().to_i64(), Some(11));
    }

    #[test]
    fn test_triangular() {
        let tri = Triangular;
        let args = vec![Value::Number(Number::from_i64(5))];
        let result = tri.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 5);
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(1));
        assert_eq!(list[1].as_number().unwrap().to_i64(), Some(3));
        assert_eq!(list[2].as_number().unwrap().to_i64(), Some(6));
        assert_eq!(list[3].as_number().unwrap().to_i64(), Some(10));
        assert_eq!(list[4].as_number().unwrap().to_i64(), Some(15));
    }

    #[test]
    fn test_catalan() {
        let cat = Catalan;
        let args = vec![Value::Number(Number::from_i64(5))];
        let result = cat.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 5);
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(1));
        assert_eq!(list[1].as_number().unwrap().to_i64(), Some(1));
        assert_eq!(list[2].as_number().unwrap().to_i64(), Some(2));
        assert_eq!(list[3].as_number().unwrap().to_i64(), Some(5));
        assert_eq!(list[4].as_number().unwrap().to_i64(), Some(14));
    }
}
