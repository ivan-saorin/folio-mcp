# Folio Sequences & Series Specification

## Overview

Sequence generation, named sequences, pattern detection, and series operations. Pragmatic list generators - no infinite sequences, no lambdas. Recurrence relations use string expressions.

---

## Module Structure

```
folio-sequence/
├── Cargo.toml
└── src/
    ├── lib.rs           # Registration
    ├── generators.rs    # range, linspace, arithmetic, geometric
    ├── named.rs         # fibonacci, lucas, primes, factorial, etc.
    ├── recurrence.rs    # Custom recurrence relations
    ├── pattern.rs       # Pattern detection
    ├── series.rs        # Sums and products
    └── expr.rs          # Mini-expression parser for recurrence
```

---

## Design Principles

### Pragmatic, Not Mathematical

```markdown
| fib | fibonacci(10) | [1, 1, 2, 3, 5, 8, 13, 21, 34, 55] |
```

Not:
```markdown
| fib | take(fibonacci(), 10) |  // No infinite sequences
```

### Offset Support

```markdown
| fib | fibonacci(5, 5) | [8, 13, 21, 34, 55] |
```

Syntax: `sequence(count, start_index)`

### No Lambdas

Recurrence uses string expressions:

```markdown
| fib | recurrence([1, 1], "a + b", 10) | [1, 1, 2, 3, 5, 8, ...] |
```

Not:
```markdown
| fib | recurrence([1, 1], (a, b) -> a + b, 10) |  // No lambda syntax
```

---

## Functions

### Basic Generators

#### `range(start, end, [step])`

Generate integer sequence.

```markdown
| r | range(1, 5)       | [1, 2, 3, 4, 5]   |
| r | range(0, 10, 2)   | [0, 2, 4, 6, 8, 10] |
| r | range(10, 1, -1)  | [10, 9, 8, 7, 6, 5, 4, 3, 2, 1] |
```

End is inclusive.

#### `linspace(start, end, count)`

Linear spacing (includes endpoints).

```markdown
| l | linspace(0, 1, 5) | [0, 0.25, 0.5, 0.75, 1] |
```

#### `logspace(start, end, count, [base])`

Logarithmic spacing.

```markdown
| l | logspace(1, 1000, 4)    | [1, 10, 100, 1000]     |
| l | logspace(1, 8, 4, 2)    | [1, 2, 4, 8]           |
```

Default base: 10

#### `arithmetic(first, diff, count)`

Arithmetic sequence.

```markdown
| a | arithmetic(5, 3, 6) | [5, 8, 11, 14, 17, 20] |
```

Formula: a_n = first + (n-1) × diff

#### `geometric(first, ratio, count)`

Geometric sequence.

```markdown
| g | geometric(2, 3, 5) | [2, 6, 18, 54, 162] |
```

Formula: a_n = first × ratio^(n-1)

#### `harmonic(count)`

Harmonic sequence: 1, 1/2, 1/3, 1/4, ...

```markdown
| h | harmonic(5) | [1, 1/2, 1/3, 1/4, 1/5] |
```

#### `repeat_seq(value, count)`

Repeat single value.

```markdown
| r | repeat_seq(7, 5) | [7, 7, 7, 7, 7] |
```

#### `cycle(list, count)`

Cycle through list.

```markdown
| c | cycle([1, 2, 3], 8) | [1, 2, 3, 1, 2, 3, 1, 2] |
```

---

### Named Sequences

#### `fibonacci(count, [start])`

Fibonacci sequence.

```markdown
| f | fibonacci(10)    | [1, 1, 2, 3, 5, 8, 13, 21, 34, 55] |
| f | fibonacci(5, 10) | [55, 89, 144, 233, 377]            |
| f | fibonacci(5, 0)  | [0, 1, 1, 2, 3]                    |
```

Start index: 0 = F(0)=0, 1 = F(1)=1, etc.

#### `lucas(count, [start])`

Lucas numbers (2, 1, 3, 4, 7, 11, ...).

```markdown
| l | lucas(8) | [2, 1, 3, 4, 7, 11, 18, 29] |
```

#### `tribonacci(count, [start])`

Tribonacci (each term = sum of previous 3).

```markdown
| t | tribonacci(10) | [0, 0, 1, 1, 2, 4, 7, 13, 24, 44] |
```

