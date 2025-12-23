//! Markdown table parser

use crate::ast::{Document, Section, Table, Row, Cell, Expr, BinOp};
use folio_core::FolioError;
use std::collections::HashMap;

/// Parse markdown document to AST
pub fn parse(input: &str) -> Result<Document, FolioError> {
    let mut sections = Vec::new();
    let mut current_section: Option<Section> = None;
    let mut in_table = false;
    let mut table_rows: Vec<Row> = Vec::new();
    let mut columns: Vec<String> = Vec::new();
    
    for line in input.lines() {
        let line = line.trim();

        // Section header - support both # and ## (# takes priority check first)
        if line.starts_with("# ") && !line.starts_with("## ") {
            // Single # header - treat as section
            if let Some(mut sec) = current_section.take() {
                sec.table.rows = std::mem::take(&mut table_rows);
                sec.table.columns = std::mem::take(&mut columns);
                sections.push(sec);
            }

            let header = &line[2..]; // Skip "# "
            let (name, attrs) = parse_section_header(header);
            current_section = Some(Section {
                name,
                attributes: attrs,
                table: Table::default(),
            });
            in_table = false;
            continue;
        }

        // Double ## section header
        if line.starts_with("## ") {
            // Save previous section
            if let Some(mut sec) = current_section.take() {
                sec.table.rows = std::mem::take(&mut table_rows);
                sec.table.columns = std::mem::take(&mut columns);
                sections.push(sec);
            }
            
            let header = &line[3..];
            let (name, attrs) = parse_section_header(header);
            current_section = Some(Section {
                name,
                attributes: attrs,
                table: Table::default(),
            });
            in_table = false;
            continue;
        }
        
        // Table header
        if line.starts_with('|') && line.ends_with('|') && !in_table {
            columns = parse_table_row_cells(line);
            in_table = true;
            continue;
        }
        
        // Table separator (only matches lines that contain only |, -, :, and whitespace)
        if line.starts_with('|') && line.ends_with('|') && in_table && table_rows.is_empty() {
            let is_separator = line.chars().all(|c| c == '|' || c == '-' || c == ':' || c.is_whitespace());
            if is_separator {
                continue;
            }
        }
        
        // Table row
        if line.starts_with('|') && line.ends_with('|') && in_table {
            let cells_text = parse_table_row_cells(line);
            if cells_text.len() >= 2 {
                let name = cells_text[0].trim().to_string();
                let formula_text = cells_text[1].trim().to_string();

                // Check for formula indicator (=) and strip it
                let (is_formula, expr_text) = if formula_text.starts_with('=') {
                    (true, formula_text[1..].trim().to_string())
                } else {
                    (false, formula_text.clone())
                };

                let formula = if expr_text.is_empty() {
                    None
                } else if is_formula {
                    // Explicitly marked as formula with =
                    Some(parse_expr(&expr_text)?)
                } else {
                    // Check if it looks like an expression (contains operators or function calls)
                    // Otherwise treat as literal value
                    if looks_like_expression(&expr_text) {
                        Some(parse_expr(&expr_text)?)
                    } else {
                        None // Treat as literal
                    }
                };
                
                table_rows.push(Row {
                    cells: vec![Cell {
                        name: name.clone(),
                        formula,
                        raw_text: expr_text, // Store the expression text (without = prefix)
                    }],
                });
            }
            continue;
        }

        // Empty line ends table
        if line.is_empty() && in_table {
            in_table = false;
        }
    }

    // Save last section
    if let Some(mut sec) = current_section {
        sec.table.rows = table_rows;
        sec.table.columns = columns;
        sections.push(sec);
    } else if !table_rows.is_empty() {
        // Fallback: create default section if there's content but no section header
        sections.push(Section {
            name: "Default".to_string(),
            attributes: HashMap::new(),
            table: Table { rows: table_rows, columns },
        });
    }

    Ok(Document { sections })
}

/// Check if text looks like an expression (vs a literal value)
fn looks_like_expression(text: &str) -> bool {
    let text = text.trim();

    // First check if it's a valid number literal (including scientific notation)
    if is_number_literal(text) {
        return false;
    }

    // List literal
    if text.starts_with('[') && text.ends_with(']') {
        return true;
    }

    // Contains operators (but not just a negative number)
    if text.contains('+') || text.contains('*') || text.contains('/') || text.contains('^') {
        return true;
    }

    // Contains comparison operators
    if text.contains('<') || text.contains('>') || text.contains("==") || text.contains("!=") {
        return true;
    }

    // Contains subtraction that's not a leading minus
    if let Some(pos) = text.find('-') {
        if pos > 0 {
            return true;
        }
    }

    // Contains function call
    if text.contains('(') && text.contains(')') {
        return true;
    }

    // References a variable (starts with letter, not just a number)
    if text.chars().next().map_or(false, |c| c.is_alphabetic()) {
        return true;
    }

    false
}

