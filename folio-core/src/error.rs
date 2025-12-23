//! Structured errors for LLM consumption
//!
//! Errors never crash the system. They are values that propagate through
//! computations and provide clear, actionable information.

use crate::{NumberError, DateTimeError};
use serde::{Deserialize, Serialize};

/// Standard error codes (machine-readable)
pub mod codes {
    pub const PARSE_ERROR: &str = "PARSE_ERROR";
    pub const DIV_ZERO: &str = "DIV_ZERO";
    pub const UNDEFINED_VAR: &str = "UNDEFINED_VAR";
    pub const UNDEFINED_FUNC: &str = "UNDEFINED_FUNC";
    pub const UNDEFINED_FIELD: &str = "UNDEFINED_FIELD";
    pub const TYPE_ERROR: &str = "TYPE_ERROR";
    pub const ARG_COUNT: &str = "ARG_COUNT";
    pub const ARG_TYPE: &str = "ARG_TYPE";
    pub const DOMAIN_ERROR: &str = "DOMAIN_ERROR";
    pub const OVERFLOW: &str = "OVERFLOW";
    pub const CIRCULAR_REF: &str = "CIRCULAR_REF";
    pub const INTERNAL: &str = "INTERNAL";
    // DateTime-specific error codes
    pub const INVALID_DATE: &str = "INVALID_DATE";
    pub const INVALID_TIME: &str = "INVALID_TIME";
    pub const DATE_OVERFLOW: &str = "DATE_OVERFLOW";
    pub const DATE_PARSE_ERROR: &str = "DATE_PARSE_ERROR";
}

/// Severity level of an error
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Computation continued with degraded result
    Warning,
    /// Computation failed for this cell
    Error,
    /// Document cannot be evaluated
    Fatal,
}

/// Context about where an error occurred
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Cell name where error occurred
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell: Option<String>,
    
    /// Formula that caused the error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formula: Option<String>,
    
    /// Line number in document
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    
    /// Column number in document
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
    
    /// Propagation notes
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub notes: Vec<String>,
}

/// Structured error for LLM consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolioError {
    /// Machine-readable error code
    pub code: String,
    
    /// Human-readable error message
    pub message: String,
    
    /// Suggestion for fixing the error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    
    /// Where the error occurred
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<ErrorContext>,
    
    /// Severity level
    pub severity: Severity,
}

