//! Markdown renderer
//!
//! Renders evaluated document back to markdown with results.

use crate::ast::Document;
use folio_core::Value;
use std::collections::HashMap;

/// Display format for numbers
#[derive(Clone, Copy)]
pub enum NumberFormat {
    /// Fixed decimal places (default)
    Decimal(u32),
    /// Significant figures with scientific notation for large/small values
    SigFigs(u32),
}

impl Default for NumberFormat {
    fn default() -> Self {
        NumberFormat::Decimal(10)
    }
}

/// Display formats for datetime values
#[derive(Clone, Default)]
pub struct DateTimeFormats {
    /// Format for Date values (default: YYYY-MM-DD)
    pub date_fmt: Option<String>,
    /// Format for Time values (default: HH:mm:ss)
    pub time_fmt: Option<String>,
    /// Format for DateTime values (default: ISO 8601)
    pub datetime_fmt: Option<String>,
}

/// Document renderer
pub struct Renderer;

impl Renderer {
    pub fn new() -> Self {
        Self
    }

    /// Render document with computed values
    pub fn render(
        &self,
        doc: &Document,
        values: &HashMap<String, Value>,
        external: &HashMap<String, Value>,
    ) -> String {
        let mut output = String::new();

        // Render external variables section if any
        if !external.is_empty() {
            output.push_str("## External Variables\n\n");
            output.push_str("| name | value |\n");
            output.push_str("|------|-------|\n");
            let default_dt_formats = DateTimeFormats::default();
            for (name, value) in external {
                output.push_str(&format!("| {} | {} |\n", name, self.render_value(value, NumberFormat::default(), &default_dt_formats)));
            }
            output.push('\n');
        }

        // Render each section
        for section in &doc.sections {
            output.push_str(&format!("## {}", section.name));

            // Add attributes if any
            if !section.attributes.is_empty() {
                let attrs: Vec<String> = section.attributes
                    .iter()
                    .map(|(k, v)| format!("{}:{}", k, v))
                    .collect();
                output.push_str(&format!(" @{}", attrs.join(",")));
            }
            output.push_str("\n\n");

            // Determine formats from section attributes
            let num_format = self.get_number_format(&section.attributes);
            let dt_formats = self.get_datetime_formats(&section.attributes);

            // Render table header
            output.push_str("| name | formula | result |\n");
            output.push_str("|------|---------|--------|\n");

            // Rows
            for row in &section.table.rows {
                for cell in &row.cells {
                    let result = values.get(&cell.name)
                        .map(|v| self.render_value(v, num_format, &dt_formats))
                        .unwrap_or_default();
                    output.push_str(&format!("| {} | {} | {} |\n",
                        cell.name, cell.raw_text, result));
                }
            }

            output.push('\n');
        }

        output
    }

    /// Get number format from section attributes
    fn get_number_format(&self, attrs: &HashMap<String, String>) -> NumberFormat {
        // Check for @sigfigs first (takes precedence)
        if let Some(sigfigs) = attrs.get("sigfigs") {
            if let Ok(n) = sigfigs.parse::<u32>() {
                return NumberFormat::SigFigs(n);
            }
        }
        // Fall back to decimal places (default 10)
        NumberFormat::Decimal(10)
    }

    /// Get datetime formats from section attributes
    fn get_datetime_formats(&self, attrs: &HashMap<String, String>) -> DateTimeFormats {
        DateTimeFormats {
            date_fmt: attrs.get("dateFmt").cloned(),
            time_fmt: attrs.get("timeFmt").cloned(),
            datetime_fmt: attrs.get("datetimeFmt").cloned(),
        }
    }

    fn render_value(&self, value: &Value, num_format: NumberFormat, dt_formats: &DateTimeFormats) -> String {
        match value {
            Value::Number(n) => match num_format {
                NumberFormat::Decimal(places) => n.as_decimal(places),
                NumberFormat::SigFigs(sigfigs) => n.as_sigfigs(sigfigs),
            },
            Value::Text(s) => s.clone(),
            Value::Bool(b) => b.to_string(),
            Value::DateTime(dt) => {
                // Use section datetime format if specified
                if let Some(ref fmt) = dt_formats.datetime_fmt {
                    dt.format(fmt)
                } else if let Some(ref fmt) = dt_formats.date_fmt {
                    // If only date format is specified, use it
                    dt.format(fmt)
                } else {
                    dt.to_string()
                }
            },
            Value::Duration(d) => d.to_string(),
            Value::Object(_) => "[Object]".to_string(),
            Value::List(l) => format!("[{}]", l.len()),
            Value::Null => "null".to_string(),
            Value::Error(e) => format!("#ERROR: {}", e.code),
        }
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