/// Check if text is a valid number literal (integer, decimal, or scientific notation)
fn is_number_literal(text: &str) -> bool {
    let text = text.trim();
    if text.is_empty() {
        return false;
    }

    // If it contains spaces, it's an expression, not a literal
    if text.contains(' ') {
        return false;
    }

    // Try parsing as f64 - handles integers, decimals, and scientific notation
    if text.parse::<f64>().is_ok() {
        return true;
    }

    // Also check for fraction format without spaces (e.g., "1/3" but not "42 / 0")
    if let Some(slash_pos) = text.find('/') {
        let (num, den) = text.split_at(slash_pos);
        let den = &den[1..]; // skip the '/'
        if !num.is_empty() && !den.is_empty()
            && num.parse::<f64>().is_ok()
            && den.parse::<f64>().is_ok() {
            return true;
        }
    }

    false
}

fn parse_section_header(header: &str) -> (String, HashMap<String, String>) {
    let mut attrs = HashMap::new();
    let parts: Vec<&str> = header.split('@').collect();
    let name = parts[0].trim().to_string();
    
    for attr_part in parts.iter().skip(1) {
        if let Some((key, value)) = attr_part.split_once(':') {
            attrs.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    
    (name, attrs)
}

fn parse_table_row_cells(line: &str) -> Vec<String> {
    line.trim_matches('|')
        .split('|')
        .map(|s| s.trim().to_string())
        .collect()
}

/// Parse expression (simple recursive descent)
pub fn parse_expr(input: &str) -> Result<Expr, FolioError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(FolioError::parse_error("Empty expression"));
    }

    parse_comparison(input)
}