impl FolioError {
    /// Create a new error
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            suggestion: None,
            context: None,
            severity: Severity::Error,
        }
    }
    
    /// Builder: add suggestion
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
    
    /// Builder: add context
    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.context = Some(context);
        self
    }
    
    /// Builder: set cell context
    pub fn in_cell(mut self, cell: impl Into<String>) -> Self {
        let ctx = self.context.get_or_insert_with(ErrorContext::default);
        ctx.cell = Some(cell.into());
        self
    }
    
    /// Builder: set formula context
    pub fn with_formula(mut self, formula: impl Into<String>) -> Self {
        let ctx = self.context.get_or_insert_with(ErrorContext::default);
        ctx.formula = Some(formula.into());
        self
    }
    
    /// Builder: add propagation note
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        let ctx = self.context.get_or_insert_with(ErrorContext::default);
        ctx.notes.push(note.into());
        self
    }
    
    /// Builder: set severity
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }
    
    // ========== Common Error Constructors ==========
    
    pub fn parse_error(details: impl Into<String>) -> Self {
        Self::new(codes::PARSE_ERROR, format!("Parse error: {}", details.into()))
            .with_suggestion("Check formula syntax")
    }
    
    pub fn div_zero() -> Self {
        Self::new(codes::DIV_ZERO, "Division by zero")
            .with_suggestion("Ensure divisor is not zero")
    }
    
    pub fn undefined_var(name: &str) -> Self {
        Self::new(codes::UNDEFINED_VAR, format!("Undefined variable: {}", name))
            .with_suggestion(format!("Define '{}' or check spelling", name))
    }
    
    pub fn undefined_func(name: &str) -> Self {
        Self::new(codes::UNDEFINED_FUNC, format!("Unknown function: {}", name))
            .with_suggestion("Use folio() to list available functions")
    }

    pub fn undefined_field(name: &str) -> Self {
        Self::new(codes::UNDEFINED_FIELD, format!("Undefined field: {}", name))
            .with_suggestion("Check object structure with folio()")
    }
    
    pub fn type_error(expected: &str, got: &str) -> Self {
        Self::new(codes::TYPE_ERROR, format!("Expected {}, got {}", expected, got))
            .with_suggestion(format!("Convert value to {} or check formula", expected))
    }
    
    pub fn arg_count(func: &str, expected: usize, got: usize) -> Self {
        Self::new(codes::ARG_COUNT, 
            format!("{}() expects {} arguments, got {}", func, expected, got))
            .with_suggestion(format!("Use help('{}') for usage", func))
    }
    
    pub fn arg_type(func: &str, arg: &str, expected: &str, got: &str) -> Self {
        Self::new(codes::ARG_TYPE,
            format!("{}() argument '{}': expected {}, got {}", func, arg, expected, got))
    }
    
    pub fn domain_error(details: impl Into<String>) -> Self {
        Self::new(codes::DOMAIN_ERROR, format!("Domain error: {}", details.into()))
    }
    
    pub fn circular_ref(cells: &[String]) -> Self {
        Self::new(codes::CIRCULAR_REF, 
            format!("Circular reference: {}", cells.join(" → ")))
            .with_suggestion("Remove circular dependency")
            .with_severity(Severity::Fatal)
    }
    
    pub fn internal(details: impl Into<String>) -> Self {
        Self::new(codes::INTERNAL, format!("Internal error: {}", details.into()))
            .with_suggestion("This is a bug, please report it")
            .with_severity(Severity::Fatal)
    }

    // ========== DateTime Error Constructors ==========

    pub fn invalid_date(details: impl Into<String>) -> Self {
        Self::new(codes::INVALID_DATE, format!("Invalid date: {}", details.into()))
            .with_suggestion("Check date components (year, month 1-12, day 1-31)")
    }

    pub fn invalid_time(details: impl Into<String>) -> Self {
        Self::new(codes::INVALID_TIME, format!("Invalid time: {}", details.into()))
            .with_suggestion("Check time components (hour 0-23, minute 0-59, second 0-59)")
    }

    pub fn date_overflow() -> Self {
        Self::new(codes::DATE_OVERFLOW, "DateTime overflow")
            .with_suggestion("Date value is out of supported range")
    }

    pub fn date_parse_error(details: impl Into<String>) -> Self {
        Self::new(codes::DATE_PARSE_ERROR, format!("DateTime parse error: {}", details.into()))
            .with_suggestion("Use ISO 8601 format (YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS)")
    }
}

impl std::fmt::Display for FolioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;
        if let Some(ref suggestion) = self.suggestion {
            write!(f, " (suggestion: {})", suggestion)?;
        }
        Ok(())
    }
}

impl std::error::Error for FolioError {}

impl From<NumberError> for FolioError {
    fn from(err: NumberError) -> Self {
        match err {
            NumberError::ParseError(s) => Self::parse_error(s),
            NumberError::DivisionByZero => Self::div_zero(),
            NumberError::DomainError(s) => Self::domain_error(s),
            NumberError::Overflow => Self::new(codes::OVERFLOW, "Numeric overflow"),
        }
    }
}

impl From<DateTimeError> for FolioError {
    fn from(err: DateTimeError) -> Self {
        match err {
            DateTimeError::InvalidMonth(m) => Self::invalid_date(format!("month {} out of range 1-12", m)),
            DateTimeError::InvalidDay(d, m, y) => Self::invalid_date(format!("day {} invalid for {}/{}", d, m, y)),
            DateTimeError::InvalidHour(h) => Self::invalid_time(format!("hour {} out of range 0-23", h)),
            DateTimeError::InvalidMinute(m) => Self::invalid_time(format!("minute {} out of range 0-59", m)),
            DateTimeError::InvalidSecond(s) => Self::invalid_time(format!("second {} out of range 0-59", s)),
            DateTimeError::InvalidNano(n) => Self::invalid_time(format!("nanosecond {} out of range", n)),
            DateTimeError::ParseError(s) => Self::date_parse_error(s),
            DateTimeError::Overflow => Self::date_overflow(),
        }
    }
}