#### `primes(count, [start])`

Prime numbers.

```markdown
| p | primes(10)    | [2, 3, 5, 7, 11, 13, 17, 19, 23, 29] |
| p | primes(5, 10) | [29, 31, 37, 41, 43]                 |
```

Start index: 1 = first prime (2), 10 = 10th prime (29).

**Limit:** count ≤ 1000 (performance).

#### `primes_up_to(max)`

All primes up to max.

```markdown
| p | primes_up_to(30) | [2, 3, 5, 7, 11, 13, 17, 19, 23, 29] |
```

Uses Sieve of Eratosthenes.

**Limit:** max ≤ 1,000,000.

#### `factorial_seq(count)`

Factorials: 1, 1, 2, 6, 24, 120, ...

```markdown
| f | factorial_seq(7) | [1, 1, 2, 6, 24, 120, 720] |
```

#### `triangular(count)`

Triangular numbers: 1, 3, 6, 10, 15, ...

```markdown
| t | triangular(6) | [1, 3, 6, 10, 15, 21] |
```

Formula: T(n) = n(n+1)/2

#### `square_numbers(count)`

Perfect squares: 1, 4, 9, 16, 25, ...

```markdown
| s | square_numbers(6) | [1, 4, 9, 16, 25, 36] |
```

#### `cube_numbers(count)`

Perfect cubes: 1, 8, 27, 64, 125, ...

```markdown
| c | cube_numbers(5) | [1, 8, 27, 64, 125] |
```

#### `powers(base, count)`

Powers of base: base^0, base^1, base^2, ...

```markdown
| p | powers(2, 8) | [1, 2, 4, 8, 16, 32, 64, 128] |
```

#### `catalan(count)`

Catalan numbers: 1, 1, 2, 5, 14, 42, 132, ...

```markdown
| c | catalan(7) | [1, 1, 2, 5, 14, 42, 132] |
```

#### `bell(count)`

Bell numbers: 1, 1, 2, 5, 15, 52, 203, ...

```markdown
| b | bell(7) | [1, 1, 2, 5, 15, 52, 203] |
```

#### `pentagonal(count)`

Pentagonal numbers: 1, 5, 12, 22, 35, ...

```markdown
| p | pentagonal(6) | [1, 5, 12, 22, 35, 51] |
```

#### `hexagonal(count)`

Hexagonal numbers: 1, 6, 15, 28, 45, ...

```markdown
| h | hexagonal(5) | [1, 6, 15, 28, 45] |
```

---

### Recurrence Relations

#### `recurrence(initial, expr, count)`

Custom recurrence with string expression.

```markdown
| fib   | recurrence([1, 1], "a + b", 10)           | [1, 1, 2, 3, 5, ...] |
| tri   | recurrence([0, 0, 1], "a + b + c", 10)    | [0, 0, 1, 1, 2, 4, ...] |
| double| recurrence([1], "2 * a", 8)                | [1, 2, 4, 8, 16, ...] |
```

**Expression Variables:**
- `a`: Most recent value (n-1)
- `b`: Second most recent (n-2)
- `c`: Third most recent (n-3)
- `d`: Fourth most recent (n-4)
- `n`: Current index (1-based)

**Supported Operations:**
- Arithmetic: `+`, `-`, `*`, `/`, `^`
- Functions: `abs`, `sqrt`, `floor`, `ceil`
- Constants: Numbers, `n` (index)

**Examples:**
```markdown
| seq | recurrence([1], "a * n", 6)           | [1, 2, 6, 24, 120, 720] |  // Factorial
| seq | recurrence([0, 1], "a + 2*b", 8)      | [0, 1, 2, 5, 12, 29, ...] |  // Pell
| seq | recurrence([1, 1], "a * b", 6)        | [1, 1, 1, 1, 1, 1]        |  // Constant
```

#### `recurrence_named(name, count, [start])`

Predefined recurrence patterns.

```markdown
| seq | recurrence_named("fibonacci", 10)    | [1, 1, 2, 3, 5, ...] |
| seq | recurrence_named("pell", 8)          | [0, 1, 2, 5, 12, ...] |
| seq | recurrence_named("jacobsthal", 8)    | [0, 1, 1, 3, 5, ...] |
```