/// Parse comparison operators (lowest precedence)
fn parse_comparison(input: &str) -> Result<Expr, FolioError> {
    let mut paren_depth = 0;
    let mut bracket_depth = 0;
    let mut in_double_quote = false;
    let mut in_single_quote = false;

    let char_indices: Vec<(usize, char)> = input.char_indices().collect();

    // Scan for comparison operators from left to right (left associative)
    let mut i = 0;
    while i < char_indices.len() {
        let (byte_pos, c) = char_indices[i];
        match c {
            '"' if !in_single_quote => in_double_quote = !in_double_quote,
            '\'' if !in_double_quote => in_single_quote = !in_single_quote,
            '(' if !in_double_quote && !in_single_quote => paren_depth += 1,
            ')' if !in_double_quote && !in_single_quote => paren_depth -= 1,
            '[' if !in_double_quote && !in_single_quote => bracket_depth += 1,
            ']' if !in_double_quote && !in_single_quote => bracket_depth -= 1,
            '<' | '>' | '=' | '!' if paren_depth == 0 && bracket_depth == 0 && !in_double_quote && !in_single_quote => {
                // Check for two-character operators
                let next_char = if i + 1 < char_indices.len() { Some(char_indices[i + 1].1) } else { None };
                let (op, op_len) = match (c, next_char) {
                    ('<', Some('=')) => (Some(BinOp::Le), 2),
                    ('>', Some('=')) => (Some(BinOp::Ge), 2),
                    ('=', Some('=')) => (Some(BinOp::Eq), 2),
                    ('!', Some('=')) => (Some(BinOp::Ne), 2),
                    ('<', _) => (Some(BinOp::Lt), 1),
                    ('>', _) => (Some(BinOp::Gt), 1),
                    _ => (None, 1),
                };

                if let Some(op) = op {
                    let left = input[..byte_pos].trim();
                    let right_start = if op_len == 2 {
                        char_indices[i + 1].0 + char_indices[i + 1].1.len_utf8()
                    } else {
                        byte_pos + c.len_utf8()
                    };
                    let right = input[right_start..].trim();

                    if !left.is_empty() && !right.is_empty() {
                        return Ok(Expr::BinaryOp(
                            Box::new(parse_additive(left)?),
                            op,
                            Box::new(parse_additive(right)?),
                        ));
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }

    parse_additive(input)
}

fn parse_additive(input: &str) -> Result<Expr, FolioError> {
    // Find + or - not inside parentheses, brackets, function calls, or quotes
    let mut paren_depth = 0;
    let mut bracket_depth = 0;
    let mut in_double_quote = false;
    let mut in_single_quote = false;

    // Collect (byte_offset, char) pairs to handle multi-byte UTF-8 correctly
    let char_indices: Vec<(usize, char)> = input.char_indices().collect();

    for idx in (0..char_indices.len()).rev() {
        let (byte_pos, c) = char_indices[idx];
        match c {
            '"' if !in_single_quote => in_double_quote = !in_double_quote,
            '\'' if !in_double_quote => in_single_quote = !in_single_quote,
            ')' if !in_double_quote && !in_single_quote => paren_depth += 1,
            '(' if !in_double_quote && !in_single_quote => paren_depth -= 1,
            ']' if !in_double_quote && !in_single_quote => bracket_depth += 1,
            '[' if !in_double_quote && !in_single_quote => bracket_depth -= 1,
            '+' | '-' if paren_depth == 0 && bracket_depth == 0 && idx > 0 && !in_double_quote && !in_single_quote => {
                let left = input[..byte_pos].trim();
                let right = input[byte_pos + c.len_utf8()..].trim();
                if !left.is_empty() && !right.is_empty() {
                    let op = if c == '+' { BinOp::Add } else { BinOp::Sub };
                    return Ok(Expr::BinaryOp(
                        Box::new(parse_additive(left)?),
                        op,
                        Box::new(parse_multiplicative(right)?),
                    ));
                }
            }
            _ => {}
        }
    }

    parse_multiplicative(input)
}

fn parse_multiplicative(input: &str) -> Result<Expr, FolioError> {
    let mut paren_depth = 0;
    let mut bracket_depth = 0;
    let mut in_double_quote = false;
    let mut in_single_quote = false;

    // Collect (byte_offset, char) pairs to handle multi-byte UTF-8 correctly
    let char_indices: Vec<(usize, char)> = input.char_indices().collect();

    for idx in (0..char_indices.len()).rev() {
        let (byte_pos, c) = char_indices[idx];
        match c {
            '"' if !in_single_quote => in_double_quote = !in_double_quote,
            '\'' if !in_double_quote => in_single_quote = !in_single_quote,
            ')' if !in_double_quote && !in_single_quote => paren_depth += 1,
            '(' if !in_double_quote && !in_single_quote => paren_depth -= 1,
            ']' if !in_double_quote && !in_single_quote => bracket_depth += 1,
            '[' if !in_double_quote && !in_single_quote => bracket_depth -= 1,
            '*' | '/' if paren_depth == 0 && bracket_depth == 0 && !in_double_quote && !in_single_quote => {
                let left = input[..byte_pos].trim();
                let right = input[byte_pos + c.len_utf8()..].trim();
                if !left.is_empty() && !right.is_empty() {
                    let op = if c == '*' { BinOp::Mul } else { BinOp::Div };
                    return Ok(Expr::BinaryOp(
                        Box::new(parse_multiplicative(left)?),
                        op,
                        Box::new(parse_power(right)?),
                    ));
                }
            }
            _ => {}
        }
    }

    parse_power(input)
}

fn parse_power(input: &str) -> Result<Expr, FolioError> {
    let mut paren_depth = 0;
    let mut bracket_depth = 0;
    let mut in_double_quote = false;
    let mut in_single_quote = false;

    // Collect (byte_offset, char) pairs to handle multi-byte UTF-8 correctly
    let char_indices: Vec<(usize, char)> = input.char_indices().collect();

    for idx in 0..char_indices.len() {
        let (byte_pos, c) = char_indices[idx];
        match c {
            '"' if !in_single_quote => in_double_quote = !in_double_quote,
            '\'' if !in_double_quote => in_single_quote = !in_single_quote,
            '(' if !in_double_quote && !in_single_quote => paren_depth += 1,
            ')' if !in_double_quote && !in_single_quote => paren_depth -= 1,
            '[' if !in_double_quote && !in_single_quote => bracket_depth += 1,
            ']' if !in_double_quote && !in_single_quote => bracket_depth -= 1,
            '^' if paren_depth == 0 && bracket_depth == 0 && !in_double_quote && !in_single_quote => {
                let left = input[..byte_pos].trim();
                let right = input[byte_pos + c.len_utf8()..].trim();
                if !left.is_empty() && !right.is_empty() {
                    return Ok(Expr::BinaryOp(
                        Box::new(parse_primary(left)?),
                        BinOp::Pow,
                        Box::new(parse_power(right)?),
                    ));
                }
            }
            _ => {}
        }
    }

    parse_primary(input)
}

fn parse_primary(input: &str) -> Result<Expr, FolioError> {
    let input = input.trim();

    // String literal (double-quoted)
    if input.starts_with('"') && input.ends_with('"') && input.len() >= 2 {
        let content = &input[1..input.len()-1];
        return Ok(Expr::StringLiteral(content.to_string()));
    }

    // String literal (single-quoted) - also support single quotes
    if input.starts_with('\'') && input.ends_with('\'') && input.len() >= 2 {
        let content = &input[1..input.len()-1];
        return Ok(Expr::StringLiteral(content.to_string()));
    }

    // List literal: [a, b, c]
    if input.starts_with('[') && input.ends_with(']') && input.len() >= 2 {
        let content = &input[1..input.len()-1];
        let elements = parse_list_elements(content)?;
        return Ok(Expr::List(elements));
    }

    // Parentheses
    if input.starts_with('(') && input.ends_with(')') {
        return parse_expr(&input[1..input.len()-1]);
    }

    // Function call - need to find matching closing parenthesis
    if let Some(paren_pos) = input.find('(') {
        let func_name = input[..paren_pos].trim().to_string();
        // Find the matching closing parenthesis
        let after_open = &input[paren_pos+1..];
        let mut depth = 1;
        let mut close_pos = None;
        let mut in_double_quote = false;
        let mut in_single_quote = false;
        for (i, c) in after_open.char_indices() {
            match c {
                '"' if !in_single_quote => in_double_quote = !in_double_quote,
                '\'' if !in_double_quote => in_single_quote = !in_single_quote,
                '(' if !in_double_quote && !in_single_quote => depth += 1,
                ')' if !in_double_quote && !in_single_quote => {
                    depth -= 1;
                    if depth == 0 {
                        close_pos = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }
        if let Some(close_idx) = close_pos {
            let args_str = &after_open[..close_idx];
            let args = parse_args(args_str)?;
            let func_call = Expr::FunctionCall(func_name, args);

            // Check if there's a property access after the function call
            let after_close = &after_open[close_idx + 1..];
            if after_close.starts_with('.') {
                // Parse as field access: func().prop.subprop
                let field_names: Vec<String> = after_close[1..].split('.').map(|s| s.trim().to_string()).collect();
                return Ok(Expr::FieldAccess(Box::new(func_call), field_names));
            }
            return Ok(func_call);
        }
    }

    // Number
    if input.chars().next().map_or(false, |c| c.is_ascii_digit() || c == '-' || c == '.') {
        if input.parse::<f64>().is_ok() || input.contains('/') {
            return Ok(Expr::Number(input.to_string()));
        }
    }

    // Variable (possibly dotted for Section.Column resolution)
    let parts: Vec<String> = input.split('.').map(|s| s.trim().to_string()).collect();
    Ok(Expr::Variable(parts))
}

/// Parse list literal elements: a, b, c (similar to args but for lists)
fn parse_list_elements(input: &str) -> Result<Vec<Expr>, FolioError> {
    if input.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut elements = Vec::new();
    let mut paren_depth = 0;
    let mut bracket_depth = 0;
    let mut in_double_quote = false;
    let mut in_single_quote = false;
    let mut current_start = 0;

    // Use char_indices for proper UTF-8 handling
    for (byte_pos, c) in input.char_indices() {
        match c {
            '"' if !in_single_quote => in_double_quote = !in_double_quote,
            '\'' if !in_double_quote => in_single_quote = !in_single_quote,
            '(' if !in_double_quote && !in_single_quote => paren_depth += 1,
            ')' if !in_double_quote && !in_single_quote => paren_depth -= 1,
            '[' if !in_double_quote && !in_single_quote => bracket_depth += 1,
            ']' if !in_double_quote && !in_single_quote => bracket_depth -= 1,
            ',' if paren_depth == 0 && bracket_depth == 0 && !in_double_quote && !in_single_quote => {
                elements.push(parse_expr(&input[current_start..byte_pos])?);
                current_start = byte_pos + c.len_utf8();
            }
            _ => {}
        }
    }

    elements.push(parse_expr(&input[current_start..])?);
    Ok(elements)
}

fn parse_args(input: &str) -> Result<Vec<Expr>, FolioError> {
    if input.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut args = Vec::new();
    let mut paren_depth = 0;
    let mut bracket_depth = 0;
    let mut in_double_quote = false;
    let mut in_single_quote = false;
    let mut current_start = 0;

    // Use char_indices for proper UTF-8 handling
    for (byte_pos, c) in input.char_indices() {
        match c {
            '"' if !in_single_quote => in_double_quote = !in_double_quote,
            '\'' if !in_double_quote => in_single_quote = !in_single_quote,
            '(' if !in_double_quote && !in_single_quote => paren_depth += 1,
            ')' if !in_double_quote && !in_single_quote => paren_depth -= 1,
            '[' if !in_double_quote && !in_single_quote => bracket_depth += 1,
            ']' if !in_double_quote && !in_single_quote => bracket_depth -= 1,
            ',' if paren_depth == 0 && bracket_depth == 0 && !in_double_quote && !in_single_quote => {
                args.push(parse_expr(&input[current_start..byte_pos])?);
                current_start = byte_pos + c.len_utf8();
            }
            _ => {}
        }
    }

    args.push(parse_expr(&input[current_start..])?);
    Ok(args)
}
