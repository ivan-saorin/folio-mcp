# Folio Text Functions Specification

## Overview

String manipulation, parsing, and validation functions. Uses Rust `regex` crate for pattern matching. ASCII-focused with UTF-8 safety and standard Unicode support (accented characters, common symbols). Lenient parsing with sensible overridable defaults.

---

## Module Structure

```
folio-text/
├── Cargo.toml
└── src/
    ├── lib.rs           # Registration
    ├── transform.rs     # Case, trim, pad
    ├── search.rs        # Find, contains, index
    ├── extract.rs       # Substring, split, regex extract
    ├── modify.rs        # Replace, remove, insert
    ├── join.rs          # Concat, join, format
    ├── parse.rs         # Parse numbers, dates, JSON
    └── validate.rs      # Patterns, checks
```

---

## Design Principles

### Lenient Defaults

```rust
// Parse with default on failure
parse_number("abc", 0)  // → 0 (default)
parse_number("abc")     // → Error

// Extract with empty on no match
extract("hello", r"\d+")  // → ""
extract("hello", r"\d+", null)  // → null (explicit default)
```

### Unicode Handling

- Functions work on Unicode code points by default
- `len("café")` → 4 (not 5 bytes)
- `substring("日本語", 0, 2)` → "日本"
- Regex uses Unicode mode

### Null Propagation

```rust
// Null input → Null output (not error)
upper(null)  // → null
trim(null)   // → null
```

---

## Functions

### Transform

#### `upper(text)`

Convert to uppercase.

```markdown
| result | upper("hello")   | HELLO |
| result | upper("café")    | CAFÉ  |
```

#### `lower(text)`

Convert to lowercase.

```markdown
| result | lower("HELLO")   | hello |
```

#### `capitalize(text)`

Capitalize first character.

```markdown
| result | capitalize("hello world") | Hello world |
```

#### `title_case(text)`

Capitalize first letter of each word.

```markdown
| result | title_case("hello world") | Hello World |
```

#### `trim(text)`

Remove leading and trailing whitespace.

```markdown
| result | trim("  hello  ") | hello |
```

#### `ltrim(text)`

Remove leading whitespace.

```markdown
| result | ltrim("  hello  ") | "hello  " |
```

#### `rtrim(text)`

Remove trailing whitespace.

```markdown
| result | rtrim("  hello  ") | "  hello" |
```

#### `trim_chars(text, chars)`

Remove specific characters from both ends.

```markdown
| result | trim_chars("...hello...", ".") | hello |
| result | trim_chars("##test##", "#")    | test  |
```

#### `pad_left(text, length, [char])`

Pad on left to reach length.

```markdown
| result | pad_left("42", 5, "0")   | 00042  |
| result | pad_left("hi", 5)        | "   hi" |
```

Default char: space.

#### `pad_right(text, length, [char])`

Pad on right to reach length.

```markdown
| result | pad_right("42", 5, "0") | 42000 |
```

#### `center(text, length, [char])`

Center text with padding.

```markdown
| result | center("hi", 6) | "  hi  " |
```

#### `repeat(text, count)`

Repeat text n times.

```markdown
| result | repeat("ab", 3) | ababab |
```

#### `reverse(text)`

Reverse string.

```markdown
| result | reverse("hello") | olleh |
```

---

### Search

#### `contains(text, search)`

Check if text contains substring.

```markdown
| result | contains("hello world", "world") | true  |
| result | contains("hello", "World")        | false |
```

#### `contains_any(text, searches)`

Check if text contains any of the substrings.

```markdown
| result | contains_any("hello", ["hi", "lo"]) | true |
```

#### `starts_with(text, prefix)`

Check if text starts with prefix.

```markdown
| result | starts_with("hello", "he") | true |
```

#### `ends_with(text, suffix)`

Check if text ends with suffix.

```markdown
| result | ends_with("hello", "lo") | true |
```

#### `index_of(text, search, [start])`

Find first occurrence (0-indexed). Returns -1 if not found.

```markdown
| result | index_of("hello", "l")    | 2  |
| result | index_of("hello", "l", 3) | 3  |
| result | index_of("hello", "x")    | -1 |
```

#### `last_index_of(text, search)`

Find last occurrence.

```markdown
| result | last_index_of("hello", "l") | 3 |
```

#### `count_matches(text, search)`

Count non-overlapping occurrences.

```markdown
| result | count_matches("banana", "a")  | 3 |
| result | count_matches("aaa", "aa")    | 1 |
```

#### `matches(text, pattern)`

Check if text matches regex pattern.

```markdown
| result | matches("hello123", r"[a-z]+\d+") | true |
```