Named patterns:
- `"fibonacci"`: a + b
- `"lucas"`: a + b (different initial)
- `"pell"`: 2a + b
- `"jacobsthal"`: a + 2b
- `"tribonacci"`: a + b + c
- `"padovan"`: b + c
- `"perrin"`: b + c (different initial)

---

### Pattern Detection

#### `detect_pattern(list)`

Detect sequence type from data.

```markdown
| data    | [2, 5, 8, 11, 14]        |                            |
| pattern | detect_pattern(data)      |                            |
| type    | pattern.type              | "arithmetic"               |
| params  | pattern.parameters        | {first: 2, diff: 3}        |
| formula | pattern.formula           | "2 + 3*(n-1)"              |
```

**Returns Object:**
```json
{
  "type": "arithmetic" | "geometric" | "polynomial" | "fibonacci_like" | "unknown",
  "confidence": Number,  // 0-1
  "parameters": Object,  // Type-specific
  "formula": String,     // Human-readable
  "next_values": [Number, Number, Number]  // Predicted next 3
}
```

**Detected Types:**

| Type | Detection | Parameters |
|------|-----------|------------|
| `arithmetic` | Constant difference | first, diff |
| `geometric` | Constant ratio | first, ratio |
| `polynomial` | Fits polynomial | degree, coefficients |
| `fibonacci_like` | a_n = a_{n-1} + a_{n-2} | initial values |
| `power` | a_n = n^k | exponent |
| `factorial` | n! pattern | - |

**Minimum data:** 4 elements.

#### `extend_pattern(list, count)`

Extend sequence using detected pattern.

```markdown
| data     | [2, 4, 8, 16, 32]        |                       |
| extended | extend_pattern(data, 5)  | [64, 128, 256, 512, 1024] |
```

Returns error if pattern not detected with high confidence.

#### `is_arithmetic(list)`

Check if sequence is arithmetic.

```markdown
| result | is_arithmetic([2, 5, 8, 11]) | true  |
| result | is_arithmetic([1, 2, 4, 8])  | false |
```

#### `is_geometric(list)`

Check if sequence is geometric.

```markdown
| result | is_geometric([1, 2, 4, 8])   | true  |
| result | is_geometric([2, 5, 8, 11])  | false |
```

#### `common_diff(list)`

Get common difference (arithmetic).

```markdown
| d | common_diff([2, 5, 8, 11]) | 3 |
```

Error if not arithmetic.

#### `common_ratio(list)`

Get common ratio (geometric).

```markdown
| r | common_ratio([2, 6, 18, 54]) | 3 |
```

Error if not geometric.

#### `nth_term_formula(list)`

Get formula for nth term.

```markdown
| f | nth_term_formula([3, 7, 11, 15]) | "3 + 4*(n-1)" |
```

Returns string formula or "unknown".

---

### Series (Sums and Products)

#### `sum_seq(list)`

Sum of sequence elements.

```markdown
| s | sum_seq(range(1, 100)) | 5050 |
```

Same as `sum()` from aggregates.

#### `product_seq(list)`

Product of sequence elements.

```markdown
| p | product_seq(range(1, 5)) | 120 |  // 5!
```

#### `partial_sums(list)`

Cumulative sums.

```markdown
| ps | partial_sums([1, 2, 3, 4, 5]) | [1, 3, 6, 10, 15] |
```

#### `partial_products(list)`

Cumulative products.

```markdown
| pp | partial_products([1, 2, 3, 4]) | [1, 2, 6, 24] |
```

#### `alternating_sum(list)`

Sum with alternating signs: a_1 - a_2 + a_3 - a_4 + ...

```markdown
| as | alternating_sum([1, 2, 3, 4, 5]) | 3 |  // 1-2+3-4+5
```

#### `sum_formula(type, n, [params])`

Closed-form sum for known sequences.

```markdown
| s | sum_formula("arithmetic", 100, {first: 1, diff: 1}) | 5050 |
| s | sum_formula("geometric", 10, {first: 1, ratio: 2})  | 1023 |
| s | sum_formula("squares", 10)                          | 385  |
| s | sum_formula("cubes", 10)                             | 3025 |
```

Supported types:
- `"arithmetic"`: n(a_1 + a_n)/2
- `"geometric"`: a_1(r^n - 1)/(r - 1)
- `"squares"`: n(n+1)(2n+1)/6
- `"cubes"`: [n(n+1)/2]²
- `"triangular"`: n(n+1)(n+2)/6
- `"natural"`: n(n+1)/2

