# Folio Issues Mitigation Plan

## Issue 1: Phantom Constants (CRITICAL)

**Problem**: Constants like `m_e`, `m_μ`, `V_us`, etc. are registered but throw `UNKNOWN_CONSTANT` when used.

**Root Cause**: In `folio-plugin/src/context.rs:82-94`, the `eval_constant_formula()` function only handles 3 hardcoded formulas:
```rust
match formula {
    "pi" => Value::Number(folio_core::Number::pi(self.precision)),
    "exp(1)" => Value::Number(folio_core::Number::e(self.precision)),
    "(1 + sqrt(5)) / 2" => Value::Number(folio_core::Number::phi(self.precision)),
    _ => Value::Error(...) // All other constants fail here!
}
```

**Fix**: Update `eval_constant_formula()` to:
1. First try parsing the formula as a Number literal (handles `"0.51099895"`, `"1776.86"`, etc.)
2. Then handle special formulas (`"pi"`, `"exp(1)"`, `"(1 + sqrt(5)) / 2"`)
3. Finally, handle `sqrt(N)` formulas for `sqrt2`, `sqrt3`

**Files to modify**: `folio-plugin/src/context.rs`

---

## Issue 2: Unicode Constant Names

**Problem**: Constants like `m_μ` and `m_τ` may have encoding issues with lookup.

**Root Cause**: In `folio-plugin/src/registry.rs:55-56`:
```rust
let name = def.name.to_lowercase();  // Unicode μ.to_lowercase() might not work as expected
self.constants.insert(name, def);
```
And in lookup at line 69:
```rust
self.constants.get(&name.to_lowercase())  // "m_μ".to_lowercase() must match
```

**Fix**: Two options:
1. Register both Unicode and ASCII aliases: `m_μ` AND `m_mu`
2. Or normalize Unicode to ASCII in registry operations

**Recommendation**: Add ASCII aliases for all Unicode constants:
- `φ` → also register as `phi`
- `π` → also register as `pi`
- `α` → also register as `alpha`
- `m_μ` → also register as `m_mu`
- `m_τ` → also register as `m_tau`

**Files to modify**: `folio-std/src/lib.rs` (add alias registrations)

---

## Issue 3: Single `#` Headers Return Empty Results

**Problem**: Using `# Title` instead of `## Title` causes empty output.

**Root Cause**: In `folio/src/parser.rs:19`:
```rust
if line.starts_with("## ") {  // Only matches ##, not #
```
A single `#` header is ignored, but there's no fallback section created.

**Fix**: Two options:
1. Support `#` as section header (most flexible)
2. Create a default section if no `##` is found (safer)
3. Emit a warning/error if content is found without a section

**Recommendation**: Option 2+3: Create implicit section, emit warning.

**Files to modify**: `folio/src/parser.rs`

---

## Issue 4: Section Scoping Confusion

**Problem**: User believed variables are section-scoped, but they're actually document-scoped.

**Analysis**: Looking at `folio/src/eval.rs:48-66`, cells from ALL sections are collected into one HashMap, so scoping IS document-wide. The user's error was likely caused by Issue 1 (phantom constants) or Issue 5 (error propagation).

**Fix**: This is a documentation/error-message issue, not a code bug. When a variable is undefined, the error should suggest where it might be defined or if it's a constant.

**Files to modify**: `folio-core/src/error.rs` (improve UNDEFINED_VAR message)

---

## Issue 5: Error Propagation Without Root Cause

**Problem**: When one variable fails, all dependents show the same generic error without indicating the root cause.

**Example**:
```
| S_lep | ... | #ERROR: UNDEFINED_VAR |
| ln_tau_pred | ln_ratio * S_lep | #ERROR: UNDEFINED_VAR |  <- Should say "S_lep undefined"
```

**Fix**: Enhance error propagation to include dependency chain:
1. When evaluating an expression that references an errored variable, wrap the error with context
2. Use `FolioError.with_note()` to add "from dependency: X" context

**Files to modify**:
- `folio/src/eval.rs` (check for errors in dependencies before evaluation)
- `folio-core/src/error.rs` (add `caused_by` field or enhance notes)

---

## Issue 6: Boolean/Comparison Operators Not Supported

**Problem**: `abs(k0 - euler) < 0.001` returns undefined variable error.

**Root Cause**: The parser in `folio/src/parser.rs` doesn't recognize `<`, `>`, `==`, etc. as operators.

**Fix Options**:
1. Add comparison operators to parser and evaluator (significant work)
2. Add comparison functions: `lt(a, b)`, `gt(a, b)`, `eq(a, b)` (simpler)
3. Document this as a limitation (minimal work)

**Recommendation**: For now, document the limitation. If needed later, add functions like `lt()`, `gt()`, `approx(a, b, tolerance)`.

**Files to modify**: For option 3, just documentation.

---

## Implementation Priority

### Phase 1: Critical Fixes (High Impact, Low Effort)
1. **Issue 1**: Fix `eval_constant_formula()` to handle numeric constants
2. **Issue 2**: Add ASCII aliases for Unicode constants

### Phase 2: Parser Improvements
3. **Issue 3**: Add fallback section or support `#` headers

### Phase 3: Error Handling Improvements
4. **Issue 5**: Enhance error messages with dependency context
5. **Issue 4**: Document scoping behavior, improve undefined var message

### Phase 4: Future Consideration
6. **Issue 6**: Add comparison functions if needed

---

## Code Changes Summary

### `folio-plugin/src/context.rs`
```rust
fn eval_constant_formula(&self, formula: &str) -> Value {
    // First try: parse as number literal
    if let Ok(n) = folio_core::Number::from_str(formula) {
        return Value::Number(n);
    }

    // Handle special formulas
    match formula {
        "pi" => Value::Number(folio_core::Number::pi(self.precision)),
        "exp(1)" => Value::Number(folio_core::Number::e(self.precision)),
        "(1 + sqrt(5)) / 2" => Value::Number(folio_core::Number::phi(self.precision)),
        "sqrt(2)" => {
            let two = folio_core::Number::from_i64(2);
            two.sqrt(self.precision).map(Value::Number)
                .unwrap_or_else(|e| Value::Error(e.into()))
        }
        "sqrt(3)" => {
            let three = folio_core::Number::from_i64(3);
            three.sqrt(self.precision).map(Value::Number)
                .unwrap_or_else(|e| Value::Error(e.into()))
        }
        _ => Value::Error(...)
    }
}
```

### `folio-std/src/lib.rs`
Add ASCII aliases:
```rust
.with_constant(constants::phi())      // φ
.with_constant(phi_ascii())           // phi
.with_constant(constants::m_mu())     // m_μ
.with_constant(m_mu_ascii())          // m_mu
// etc.
```

### `folio/src/parser.rs`
```rust
// Add default section if none exists
if sections.is_empty() && !table_rows.is_empty() {
    sections.push(Section {
        name: "Default".to_string(),
        attributes: HashMap::new(),
        table: Table { rows: table_rows, columns },
    });
}
```