---

### Extract

#### `len(text)`

Length in characters (Unicode code points).

```markdown
| result | len("hello") | 5 |
| result | len("café")  | 4 |
| result | len("日本")  | 2 |
```

#### `byte_len(text)`

Length in bytes (UTF-8).

```markdown
| result | byte_len("café") | 5 |
| result | byte_len("日本") | 6 |
```

#### `char_at(text, index)`

Character at index (0-indexed).

```markdown
| result | char_at("hello", 1) | e |
```

#### `substring(text, start, [end])`

Extract substring (0-indexed, end exclusive).

```markdown
| result | substring("hello", 1, 4)  | ell   |
| result | substring("hello", 2)     | llo   |
| result | substring("hello", -3)    | llo   |
| result | substring("hello", 1, -1) | ell   |
```

Negative indices count from end.

#### `left(text, count)`

First n characters.

```markdown
| result | left("hello", 2) | he |
```

#### `right(text, count)`

Last n characters.

```markdown
| result | right("hello", 2) | lo |
```

#### `mid(text, start, count)`

Extract count characters starting at index.

```markdown
| result | mid("hello", 1, 3) | ell |
```

#### `split(text, delimiter)`

Split into list.

```markdown
| result | split("a,b,c", ",")     | ["a", "b", "c"] |
| result | split("a  b  c", r"\s+") | ["a", "b", "c"] |
```

#### `split_lines(text)`

Split by newlines.

```markdown
| result | split_lines("a\nb\nc") | ["a", "b", "c"] |
```

Handles `\n`, `\r\n`, and `\r`.

#### `extract(text, pattern, [default])`

Extract first regex match.

```markdown
| result | extract("price: $42.50", r"\d+\.\d+")       | 42.50 |
| result | extract("no number", r"\d+", "0")           | 0     |
| result | extract("no number", r"\d+")                | ""    |
```

#### `extract_group(text, pattern, group)`

Extract specific capture group.

```markdown
| result | extract_group("John Smith", r"(\w+) (\w+)", 2) | Smith |
```

#### `extract_all(text, pattern)`

Extract all matches as list.

```markdown
| result | extract_all("a1b2c3", r"\d") | ["1", "2", "3"] |
```

#### `extract_groups(text, pattern)`

Extract all capture groups from first match.

```markdown
| result | extract_groups("John Smith", r"(\w+) (\w+)") | ["John", "Smith"] |
```

---

### Modify

#### `replace(text, search, replacement)`

Replace first occurrence.

```markdown
| result | replace("hello", "l", "L") | heLlo |
```

#### `replace_all(text, search, replacement)`

Replace all occurrences.

```markdown
| result | replace_all("hello", "l", "L") | heLLo |
```

#### `replace_regex(text, pattern, replacement)`

Replace using regex pattern.

```markdown
| result | replace_regex("hello123", r"\d+", "XXX") | helloXXX |
```

Supports backreferences: `$1`, `$2`, etc.

```markdown
| result | replace_regex("John Smith", r"(\w+) (\w+)", "$2, $1") | Smith, John |
```

#### `remove(text, search)`

Remove all occurrences.

```markdown
| result | remove("hello", "l") | heo |
```

#### `remove_regex(text, pattern)`

Remove all regex matches.

```markdown
| result | remove_regex("hello123world456", r"\d+") | helloworld |
```

#### `insert(text, index, insertion)`

Insert at position.

```markdown
| result | insert("hello", 2, "XY") | heXYllo |
```

#### `truncate(text, max_length, [suffix])`

Truncate with optional suffix.

```markdown
| result | truncate("hello world", 8, "...") | hello... |
| result | truncate("hi", 8, "...")           | hi       |
```

#### `ellipsis(text, max_length)`

Truncate with ellipsis, word-aware.

```markdown
| result | ellipsis("hello beautiful world", 15) | hello... |
```

Breaks at word boundary when possible.

#### `squeeze(text, [char])`

Collapse consecutive chars to single.

```markdown
| result | squeeze("hellooo   world")       | "helo world" |
| result | squeeze("hellooo   world", " ")  | "hellooo world" |
```

Default: all whitespace.

---

### Join

#### `concat(text1, text2, ...)`

Concatenate strings.

```markdown
| result | concat("hello", " ", "world") | hello world |
```

#### `join(list, delimiter)`

Join list elements.

```markdown
| result | join(["a", "b", "c"], ", ") | a, b, c |
```

#### `format(template, values)`

Format string with values.

```markdown
| result | format("{0} is {1}", ["answer", 42]) | answer is 42 |
| result | format("{name}: {value}", {name: "x", value: 10}) | x: 10 |
```