---

### Utility

#### `nth(sequence_name, n)`

Get nth element of named sequence.

```markdown
| f10 | nth("fibonacci", 10) | 55  |
| p5  | nth("prime", 5)      | 11  |
```

More efficient than generating full sequence.

#### `index_of_seq(list, value)`

Find index of value in sequence.

```markdown
| idx | index_of_seq(fibonacci(20), 55) | 10 |
```

Returns -1 if not found.

#### `is_in_sequence(value, sequence_name, [max_check])`

Check if value is in named sequence.

```markdown
| is_fib   | is_in_sequence(55, "fibonacci")   | true  |
| is_prime | is_in_sequence(17, "prime")       | true  |
| is_prime | is_in_sequence(15, "prime")       | false |
```

max_check: Maximum index to check (default: 1000).

#### `reverse_seq(list)`

Reverse sequence.

```markdown
| r | reverse_seq([1, 2, 3, 4, 5]) | [5, 4, 3, 2, 1] |
```

#### `interleave(list1, list2)`

Interleave two sequences.

```markdown
| i | interleave([1, 3, 5], [2, 4, 6]) | [1, 2, 3, 4, 5, 6] |
```

#### `zip_seq(list1, list2)`

Zip into pairs.

```markdown
| z | zip_seq([1, 2, 3], ["a", "b", "c"]) | [[1, "a"], [2, "b"], [3, "c"]] |
```

#### `take_seq(list, n)`

Take first n elements.

```markdown
| t | take_seq(range(1, 100), 5) | [1, 2, 3, 4, 5] |
```

#### `drop_seq(list, n)`

Drop first n elements.

```markdown
| d | drop_seq([1, 2, 3, 4, 5], 2) | [3, 4, 5] |
```

#### `slice_seq(list, start, end)`

Slice sequence (0-indexed).

```markdown
| s | slice_seq([1, 2, 3, 4, 5], 1, 4) | [2, 3, 4] |
```

---

## Implementation Notes

### Prime Generation

```rust
fn primes_up_to(max: u64) -> Vec<Number> {
    // Sieve of Eratosthenes
    let mut sieve = vec![true; (max + 1) as usize];
    sieve[0] = false;
    sieve[1] = false;
    
    for i in 2..=((max as f64).sqrt() as usize) {
        if sieve[i] {
            for j in (i * i..=max as usize).step_by(i) {
                sieve[j] = false;
            }
        }
    }
    
    sieve.iter().enumerate()
        .filter(|(_, &is_prime)| is_prime)
        .map(|(i, _)| Number::from_i64(i as i64))
        .collect()
}
```

### Recurrence Expression Parser

```rust
fn eval_recurrence_expr(expr: &str, prev: &[Number], n: i64) -> Result<Number, FolioError> {
    // Simple expression parser
    // Variables: a, b, c, d, n
    // Operations: +, -, *, /, ^
    
    let tokens = tokenize(expr)?;
    let ast = parse_expr(&tokens)?;
    
    let context = hashmap! {
        "a" => prev.get(prev.len() - 1).cloned().unwrap_or_default(),
        "b" => prev.get(prev.len().saturating_sub(2)).cloned().unwrap_or_default(),
        "c" => prev.get(prev.len().saturating_sub(3)).cloned().unwrap_or_default(),
        "d" => prev.get(prev.len().saturating_sub(4)).cloned().unwrap_or_default(),
        "n" => Number::from_i64(n),
    };
    
    eval_ast(&ast, &context)
}
```

### Pattern Detection

```rust
fn detect_pattern(data: &[Number]) -> PatternResult {
    if data.len() < 4 {
        return PatternResult::insufficient_data();
    }
    
    // Try arithmetic
    if let Some(diff) = check_arithmetic(data) {
        return PatternResult::arithmetic(data[0].clone(), diff);
    }
    
    // Try geometric
    if let Some(ratio) = check_geometric(data) {
        return PatternResult::geometric(data[0].clone(), ratio);
    }
    
    // Try fibonacci-like
    if check_fibonacci_like(data) {
        return PatternResult::fibonacci_like(data[0].clone(), data[1].clone());
    }
    
    // Try polynomial fit (degree 2, 3)
    if let Some((degree, coeffs)) = fit_polynomial(data, 3) {
        return PatternResult::polynomial(degree, coeffs);
    }
    
    PatternResult::unknown()
}
```

