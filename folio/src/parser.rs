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
        
        // Table separator
        if line.starts_with('|') && line.contains('-') && in_table && table_rows.is_empty() {
            continue;
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

    // Contains operators (but not just a negative number)
    if text.contains('+') || text.contains('*') || text.contains('/') || text.contains('^') {
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
    
    parse_additive(input)
}

fn parse_additive(input: &str) -> Result<Expr, FolioError> {
    // Find + or - not inside parentheses or function calls
    let mut depth = 0;

    // Collect (byte_offset, char) pairs to handle multi-byte UTF-8 correctly
    let char_indices: Vec<(usize, char)> = input.char_indices().collect();

    for idx in (0..char_indices.len()).rev() {
        let (byte_pos, c) = char_indices[idx];
        match c {
            ')' => depth += 1,
            '(' => depth -= 1,
            '+' | '-' if depth == 0 && idx > 0 => {
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
    let mut depth = 0;

    // Collect (byte_offset, char) pairs to handle multi-byte UTF-8 correctly
    let char_indices: Vec<(usize, char)> = input.char_indices().collect();

    for idx in (0..char_indices.len()).rev() {
        let (byte_pos, c) = char_indices[idx];
        match c {
            ')' => depth += 1,
            '(' => depth -= 1,
            '*' | '/' if depth == 0 => {
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
    let mut depth = 0;

    // Collect (byte_offset, char) pairs to handle multi-byte UTF-8 correctly
    let char_indices: Vec<(usize, char)> = input.char_indices().collect();

    for idx in 0..char_indices.len() {
        let (byte_pos, c) = char_indices[idx];
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            '^' if depth == 0 => {
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
    
    // Parentheses
    if input.starts_with('(') && input.ends_with(')') {
        return parse_expr(&input[1..input.len()-1]);
    }
    
    // Function call
    if let Some(paren_pos) = input.find('(') {
        if input.ends_with(')') {
            let func_name = input[..paren_pos].trim().to_string();
            let args_str = &input[paren_pos+1..input.len()-1];
            let args = parse_args(args_str)?;
            return Ok(Expr::FunctionCall(func_name, args));
        }
    }
    
    // Number
    if input.chars().next().map_or(false, |c| c.is_ascii_digit() || c == '-' || c == '.') {
        if input.parse::<f64>().is_ok() || input.contains('/') {
            return Ok(Expr::Number(input.to_string()));
        }
    }
    
    // Variable (possibly dotted)
    let parts: Vec<String> = input.split('.').map(|s| s.trim().to_string()).collect();
    Ok(Expr::Variable(parts))
}

fn parse_args(input: &str) -> Result<Vec<Expr>, FolioError> {
    if input.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut args = Vec::new();
    let mut depth = 0;
    let mut current_start = 0;

    // Use char_indices for proper UTF-8 handling
    for (byte_pos, c) in input.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                args.push(parse_expr(&input[current_start..byte_pos])?);
                current_start = byte_pos + c.len_utf8();
            }
            _ => {}
        }
    }

    args.push(parse_expr(&input[current_start..])?);
    Ok(args)
}