Supports both positional `{0}` and named `{name}` placeholders.

#### `template(text, vars)`

Mustache-style template.

```markdown
| result | template("Hello {{name}}!", {name: "World"}) | Hello World! |
```

---

### Parse

#### `parse_number(text, [default])`

Parse text to number.

```markdown
| result | parse_number("42")           | 42      |
| result | parse_number("3.14")         | 3.14    |
| result | parse_number("1,234.56")     | 1234.56 |
| result | parse_number("abc", 0)       | 0       |
| result | parse_number("abc")          | Error   |
```

Handles:
- Integer: `"123"`, `"-45"`
- Decimal: `"3.14"`, `".5"`
- Thousands separator: `"1,234,567"`
- Scientific: `"1.5e10"`
- Percentage: `"25%"` → 0.25

#### `parse_int(text, [default])`

Parse as integer.

```markdown
| result | parse_int("42.9")   | 42 |
| result | parse_int("abc", 0) | 0  |
```

Truncates decimal part.

#### `parse_float(text, [default])`

Parse as float (returns Number).

```markdown
| result | parse_float("3.14159") | 3.14159 |
```

#### `parse_bool(text, [default])`

Parse as boolean.

```markdown
| result | parse_bool("true")  | true  |
| result | parse_bool("yes")   | true  |
| result | parse_bool("1")     | true  |
| result | parse_bool("false") | false |
| result | parse_bool("no")    | false |
| result | parse_bool("0")     | false |
```

Truthy: `true`, `yes`, `y`, `1`, `on`, `t`
Falsy: `false`, `no`, `n`, `0`, `off`, `f`

#### `parse_date(text, [format], [default])`

Parse text to DateTime.

```markdown
| result | parse_date("2024-03-15")                     | DateTime |
| result | parse_date("15/03/2024", "DD/MM/YYYY")       | DateTime |
| result | parse_date("March 15, 2024", "MMMM D, YYYY") | DateTime |
```

Auto-detected formats (when format not specified):
- ISO 8601: `2024-03-15`, `2024-03-15T14:30:00`
- US: `03/15/2024`, `3/15/24`
- EU: `15.03.2024`, `15-03-2024`
- Long: `March 15, 2024`, `15 Mar 2024`

Format tokens:
| Token | Meaning | Example |
|-------|---------|---------|
| YYYY | 4-digit year | 2024 |
| YY | 2-digit year | 24 |
| MMMM | Full month | January |
| MMM | Short month | Jan |
| MM | 2-digit month | 03 |
| M | Month | 3 |
| DD | 2-digit day | 05 |
| D | Day | 5 |
| HH | 24-hour | 14 |
| hh | 12-hour | 02 |
| mm | Minutes | 30 |
| ss | Seconds | 45 |
| A | AM/PM | PM |

#### `parse_json(text, [default])`

Parse JSON string.

```markdown
| result | parse_json('{"a": 1, "b": 2}') | Object |
| val    | result.a                        | 1      |
```

#### `parse_csv_line(text, [delimiter])`

Parse single CSV line.

```markdown
| result | parse_csv_line("a,b,c")             | ["a", "b", "c"] |
| result | parse_csv_line('"a,b",c', ",")      | ["a,b", "c"]    |
| result | parse_csv_line("a;b;c", ";")        | ["a", "b", "c"] |
```

Handles quoted fields correctly.

---

### Validate

#### `is_empty(text)`

Check if null, empty, or whitespace-only.

```markdown
| result | is_empty("")      | true  |
| result | is_empty("  ")    | true  |
| result | is_empty(null)    | true  |
| result | is_empty("hello") | false |
```

#### `is_blank(text)`

Alias for `is_empty`.

#### `is_numeric(text)`

Check if parseable as number.

```markdown
| result | is_numeric("123")    | true  |
| result | is_numeric("12.34")  | true  |
| result | is_numeric("1e5")    | true  |
| result | is_numeric("abc")    | false |
```

#### `is_integer(text)`

Check if parseable as integer.

```markdown
| result | is_integer("123")   | true  |
| result | is_integer("12.34") | false |
```

#### `is_alpha(text)`

Check if only letters.

```markdown
| result | is_alpha("hello") | true  |
| result | is_alpha("hello1")| false |
| result | is_alpha("café")  | true  |
```

#### `is_alphanumeric(text)`

Check if only letters and digits.

```markdown
| result | is_alphanumeric("hello123") | true  |
| result | is_alphanumeric("hello!")   | false |
```

#### `is_email(text)`

Basic email format validation.