### Error Handling

```rust
fn primes(count: usize, start: Option<usize>) -> Value {
    if count > 1000 {
        return Value::Error(FolioError::domain_error(
            "primes() limited to 1000 elements for performance"
        ));
    }
    
    // ...
}

fn recurrence(initial: &[Number], expr: &str, count: usize) -> Value {
    if initial.is_empty() {
        return Value::Error(FolioError::domain_error(
            "recurrence() requires at least one initial value"
        ));
    }
    
    if count > 10000 {
        return Value::Error(FolioError::domain_error(
            "recurrence() limited to 10000 elements"
        ));
    }
    
    // Parse and validate expression
    match parse_recurrence_expr(expr) {
        Ok(_) => {},
        Err(e) => return Value::Error(FolioError::parse_error(
            format!("Invalid recurrence expression: {}", e)
        )),
    }
    
    // ...
}
```

---

## Examples

### Fibonacci Analysis

```markdown
## Fibonacci @precision:15

| Metric      | Formula                              | Result           |
|-------------|--------------------------------------|------------------|
| fib         | fibonacci(20)                        |                  |
| sum         | sum_seq(fib)                         | 17710            |
| ratios      | [fib[i+1]/fib[i] for i in 1..19]     |                  |
| last_ratio  | fib[19] / fib[18]                    | 1.6180339...     |
| phi         | (1 + sqrt(5)) / 2                    | 1.6180339...     |
| diff        | abs(last_ratio - phi)                | ~0.0000001       |
```

### Custom Sequence

```markdown
## Custom Recurrence

| Metric    | Formula                                   | Result               |
|-----------|-------------------------------------------|----------------------|
| tribonacci| recurrence([0, 0, 1], "a + b + c", 15)    | [0,0,1,1,2,4,7,...]  |
| pell      | recurrence([0, 1], "2*a + b", 10)         | [0,1,2,5,12,29,...]  |
| factorial | recurrence([1], "a * n", 10)              | [1,1,2,6,24,120,...] |
```

### Pattern Detection

```markdown
## Pattern Analysis

| Data       | Pattern                      | Next Values           |
|------------|------------------------------|-----------------------|
| [2,5,8,11] | detect_pattern($1)           |                       |
| type       | Data.Pattern.type            | "arithmetic"          |
| next       | extend_pattern([2,5,8,11], 3)| [14, 17, 20]          |

| Data2      | [1,4,9,16,25]                |                       |
| pattern2   | detect_pattern(Data2)        |                       |
| type2      | pattern2.type                | "polynomial"          |
| degree     | pattern2.parameters.degree   | 2                     |
```

### Series Formulas

```markdown
## Series

| Metric        | Formula                                        | Result |
|---------------|------------------------------------------------|--------|
| sum_100       | sum_formula("natural", 100)                    | 5050   |
| sum_squares   | sum_formula("squares", 10)                     | 385    |
| sum_cubes     | sum_formula("cubes", 10)                       | 3025   |
| geom_sum      | sum_formula("geometric", 10, {first:1, ratio:2})| 1023  |
```

---

## Function Summary

| Category | Functions |
|----------|-----------|
| **Generators** | `range`, `linspace`, `logspace`, `arithmetic`, `geometric`, `harmonic`, `repeat_seq`, `cycle` |
| **Named** | `fibonacci`, `lucas`, `tribonacci`, `primes`, `primes_up_to`, `factorial_seq`, `triangular`, `square_numbers`, `cube_numbers`, `powers`, `catalan`, `bell`, `pentagonal`, `hexagonal` |
| **Recurrence** | `recurrence`, `recurrence_named` |
| **Pattern** | `detect_pattern`, `extend_pattern`, `is_arithmetic`, `is_geometric`, `common_diff`, `common_ratio`, `nth_term_formula` |
| **Series** | `sum_seq`, `product_seq`, `partial_sums`, `partial_products`, `alternating_sum`, `sum_formula` |
| **Utility** | `nth`, `index_of_seq`, `is_in_sequence`, `reverse_seq`, `interleave`, `zip_seq`, `take_seq`, `drop_seq`, `slice_seq` |

Total: 44 functions