```markdown
| result | is_email("user@example.com") | true  |
| result | is_email("invalid")          | false |
```

Uses simplified RFC 5322 pattern.

#### `is_url(text)`

Basic URL format validation.

```markdown
| result | is_url("https://example.com") | true  |
| result | is_url("not a url")           | false |
```

#### `is_uuid(text)`

Check UUID format.

```markdown
| result | is_uuid("550e8400-e29b-41d4-a716-446655440000") | true |
```

#### `is_phone(text, [region])`

Basic phone number validation.

```markdown
| result | is_phone("+1-555-123-4567") | true |
```

#### `validate(text, pattern)`

Validate against regex.

```markdown
| result | validate("AB123", r"^[A-Z]{2}\d{3}$") | true |
```

---

## Implementation Notes

### Regex Caching

```rust
use std::sync::OnceLock;
use regex::Regex;

fn get_email_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
    })
}
```

Compile patterns once, reuse.

### Error Handling

```rust
fn extract(text: &str, pattern: &str, default: Option<&str>) -> Value {
    let re = match Regex::new(pattern) {
        Ok(r) => r,
        Err(e) => return Value::Error(FolioError::parse_error(
            format!("Invalid regex: {}", e)
        )),
    };
    
    match re.find(text) {
        Some(m) => Value::Text(m.as_str().to_string()),
        None => match default {
            Some(d) => Value::Text(d.to_string()),
            None => Value::Text(String::new()),
        }
    }
}
```

### Unicode Safety

```rust
fn substring(text: &str, start: i64, end: Option<i64>) -> Value {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len() as i64;
    
    // Handle negative indices
    let start = if start < 0 { (len + start).max(0) } else { start };
    let end = match end {
        Some(e) if e < 0 => (len + e).max(0),
        Some(e) => e.min(len),
        None => len,
    };
    
    if start >= len || start >= end {
        return Value::Text(String::new());
    }
    
    let result: String = chars[start as usize..end as usize].iter().collect();
    Value::Text(result)
}
```

---

## Examples

### Data Cleaning Pipeline

```markdown
## Raw Data

| id | name            | phone          |
|----|-----------------|----------------|
| 1  | "  JOHN DOE  "  | "555-123-4567" |
| 2  | "jane smith"    | "(555) 987-6543"|

## Cleaned @bind:n=name,p=phone

| id | clean_name                      | clean_phone                   |
|----|---------------------------------|-------------------------------|
| 1  | title_case(trim(n))             | remove_regex(p, r"[^\d]")     |
| 2  | title_case(trim(n))             | remove_regex(p, r"[^\d]")     |
```

### Parsing Mixed Data

```markdown
## Import

| raw_value | parsed                                          |
|-----------|-------------------------------------------------|
| "$1,234"  | parse_number(remove(raw_value, "$"))            |
| "15%"     | parse_number(raw_value)                         |
| "true"    | parse_bool(raw_value)                           |
| "2024-03" | parse_date(concat(raw_value, "-01"))            |
```

### Validation Report

```markdown
## Validation

| email                | is_valid                    |
|----------------------|-----------------------------|
| "user@example.com"   | is_email(email)             |
| "invalid"            | is_email(email)             |

| Metric       | Formula                                     | Result |
|--------------|---------------------------------------------|--------|
| valid_count  | count(filter(Validation.is_valid, v -> v)) | 1      |
```

---

## Function Summary

| Category | Functions |
|----------|-----------|
| **Transform** | `upper`, `lower`, `capitalize`, `title_case`, `trim`, `ltrim`, `rtrim`, `trim_chars`, `pad_left`, `pad_right`, `center`, `repeat`, `reverse` |
| **Search** | `contains`, `contains_any`, `starts_with`, `ends_with`, `index_of`, `last_index_of`, `count_matches`, `matches` |
| **Extract** | `len`, `byte_len`, `char_at`, `substring`, `left`, `right`, `mid`, `split`, `split_lines`, `extract`, `extract_group`, `extract_all`, `extract_groups` |
| **Modify** | `replace`, `replace_all`, `replace_regex`, `remove`, `remove_regex`, `insert`, `truncate`, `ellipsis`, `squeeze` |
| **Join** | `concat`, `join`, `format`, `template` |
| **Parse** | `parse_number`, `parse_int`, `parse_float`, `parse_bool`, `parse_date`, `parse_json`, `parse_csv_line` |
| **Validate** | `is_empty`, `is_blank`, `is_numeric`, `is_integer`, `is_alpha`, `is_alphanumeric`, `is_email`, `is_url`, `is_uuid`, `is_phone`, `validate` |

Total: 56 functions
