//! DateTime and Duration functions
//!
//! Provides functions for constructing, parsing, extracting, formatting,
//! and manipulating dates, times, and durations.

use folio_plugin::prelude::*;

// ============================================================================
// Construction Functions
// ============================================================================

pub struct DateFn;
pub struct TimeFn;
pub struct DateTimeFn;
pub struct NowFn;

// Date construction
static DATE_ARGS: [ArgMeta; 3] = [
    ArgMeta { name: "year", typ: "Number", description: "Year (e.g., 2025)", optional: false, default: None },
    ArgMeta { name: "month", typ: "Number", description: "Month (1-12)", optional: false, default: None },
    ArgMeta { name: "day", typ: "Number", description: "Day (1-31)", optional: false, default: None },
];
static DATE_EXAMPLES: [&str; 2] = ["date(2025, 6, 15)", "date(1999, 12, 31)"];
static DATE_RELATED: [&str; 3] = ["datetime", "time", "now"];

// Time construction
static TIME_ARGS: [ArgMeta; 3] = [
    ArgMeta { name: "hour", typ: "Number", description: "Hour (0-23)", optional: false, default: None },
    ArgMeta { name: "minute", typ: "Number", description: "Minute (0-59)", optional: false, default: None },
    ArgMeta { name: "second", typ: "Number", description: "Second (0-59)", optional: true, default: Some("0") },
];
static TIME_EXAMPLES: [&str; 2] = ["time(14, 30)", "time(9, 0, 30)"];
static TIME_RELATED: [&str; 2] = ["date", "datetime"];

// DateTime construction
static DATETIME_ARGS: [ArgMeta; 6] = [
    ArgMeta { name: "year", typ: "Number", description: "Year", optional: false, default: None },
    ArgMeta { name: "month", typ: "Number", description: "Month (1-12)", optional: false, default: None },
    ArgMeta { name: "day", typ: "Number", description: "Day (1-31)", optional: false, default: None },
    ArgMeta { name: "hour", typ: "Number", description: "Hour (0-23)", optional: false, default: None },
    ArgMeta { name: "minute", typ: "Number", description: "Minute (0-59)", optional: false, default: None },
    ArgMeta { name: "second", typ: "Number", description: "Second (0-59)", optional: true, default: Some("0") },
];
static DATETIME_EXAMPLES: [&str; 2] = ["datetime(2025, 6, 15, 14, 30, 0)", "datetime(2025, 1, 1, 0, 0)"];
static DATETIME_RELATED: [&str; 2] = ["date", "time"];

// Now
static NOW_ARGS: [ArgMeta; 0] = [];
static NOW_EXAMPLES: [&str; 1] = ["now()"];
static NOW_RELATED: [&str; 2] = ["date", "datetime"];

impl FunctionPlugin for DateFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "date",
            description: "Create a date (time = 00:00:00)",
            usage: "date(year, month, day)",
            args: &DATE_ARGS,
            returns: "DateTime",
            examples: &DATE_EXAMPLES,
            category: "datetime",
            source: None,
            related: &DATE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 3 {
            return Value::Error(FolioError::arg_count("date", 3, args.len()));
        }

        let year = match get_i32(&args[0], "date", "year") {
            Ok(v) => v,
            Err(e) => return e,
        };
        let month = match get_u32(&args[1], "date", "month") {
            Ok(v) => v,
            Err(e) => return e,
        };
        let day = match get_u32(&args[2], "date", "day") {
            Ok(v) => v,
            Err(e) => return e,
        };

        match FolioDateTime::from_ymd(year, month, day) {
            Ok(dt) => Value::DateTime(dt),
            Err(e) => Value::Error(e.into()),
        }
    }
}

impl FunctionPlugin for TimeFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "time",
            description: "Create a time (date = 1970-01-01)",
            usage: "time(hour, minute, second?)",
            args: &TIME_ARGS,
            returns: "DateTime",
            examples: &TIME_EXAMPLES,
            category: "datetime",
            source: None,
            related: &TIME_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 || args.len() > 3 {
            return Value::Error(FolioError::arg_count("time", 2, args.len())
                .with_note("time(hour, minute, second?)"));
        }

        let hour = match get_u32(&args[0], "time", "hour") {
            Ok(v) => v,
            Err(e) => return e,
        };
        let minute = match get_u32(&args[1], "time", "minute") {
            Ok(v) => v,
            Err(e) => return e,
        };
        let second = if args.len() > 2 {
            match get_u32(&args[2], "time", "second") {
                Ok(v) => v,
                Err(e) => return e,
            }
        } else {
            0
        };

        match FolioDateTime::from_hms(hour, minute, second) {
            Ok(dt) => Value::DateTime(dt),
            Err(e) => Value::Error(e.into()),
        }
    }
}

impl FunctionPlugin for DateTimeFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "datetime",
            description: "Create a datetime from components",
            usage: "datetime(year, month, day, hour, minute, second?)",
            args: &DATETIME_ARGS,
            returns: "DateTime",
            examples: &DATETIME_EXAMPLES,
            category: "datetime",
            source: None,
            related: &DATETIME_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 5 || args.len() > 6 {
            return Value::Error(FolioError::arg_count("datetime", 5, args.len())
                .with_note("datetime(year, month, day, hour, minute, second?)"));
        }

        let year = match get_i32(&args[0], "datetime", "year") {
            Ok(v) => v,
            Err(e) => return e,
        };
        let month = match get_u32(&args[1], "datetime", "month") {
            Ok(v) => v,
            Err(e) => return e,
        };
        let day = match get_u32(&args[2], "datetime", "day") {
            Ok(v) => v,
            Err(e) => return e,
        };
        let hour = match get_u32(&args[3], "datetime", "hour") {
            Ok(v) => v,
            Err(e) => return e,
        };
        let minute = match get_u32(&args[4], "datetime", "minute") {
            Ok(v) => v,
            Err(e) => return e,
        };
        let second = if args.len() > 5 {
            match get_u32(&args[5], "datetime", "second") {
                Ok(v) => v,
                Err(e) => return e,
            }
        } else {
            0
        };

        match FolioDateTime::from_ymd_hms(year, month, day, hour, minute, second) {
            Ok(dt) => Value::DateTime(dt),
            Err(e) => Value::Error(e.into()),
        }
    }
}

impl FunctionPlugin for NowFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "now",
            description: "Get current UTC datetime",
            usage: "now()",
            args: &NOW_ARGS,
            returns: "DateTime",
            examples: &NOW_EXAMPLES,
            category: "datetime",
            source: None,
            related: &NOW_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if !args.is_empty() {
            return Value::Error(FolioError::arg_count("now", 0, args.len()));
        }
        Value::DateTime(FolioDateTime::now())
    }
}

// ============================================================================
// Parsing Functions
// ============================================================================

pub struct ParseDateFn;
pub struct ParseTimeFn;

static PARSEDATE_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "text", typ: "Text", description: "Date string to parse", optional: false, default: None },
    ArgMeta { name: "format", typ: "Text", description: "Optional format pattern", optional: true, default: None },
];
static PARSEDATE_EXAMPLES: [&str; 3] = ["parseDate(\"2025-06-15\")", "parseDate(\"15/06/2025\", \"DD/MM/YYYY\")", "parseDate(\"2025-06-15T14:30:00Z\")"];
static PARSEDATE_RELATED: [&str; 2] = ["date", "formatDate"];

static PARSETIME_ARGS: [ArgMeta; 1] = [
    ArgMeta { name: "text", typ: "Text", description: "Time string to parse (HH:MM:SS)", optional: false, default: None },
];
static PARSETIME_EXAMPLES: [&str; 2] = ["parseTime(\"14:30\")", "parseTime(\"09:00:30\")"];
static PARSETIME_RELATED: [&str; 2] = ["time", "formatTime"];

impl FunctionPlugin for ParseDateFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "parseDate",
            description: "Parse a date/datetime string (ISO 8601 or custom format)",
            usage: "parseDate(text, format?)",
            args: &PARSEDATE_ARGS,
            returns: "DateTime",
            examples: &PARSEDATE_EXAMPLES,
            category: "datetime",
            source: None,
            related: &PARSEDATE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("parseDate", 1, args.len()));
        }

        let text = match &args[0] {
            Value::Text(s) => s,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("parseDate", "text", "Text", other.type_name())),
        };

        if args.len() == 2 {
            // Parse with format
            let format = match &args[1] {
                Value::Text(s) => s,
                Value::Error(e) => return Value::Error(e.clone()),
                other => return Value::Error(FolioError::arg_type("parseDate", "format", "Text", other.type_name())),
            };
            match FolioDateTime::parse_format(text, format) {
                Ok(dt) => Value::DateTime(dt),
                Err(e) => Value::Error(e.into()),
            }
        } else {
            // Auto-detect format
            match FolioDateTime::parse(text) {
                Ok(dt) => Value::DateTime(dt),
                Err(e) => Value::Error(e.into()),
            }
        }
    }
}

impl FunctionPlugin for ParseTimeFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "parseTime",
            description: "Parse a time string (HH:MM or HH:MM:SS)",
            usage: "parseTime(text)",
            args: &PARSETIME_ARGS,
            returns: "DateTime",
            examples: &PARSETIME_EXAMPLES,
            category: "datetime",
            source: None,
            related: &PARSETIME_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("parseTime", 1, args.len()));
        }

        let text = match &args[0] {
            Value::Text(s) => s,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("parseTime", "text", "Text", other.type_name())),
        };

        match FolioDateTime::parse_time(text) {
            Ok(dt) => Value::DateTime(dt),
            Err(e) => Value::Error(e.into()),
        }
    }
}

// ============================================================================
// Extraction Functions
// ============================================================================

pub struct YearFn;
pub struct MonthFn;
pub struct DayFn;
pub struct HourFn;
pub struct MinuteFn;
pub struct SecondFn;
pub struct WeekdayFn;
pub struct DayOfYearFn;
pub struct WeekFn;

static EXTRACT_DT_ARGS: [ArgMeta; 1] = [
    ArgMeta { name: "dt", typ: "DateTime", description: "DateTime to extract from", optional: false, default: None },
];
static EXTRACT_EXAMPLES: [&str; 1] = ["year(now())"];
static EXTRACT_RELATED: [&str; 2] = ["date", "datetime"];

impl FunctionPlugin for YearFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "year",
            description: "Extract year from datetime",
            usage: "year(dt)",
            args: &EXTRACT_DT_ARGS,
            returns: "Number",
            examples: &EXTRACT_EXAMPLES,
            category: "datetime",
            source: None,
            related: &EXTRACT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("year", 1, args.len()));
        }
        match &args[0] {
            Value::DateTime(dt) => Value::Number(Number::from_i64(dt.year() as i64)),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("year", "dt", "DateTime", other.type_name())),
        }
    }
}

impl FunctionPlugin for MonthFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "month",
            description: "Extract month from datetime (1-12)",
            usage: "month(dt)",
            args: &EXTRACT_DT_ARGS,
            returns: "Number",
            examples: &["month(now())"],
            category: "datetime",
            source: None,
            related: &EXTRACT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("month", 1, args.len()));
        }
        match &args[0] {
            Value::DateTime(dt) => Value::Number(Number::from_i64(dt.month() as i64)),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("month", "dt", "DateTime", other.type_name())),
        }
    }
}

impl FunctionPlugin for DayFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "day",
            description: "Extract day from datetime (1-31)",
            usage: "day(dt)",
            args: &EXTRACT_DT_ARGS,
            returns: "Number",
            examples: &["day(now())"],
            category: "datetime",
            source: None,
            related: &EXTRACT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("day", 1, args.len()));
        }
        match &args[0] {
            Value::DateTime(dt) => Value::Number(Number::from_i64(dt.day() as i64)),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("day", "dt", "DateTime", other.type_name())),
        }
    }
}

impl FunctionPlugin for HourFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "hour",
            description: "Extract hour from datetime (0-23)",
            usage: "hour(dt)",
            args: &EXTRACT_DT_ARGS,
            returns: "Number",
            examples: &["hour(now())"],
            category: "datetime",
            source: None,
            related: &EXTRACT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("hour", 1, args.len()));
        }
        match &args[0] {
            Value::DateTime(dt) => Value::Number(Number::from_i64(dt.hour() as i64)),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("hour", "dt", "DateTime", other.type_name())),
        }
    }
}

impl FunctionPlugin for MinuteFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "minute",
            description: "Extract minute from datetime (0-59)",
            usage: "minute(dt)",
            args: &EXTRACT_DT_ARGS,
            returns: "Number",
            examples: &["minute(now())"],
            category: "datetime",
            source: None,
            related: &EXTRACT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("minute", 1, args.len()));
        }
        match &args[0] {
            Value::DateTime(dt) => Value::Number(Number::from_i64(dt.minute() as i64)),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("minute", "dt", "DateTime", other.type_name())),
        }
    }
}

impl FunctionPlugin for SecondFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "second",
            description: "Extract second from datetime (0-59)",
            usage: "second(dt)",
            args: &EXTRACT_DT_ARGS,
            returns: "Number",
            examples: &["second(now())"],
            category: "datetime",
            source: None,
            related: &EXTRACT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("second", 1, args.len()));
        }
        match &args[0] {
            Value::DateTime(dt) => Value::Number(Number::from_i64(dt.second() as i64)),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("second", "dt", "DateTime", other.type_name())),
        }
    }
}

impl FunctionPlugin for WeekdayFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "weekday",
            description: "Get day of week (1=Monday, 7=Sunday)",
            usage: "weekday(dt)",
            args: &EXTRACT_DT_ARGS,
            returns: "Number",
            examples: &["weekday(now())"],
            category: "datetime",
            source: None,
            related: &EXTRACT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("weekday", 1, args.len()));
        }
        match &args[0] {
            Value::DateTime(dt) => Value::Number(Number::from_i64(dt.weekday() as i64)),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("weekday", "dt", "DateTime", other.type_name())),
        }
    }
}

impl FunctionPlugin for DayOfYearFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "dayOfYear",
            description: "Get day of year (1-366)",
            usage: "dayOfYear(dt)",
            args: &EXTRACT_DT_ARGS,
            returns: "Number",
            examples: &["dayOfYear(now())"],
            category: "datetime",
            source: None,
            related: &EXTRACT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("dayOfYear", 1, args.len()));
        }
        match &args[0] {
            Value::DateTime(dt) => Value::Number(Number::from_i64(dt.day_of_year() as i64)),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("dayOfYear", "dt", "DateTime", other.type_name())),
        }
    }
}

impl FunctionPlugin for WeekFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "week",
            description: "Get ISO week number (1-53)",
            usage: "week(dt)",
            args: &EXTRACT_DT_ARGS,
            returns: "Number",
            examples: &["week(now())"],
            category: "datetime",
            source: None,
            related: &EXTRACT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("week", 1, args.len()));
        }
        match &args[0] {
            Value::DateTime(dt) => Value::Number(Number::from_i64(dt.iso_week() as i64)),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("week", "dt", "DateTime", other.type_name())),
        }
    }
}

// ============================================================================
// Formatting Functions
// ============================================================================

pub struct FormatDateFn;
pub struct FormatTimeFn;
pub struct FormatDateTimeFn;

static FORMATDATE_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "dt", typ: "DateTime", description: "DateTime to format", optional: false, default: None },
    ArgMeta { name: "pattern", typ: "Text", description: "Format pattern (e.g., DD/MM/YYYY)", optional: true, default: Some("YYYY-MM-DD") },
];
static FORMATDATE_EXAMPLES: [&str; 2] = ["formatDate(now())", "formatDate(now(), \"DD/MM/YYYY\")"];
static FORMATDATE_RELATED: [&str; 2] = ["parseDate", "formatDateTime"];

static FORMATTIME_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "dt", typ: "DateTime", description: "DateTime to format", optional: false, default: None },
    ArgMeta { name: "pattern", typ: "Text", description: "Format pattern (e.g., HH:mm:ss)", optional: true, default: Some("HH:mm:ss") },
];
static FORMATTIME_EXAMPLES: [&str; 2] = ["formatTime(now())", "formatTime(now(), \"h:mm A\")"];
static FORMATTIME_RELATED: [&str; 2] = ["parseTime", "formatDateTime"];

static FORMATDATETIME_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "dt", typ: "DateTime", description: "DateTime to format", optional: false, default: None },
    ArgMeta { name: "pattern", typ: "Text", description: "Format pattern", optional: true, default: Some("YYYY-MM-DDTHH:mm:ss") },
];
static FORMATDATETIME_EXAMPLES: [&str; 2] = ["formatDateTime(now())", "formatDateTime(now(), \"DD/MM/YYYY HH:mm\")"];
static FORMATDATETIME_RELATED: [&str; 2] = ["formatDate", "formatTime"];

impl FunctionPlugin for FormatDateFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "formatDate",
            description: "Format datetime as date string",
            usage: "formatDate(dt, pattern?)",
            args: &FORMATDATE_ARGS,
            returns: "Text",
            examples: &FORMATDATE_EXAMPLES,
            category: "datetime",
            source: None,
            related: &FORMATDATE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("formatDate", 1, args.len()));
        }

        let dt = match &args[0] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("formatDate", "dt", "DateTime", other.type_name())),
        };

        let pattern = if args.len() > 1 {
            match &args[1] {
                Value::Text(s) => s.as_str(),
                Value::Error(e) => return Value::Error(e.clone()),
                other => return Value::Error(FolioError::arg_type("formatDate", "pattern", "Text", other.type_name())),
            }
        } else {
            "YYYY-MM-DD"
        };

        Value::Text(dt.format(pattern))
    }
}

impl FunctionPlugin for FormatTimeFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "formatTime",
            description: "Format datetime as time string",
            usage: "formatTime(dt, pattern?)",
            args: &FORMATTIME_ARGS,
            returns: "Text",
            examples: &FORMATTIME_EXAMPLES,
            category: "datetime",
            source: None,
            related: &FORMATTIME_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("formatTime", 1, args.len()));
        }

        let dt = match &args[0] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("formatTime", "dt", "DateTime", other.type_name())),
        };

        let pattern = if args.len() > 1 {
            match &args[1] {
                Value::Text(s) => s.as_str(),
                Value::Error(e) => return Value::Error(e.clone()),
                other => return Value::Error(FolioError::arg_type("formatTime", "pattern", "Text", other.type_name())),
            }
        } else {
            "HH:mm:ss"
        };

        Value::Text(dt.format(pattern))
    }
}

impl FunctionPlugin for FormatDateTimeFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "formatDateTime",
            description: "Format datetime with pattern",
            usage: "formatDateTime(dt, pattern?)",
            args: &FORMATDATETIME_ARGS,
            returns: "Text",
            examples: &FORMATDATETIME_EXAMPLES,
            category: "datetime",
            source: None,
            related: &FORMATDATETIME_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("formatDateTime", 1, args.len()));
        }

        let dt = match &args[0] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("formatDateTime", "dt", "DateTime", other.type_name())),
        };

        let pattern = if args.len() > 1 {
            match &args[1] {
                Value::Text(s) => s.as_str(),
                Value::Error(e) => return Value::Error(e.clone()),
                other => return Value::Error(FolioError::arg_type("formatDateTime", "pattern", "Text", other.type_name())),
            }
        } else {
            "YYYY-MM-DDTHH:mm:ss"
        };

        Value::Text(dt.format(pattern))
    }
}

// ============================================================================
// Duration Construction Functions
// ============================================================================

pub struct WeeksDur;
pub struct DaysDur;
pub struct HoursDur;
pub struct MinutesDur;
pub struct SecondsDur;
pub struct MillisecondsDur;

static DUR_N_ARGS: [ArgMeta; 1] = [
    ArgMeta { name: "n", typ: "Number", description: "Number of units", optional: false, default: None },
];
static DUR_RELATED: [&str; 6] = ["weeks", "days", "hours", "minutes", "seconds", "milliseconds"];

impl FunctionPlugin for WeeksDur {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "weeks",
            description: "Create a duration of n weeks",
            usage: "weeks(n)",
            args: &DUR_N_ARGS,
            returns: "Duration",
            examples: &["weeks(2)"],
            category: "datetime",
            source: None,
            related: &DUR_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("weeks", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => {
                if let Some(v) = n.to_i64() {
                    Value::Duration(FolioDuration::from_weeks(v))
                } else {
                    Value::Error(FolioError::domain_error("value too large for duration"))
                }
            }
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("weeks", "n", "Number", other.type_name())),
        }
    }
}

impl FunctionPlugin for DaysDur {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "days",
            description: "Create a duration of n days",
            usage: "days(n)",
            args: &DUR_N_ARGS,
            returns: "Duration",
            examples: &["days(5)"],
            category: "datetime",
            source: None,
            related: &DUR_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("days", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => {
                if let Some(v) = n.to_i64() {
                    Value::Duration(FolioDuration::from_days(v))
                } else {
                    Value::Error(FolioError::domain_error("value too large for duration"))
                }
            }
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("days", "n", "Number", other.type_name())),
        }
    }
}

impl FunctionPlugin for HoursDur {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "hours",
            description: "Create a duration of n hours",
            usage: "hours(n)",
            args: &DUR_N_ARGS,
            returns: "Duration",
            examples: &["hours(24)"],
            category: "datetime",
            source: None,
            related: &DUR_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("hours", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => {
                if let Some(v) = n.to_i64() {
                    Value::Duration(FolioDuration::from_hours(v))
                } else {
                    Value::Error(FolioError::domain_error("value too large for duration"))
                }
            }
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("hours", "n", "Number", other.type_name())),
        }
    }
}

impl FunctionPlugin for MinutesDur {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "minutes",
            description: "Create a duration of n minutes",
            usage: "minutes(n)",
            args: &DUR_N_ARGS,
            returns: "Duration",
            examples: &["minutes(30)"],
            category: "datetime",
            source: None,
            related: &DUR_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("minutes", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => {
                if let Some(v) = n.to_i64() {
                    Value::Duration(FolioDuration::from_minutes(v))
                } else {
                    Value::Error(FolioError::domain_error("value too large for duration"))
                }
            }
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("minutes", "n", "Number", other.type_name())),
        }
    }
}

impl FunctionPlugin for SecondsDur {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "seconds",
            description: "Create a duration of n seconds",
            usage: "seconds(n)",
            args: &DUR_N_ARGS,
            returns: "Duration",
            examples: &["seconds(60)"],
            category: "datetime",
            source: None,
            related: &DUR_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("seconds", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => {
                if let Some(v) = n.to_i64() {
                    Value::Duration(FolioDuration::from_secs(v))
                } else {
                    Value::Error(FolioError::domain_error("value too large for duration"))
                }
            }
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("seconds", "n", "Number", other.type_name())),
        }
    }
}

impl FunctionPlugin for MillisecondsDur {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "milliseconds",
            description: "Create a duration of n milliseconds",
            usage: "milliseconds(n)",
            args: &DUR_N_ARGS,
            returns: "Duration",
            examples: &["milliseconds(500)"],
            category: "datetime",
            source: None,
            related: &DUR_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("milliseconds", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => {
                if let Some(v) = n.to_i64() {
                    Value::Duration(FolioDuration::from_millis(v))
                } else {
                    Value::Error(FolioError::domain_error("value too large for duration"))
                }
            }
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("milliseconds", "n", "Number", other.type_name())),
        }
    }
}

// ============================================================================
// Arithmetic Functions
// ============================================================================

pub struct AddDaysFn;
pub struct AddMonthsFn;
pub struct AddYearsFn;
pub struct DiffFn;

static ADDDAYS_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "dt", typ: "DateTime", description: "Base datetime", optional: false, default: None },
    ArgMeta { name: "n", typ: "Number", description: "Number of days to add", optional: false, default: None },
];
static ADDDAYS_EXAMPLES: [&str; 2] = ["addDays(now(), 30)", "addDays(date(2025, 1, 1), -7)"];
static ADDDAYS_RELATED: [&str; 2] = ["addMonths", "addYears"];

static ADDMONTHS_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "dt", typ: "DateTime", description: "Base datetime", optional: false, default: None },
    ArgMeta { name: "n", typ: "Number", description: "Number of months to add", optional: false, default: None },
];
static ADDMONTHS_EXAMPLES: [&str; 2] = ["addMonths(now(), 3)", "addMonths(date(2025, 1, 31), 1)"];
static ADDMONTHS_RELATED: [&str; 2] = ["addDays", "addYears"];

static ADDYEARS_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "dt", typ: "DateTime", description: "Base datetime", optional: false, default: None },
    ArgMeta { name: "n", typ: "Number", description: "Number of years to add", optional: false, default: None },
];
static ADDYEARS_EXAMPLES: [&str; 2] = ["addYears(now(), 1)", "addYears(date(2020, 2, 29), 1)"];
static ADDYEARS_RELATED: [&str; 2] = ["addDays", "addMonths"];

static DIFF_ARGS: [ArgMeta; 3] = [
    ArgMeta { name: "dt1", typ: "DateTime", description: "First datetime", optional: false, default: None },
    ArgMeta { name: "dt2", typ: "DateTime", description: "Second datetime", optional: false, default: None },
    ArgMeta { name: "unit", typ: "Text", description: "Unit: years, months, weeks, days, hours, minutes, seconds, milliseconds", optional: true, default: Some("days") },
];
static DIFF_EXAMPLES: [&str; 2] = ["diff(now(), date(2025, 1, 1), \"days\")", "diff(dt1, dt2, \"hours\")"];
static DIFF_RELATED: [&str; 2] = ["addDays", "isBefore"];

impl FunctionPlugin for AddDaysFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "addDays",
            description: "Add days to a datetime",
            usage: "addDays(dt, n)",
            args: &ADDDAYS_ARGS,
            returns: "DateTime",
            examples: &ADDDAYS_EXAMPLES,
            category: "datetime",
            source: None,
            related: &ADDDAYS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("addDays", 2, args.len()));
        }

        let dt = match &args[0] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("addDays", "dt", "DateTime", other.type_name())),
        };

        let n = match get_i64(&args[1], "addDays", "n") {
            Ok(v) => v,
            Err(e) => return e,
        };

        Value::DateTime(dt.add_days(n))
    }
}

impl FunctionPlugin for AddMonthsFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "addMonths",
            description: "Add months to a datetime (handles month boundaries)",
            usage: "addMonths(dt, n)",
            args: &ADDMONTHS_ARGS,
            returns: "DateTime",
            examples: &ADDMONTHS_EXAMPLES,
            category: "datetime",
            source: None,
            related: &ADDMONTHS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("addMonths", 2, args.len()));
        }

        let dt = match &args[0] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("addMonths", "dt", "DateTime", other.type_name())),
        };

        let n = match get_i32(&args[1], "addMonths", "n") {
            Ok(v) => v,
            Err(e) => return e,
        };

        Value::DateTime(dt.add_months(n))
    }
}

impl FunctionPlugin for AddYearsFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "addYears",
            description: "Add years to a datetime (handles leap years)",
            usage: "addYears(dt, n)",
            args: &ADDYEARS_ARGS,
            returns: "DateTime",
            examples: &ADDYEARS_EXAMPLES,
            category: "datetime",
            source: None,
            related: &ADDYEARS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("addYears", 2, args.len()));
        }

        let dt = match &args[0] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("addYears", "dt", "DateTime", other.type_name())),
        };

        let n = match get_i32(&args[1], "addYears", "n") {
            Ok(v) => v,
            Err(e) => return e,
        };

        Value::DateTime(dt.add_years(n))
    }
}

impl FunctionPlugin for DiffFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "diff",
            description: "Calculate difference between two datetimes",
            usage: "diff(dt1, dt2, unit?)",
            args: &DIFF_ARGS,
            returns: "Number",
            examples: &DIFF_EXAMPLES,
            category: "datetime",
            source: None,
            related: &DIFF_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 || args.len() > 3 {
            return Value::Error(FolioError::arg_count("diff", 2, args.len()));
        }

        let dt1 = match &args[0] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("diff", "dt1", "DateTime", other.type_name())),
        };

        let dt2 = match &args[1] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("diff", "dt2", "DateTime", other.type_name())),
        };

        let unit = if args.len() > 2 {
            match &args[2] {
                Value::Text(s) => s.as_str(),
                Value::Error(e) => return Value::Error(e.clone()),
                other => return Value::Error(FolioError::arg_type("diff", "unit", "Text", other.type_name())),
            }
        } else {
            "days"
        };

        let duration = dt1.duration_since(dt2);
        let value = match unit.to_lowercase().as_str() {
            "years" | "year" | "y" => {
                // Calendar year difference
                (dt1.year() - dt2.year()) as i64
            }
            "months" | "month" => {
                // Calendar month difference
                let y1 = dt1.year() as i64;
                let y2 = dt2.year() as i64;
                let m1 = dt1.month() as i64;
                let m2 = dt2.month() as i64;
                (y1 * 12 + m1) - (y2 * 12 + m2)
            }
            "weeks" | "week" | "w" => duration.as_weeks(),
            "days" | "day" | "d" => duration.as_days(),
            "hours" | "hour" | "h" => duration.as_hours(),
            "minutes" | "minute" | "min" | "m" => duration.as_minutes(),
            "seconds" | "second" | "sec" | "s" => duration.as_secs(),
            "milliseconds" | "millisecond" | "ms" => duration.as_millis(),
            _ => return Value::Error(FolioError::domain_error(
                format!("unknown unit '{}'; use years, months, weeks, days, hours, minutes, seconds, or milliseconds", unit)
            )),
        };

        Value::Number(Number::from_i64(value))
    }
}

// ============================================================================
// Comparison Functions
// ============================================================================

pub struct IsBeforeFn;
pub struct IsAfterFn;
pub struct IsSameDayFn;

static ISBEFORE_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "dt1", typ: "DateTime", description: "First datetime", optional: false, default: None },
    ArgMeta { name: "dt2", typ: "DateTime", description: "Second datetime", optional: false, default: None },
];
static ISBEFORE_EXAMPLES: [&str; 1] = ["isBefore(date(2025, 1, 1), now())"];
static ISBEFORE_RELATED: [&str; 2] = ["isAfter", "isSameDay"];

static ISAFTER_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "dt1", typ: "DateTime", description: "First datetime", optional: false, default: None },
    ArgMeta { name: "dt2", typ: "DateTime", description: "Second datetime", optional: false, default: None },
];
static ISAFTER_EXAMPLES: [&str; 1] = ["isAfter(now(), date(2025, 1, 1))"];
static ISAFTER_RELATED: [&str; 2] = ["isBefore", "isSameDay"];

static ISSAMEDAY_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "dt1", typ: "DateTime", description: "First datetime", optional: false, default: None },
    ArgMeta { name: "dt2", typ: "DateTime", description: "Second datetime", optional: false, default: None },
];
static ISSAMEDAY_EXAMPLES: [&str; 1] = ["isSameDay(now(), now())"];
static ISSAMEDAY_RELATED: [&str; 2] = ["isBefore", "isAfter"];

impl FunctionPlugin for IsBeforeFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "isBefore",
            description: "Check if dt1 is before dt2",
            usage: "isBefore(dt1, dt2)",
            args: &ISBEFORE_ARGS,
            returns: "Bool",
            examples: &ISBEFORE_EXAMPLES,
            category: "datetime",
            source: None,
            related: &ISBEFORE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("isBefore", 2, args.len()));
        }

        let dt1 = match &args[0] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("isBefore", "dt1", "DateTime", other.type_name())),
        };

        let dt2 = match &args[1] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("isBefore", "dt2", "DateTime", other.type_name())),
        };

        Value::Bool(dt1.is_before(dt2))
    }
}

impl FunctionPlugin for IsAfterFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "isAfter",
            description: "Check if dt1 is after dt2",
            usage: "isAfter(dt1, dt2)",
            args: &ISAFTER_ARGS,
            returns: "Bool",
            examples: &ISAFTER_EXAMPLES,
            category: "datetime",
            source: None,
            related: &ISAFTER_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("isAfter", 2, args.len()));
        }

        let dt1 = match &args[0] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("isAfter", "dt1", "DateTime", other.type_name())),
        };

        let dt2 = match &args[1] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("isAfter", "dt2", "DateTime", other.type_name())),
        };

        Value::Bool(dt1.is_after(dt2))
    }
}

impl FunctionPlugin for IsSameDayFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "isSameDay",
            description: "Check if two datetimes are on the same day",
            usage: "isSameDay(dt1, dt2)",
            args: &ISSAMEDAY_ARGS,
            returns: "Bool",
            examples: &ISSAMEDAY_EXAMPLES,
            category: "datetime",
            source: None,
            related: &ISSAMEDAY_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("isSameDay", 2, args.len()));
        }

        let dt1 = match &args[0] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("isSameDay", "dt1", "DateTime", other.type_name())),
        };

        let dt2 = match &args[1] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("isSameDay", "dt2", "DateTime", other.type_name())),
        };

        Value::Bool(dt1.is_same_day(dt2))
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

pub struct StartOfDayFn;
pub struct EndOfDayFn;
pub struct StartOfMonthFn;
pub struct StartOfYearFn;

static STARTOFDAY_ARGS: [ArgMeta; 1] = [
    ArgMeta { name: "dt", typ: "DateTime", description: "DateTime", optional: false, default: None },
];
static STARTOFDAY_EXAMPLES: [&str; 1] = ["startOfDay(now())"];
static STARTOFDAY_RELATED: [&str; 3] = ["endOfDay", "startOfMonth", "startOfYear"];

static ENDOFDAY_ARGS: [ArgMeta; 1] = [
    ArgMeta { name: "dt", typ: "DateTime", description: "DateTime", optional: false, default: None },
];
static ENDOFDAY_EXAMPLES: [&str; 1] = ["endOfDay(now())"];
static ENDOFDAY_RELATED: [&str; 2] = ["startOfDay", "startOfMonth"];

static STARTOFMONTH_ARGS: [ArgMeta; 1] = [
    ArgMeta { name: "dt", typ: "DateTime", description: "DateTime", optional: false, default: None },
];
static STARTOFMONTH_EXAMPLES: [&str; 1] = ["startOfMonth(now())"];
static STARTOFMONTH_RELATED: [&str; 2] = ["startOfDay", "startOfYear"];

static STARTOFYEAR_ARGS: [ArgMeta; 1] = [
    ArgMeta { name: "dt", typ: "DateTime", description: "DateTime", optional: false, default: None },
];
static STARTOFYEAR_EXAMPLES: [&str; 1] = ["startOfYear(now())"];
static STARTOFYEAR_RELATED: [&str; 2] = ["startOfDay", "startOfMonth"];

macro_rules! utility_fn {
    ($struct:ident, $name:literal, $desc:literal, $method:ident, $args:ident, $examples:ident, $related:ident) => {
        impl FunctionPlugin for $struct {
            fn meta(&self) -> FunctionMeta {
                FunctionMeta {
                    name: $name,
                    description: $desc,
                    usage: concat!($name, "(dt)"),
                    args: &$args,
                    returns: "DateTime",
                    examples: &$examples,
                    category: "datetime",
                    source: None,
                    related: &$related,
                }
            }

            fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
                if args.len() != 1 {
                    return Value::Error(FolioError::arg_count($name, 1, args.len()));
                }
                match &args[0] {
                    Value::DateTime(dt) => Value::DateTime(dt.$method()),
                    Value::Error(e) => Value::Error(e.clone()),
                    other => Value::Error(FolioError::arg_type($name, "dt", "DateTime", other.type_name())),
                }
            }
        }
    };
}

utility_fn!(StartOfDayFn, "startOfDay", "Get start of day (00:00:00)", start_of_day, STARTOFDAY_ARGS, STARTOFDAY_EXAMPLES, STARTOFDAY_RELATED);
utility_fn!(EndOfDayFn, "endOfDay", "Get end of day (23:59:59.999...)", end_of_day, ENDOFDAY_ARGS, ENDOFDAY_EXAMPLES, ENDOFDAY_RELATED);
utility_fn!(StartOfMonthFn, "startOfMonth", "Get first day of month", start_of_month, STARTOFMONTH_ARGS, STARTOFMONTH_EXAMPLES, STARTOFMONTH_RELATED);
utility_fn!(StartOfYearFn, "startOfYear", "Get first day of year", start_of_year, STARTOFYEAR_ARGS, STARTOFYEAR_EXAMPLES, STARTOFYEAR_RELATED);

// ============================================================================
// DateTime Shortcut Functions
// ============================================================================

// End of period shortcuts
pub struct EodFn;
pub struct EowFn;
pub struct EomFn;
pub struct EoqFn;
pub struct EoyFn;

// Start of period shortcuts
pub struct SodFn;
pub struct SowFn;
pub struct SomFn;
pub struct SoqFn;
pub struct SoyFn;

// Navigation shortcuts
pub struct TomorrowFn;
pub struct NextWeekFn;
pub struct NextMonthFn;
pub struct NextMonthWdFn;

// Workday functions
pub struct IsWorkdayFn;
pub struct NextWorkdayFn;
pub struct PrevWorkdayFn;
pub struct AddWorkdaysFn;

// Optional datetime arg
static OPT_DT_ARGS: [ArgMeta; 1] = [
    ArgMeta { name: "dt", typ: "DateTime", description: "Reference datetime (default: now)", optional: true, default: Some("now()") },
];

// Helper to get optional datetime or now()
fn get_dt_or_now(args: &[Value], func: &str) -> Result<FolioDateTime, Value> {
    if args.is_empty() {
        Ok(FolioDateTime::now())
    } else {
        match &args[0] {
            Value::DateTime(dt) => Ok(dt.clone()),
            Value::Error(e) => Err(Value::Error(e.clone())),
            other => Err(Value::Error(FolioError::arg_type(func, "dt", "DateTime", other.type_name()))),
        }
    }
}

// ========== End of Period ==========

impl FunctionPlugin for EodFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "eod",
            description: "End of day (23:59:59.999...)",
            usage: "eod() or eod(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["eod()", "eod(date(2025, 6, 15))"],
            category: "datetime",
            source: None,
            related: &["sod", "eow", "eom"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("eod", 1, args.len()));
        }
        match get_dt_or_now(args, "eod") {
            Ok(dt) => Value::DateTime(dt.end_of_day()),
            Err(e) => e,
        }
    }
}

impl FunctionPlugin for EowFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "eow",
            description: "End of week (Sunday 23:59:59.999...)",
            usage: "eow() or eow(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["eow()", "eow(date(2025, 6, 15))"],
            category: "datetime",
            source: None,
            related: &["sow", "eod", "eom"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("eow", 1, args.len()));
        }
        match get_dt_or_now(args, "eow") {
            Ok(dt) => Value::DateTime(dt.end_of_week(1)), // ISO week (Mon-Sun)
            Err(e) => e,
        }
    }
}

impl FunctionPlugin for EomFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "eom",
            description: "End of month (last day 23:59:59.999...)",
            usage: "eom() or eom(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["eom()", "eom(date(2025, 2, 15))"],
            category: "datetime",
            source: None,
            related: &["som", "eow", "eoq"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("eom", 1, args.len()));
        }
        match get_dt_or_now(args, "eom") {
            Ok(dt) => Value::DateTime(dt.end_of_month()),
            Err(e) => e,
        }
    }
}

impl FunctionPlugin for EoqFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "eoq",
            description: "End of quarter (last day 23:59:59.999...)",
            usage: "eoq() or eoq(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["eoq()", "eoq(date(2025, 7, 15))"],
            category: "datetime",
            source: None,
            related: &["soq", "eom", "eoy"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("eoq", 1, args.len()));
        }
        match get_dt_or_now(args, "eoq") {
            Ok(dt) => Value::DateTime(dt.end_of_quarter()),
            Err(e) => e,
        }
    }
}

impl FunctionPlugin for EoyFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "eoy",
            description: "End of year (Dec 31 23:59:59.999...)",
            usage: "eoy() or eoy(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["eoy()", "eoy(date(2025, 6, 15))"],
            category: "datetime",
            source: None,
            related: &["soy", "eoq", "eom"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("eoy", 1, args.len()));
        }
        match get_dt_or_now(args, "eoy") {
            Ok(dt) => Value::DateTime(dt.end_of_year()),
            Err(e) => e,
        }
    }
}

// ========== Start of Period ==========

impl FunctionPlugin for SodFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "sod",
            description: "Start of day (00:00:00)",
            usage: "sod() or sod(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["sod()", "sod(now())"],
            category: "datetime",
            source: None,
            related: &["eod", "sow", "som"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("sod", 1, args.len()));
        }
        match get_dt_or_now(args, "sod") {
            Ok(dt) => Value::DateTime(dt.start_of_day()),
            Err(e) => e,
        }
    }
}

impl FunctionPlugin for SowFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "sow",
            description: "Start of week (Monday 00:00:00)",
            usage: "sow() or sow(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["sow()", "sow(now())"],
            category: "datetime",
            source: None,
            related: &["eow", "sod", "som"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("sow", 1, args.len()));
        }
        match get_dt_or_now(args, "sow") {
            Ok(dt) => Value::DateTime(dt.start_of_week(1)), // ISO week (Monday start)
            Err(e) => e,
        }
    }
}

impl FunctionPlugin for SomFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "som",
            description: "Start of month (1st at 00:00:00)",
            usage: "som() or som(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["som()", "som(now())"],
            category: "datetime",
            source: None,
            related: &["eom", "sow", "soq"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("som", 1, args.len()));
        }
        match get_dt_or_now(args, "som") {
            Ok(dt) => Value::DateTime(dt.start_of_month()),
            Err(e) => e,
        }
    }
}

impl FunctionPlugin for SoqFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "soq",
            description: "Start of quarter (1st at 00:00:00)",
            usage: "soq() or soq(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["soq()", "soq(date(2025, 7, 15))"],
            category: "datetime",
            source: None,
            related: &["eoq", "som", "soy"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("soq", 1, args.len()));
        }
        match get_dt_or_now(args, "soq") {
            Ok(dt) => Value::DateTime(dt.start_of_quarter()),
            Err(e) => e,
        }
    }
}

impl FunctionPlugin for SoyFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "soy",
            description: "Start of year (Jan 1 00:00:00)",
            usage: "soy() or soy(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["soy()", "soy(now())"],
            category: "datetime",
            source: None,
            related: &["eoy", "soq", "som"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("soy", 1, args.len()));
        }
        match get_dt_or_now(args, "soy") {
            Ok(dt) => Value::DateTime(dt.start_of_year()),
            Err(e) => e,
        }
    }
}

// ========== Navigation ==========

impl FunctionPlugin for TomorrowFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "tomorrow",
            description: "Tomorrow (same time, +1 day)",
            usage: "tomorrow() or tomorrow(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["tomorrow()", "tomorrow(date(2025, 6, 15))"],
            category: "datetime",
            source: None,
            related: &["addDays", "nextWorkday"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("tomorrow", 1, args.len()));
        }
        match get_dt_or_now(args, "tomorrow") {
            Ok(dt) => Value::DateTime(dt.tomorrow()),
            Err(e) => e,
        }
    }
}

impl FunctionPlugin for NextWeekFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "nextWeek",
            description: "Monday of next week (00:00:00)",
            usage: "nextWeek() or nextWeek(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["nextWeek()", "nextWeek(date(2025, 6, 15))"],
            category: "datetime",
            source: None,
            related: &["sow", "eow", "nextMonth"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("nextWeek", 1, args.len()));
        }
        match get_dt_or_now(args, "nextWeek") {
            Ok(dt) => Value::DateTime(dt.next_week(1)), // ISO week (Monday start)
            Err(e) => e,
        }
    }
}

impl FunctionPlugin for NextMonthFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "nextMonth",
            description: "First day of next month (00:00:00)",
            usage: "nextMonth() or nextMonth(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["nextMonth()", "nextMonth(date(2025, 6, 15))"],
            category: "datetime",
            source: None,
            related: &["som", "eom", "nextMonthWd"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("nextMonth", 1, args.len()));
        }
        match get_dt_or_now(args, "nextMonth") {
            Ok(dt) => Value::DateTime(dt.next_month_first()),
            Err(e) => e,
        }
    }
}

impl FunctionPlugin for NextMonthWdFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "nextMonthWd",
            description: "First workday of next month (skips weekends)",
            usage: "nextMonthWd() or nextMonthWd(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["nextMonthWd()", "nextMonthWd(date(2025, 6, 15))"],
            category: "datetime",
            source: None,
            related: &["nextMonth", "nextWorkday", "isWorkday"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("nextMonthWd", 1, args.len()));
        }
        match get_dt_or_now(args, "nextMonthWd") {
            Ok(dt) => Value::DateTime(dt.next_month_first_workday()),
            Err(e) => e,
        }
    }
}

// ========== Workday Functions ==========

impl FunctionPlugin for IsWorkdayFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "isWorkday",
            description: "Check if date is a workday (Mon-Fri)",
            usage: "isWorkday() or isWorkday(dt)",
            args: &OPT_DT_ARGS,
            returns: "Bool",
            examples: &["isWorkday()", "isWorkday(date(2025, 6, 14))"],
            category: "datetime",
            source: None,
            related: &["nextWorkday", "prevWorkday", "addWorkdays"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("isWorkday", 1, args.len()));
        }
        match get_dt_or_now(args, "isWorkday") {
            Ok(dt) => Value::Bool(dt.is_workday()),
            Err(e) => e,
        }
    }
}

impl FunctionPlugin for NextWorkdayFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "nextWorkday",
            description: "Next workday (skips weekends)",
            usage: "nextWorkday() or nextWorkday(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["nextWorkday()", "nextWorkday(date(2025, 6, 14))"],
            category: "datetime",
            source: None,
            related: &["prevWorkday", "addWorkdays", "isWorkday"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("nextWorkday", 1, args.len()));
        }
        match get_dt_or_now(args, "nextWorkday") {
            Ok(dt) => Value::DateTime(dt.next_workday()),
            Err(e) => e,
        }
    }
}

impl FunctionPlugin for PrevWorkdayFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "prevWorkday",
            description: "Previous workday (skips weekends)",
            usage: "prevWorkday() or prevWorkday(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["prevWorkday()", "prevWorkday(date(2025, 6, 16))"],
            category: "datetime",
            source: None,
            related: &["nextWorkday", "addWorkdays", "isWorkday"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("prevWorkday", 1, args.len()));
        }
        match get_dt_or_now(args, "prevWorkday") {
            Ok(dt) => Value::DateTime(dt.prev_workday()),
            Err(e) => e,
        }
    }
}

static ADDWORKDAYS_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "dt", typ: "DateTime", description: "Base datetime", optional: false, default: None },
    ArgMeta { name: "n", typ: "Number", description: "Number of workdays to add", optional: false, default: None },
];

impl FunctionPlugin for AddWorkdaysFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "addWorkdays",
            description: "Add N workdays (skips weekends)",
            usage: "addWorkdays(dt, n)",
            args: &ADDWORKDAYS_ARGS,
            returns: "DateTime",
            examples: &["addWorkdays(now(), 5)", "addWorkdays(date(2025, 6, 1), 30)"],
            category: "datetime",
            source: None,
            related: &["addDays", "nextWorkday", "isWorkday"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("addWorkdays", 2, args.len()));
        }

        let dt = match &args[0] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("addWorkdays", "dt", "DateTime", other.type_name())),
        };

        let n = match get_i64(&args[1], "addWorkdays", "n") {
            Ok(v) => v,
            Err(e) => return e,
        };

        Value::DateTime(dt.add_workdays(n))
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn get_i32(value: &Value, func: &str, arg: &str) -> Result<i32, Value> {
    match value {
        Value::Number(n) => {
            if let Some(v) = n.to_i64() {
                if v >= i32::MIN as i64 && v <= i32::MAX as i64 {
                    Ok(v as i32)
                } else {
                    Err(Value::Error(FolioError::domain_error(format!("{} out of range for {}", arg, func))))
                }
            } else {
                Err(Value::Error(FolioError::arg_type(func, arg, "integer", "non-integer")))
            }
        }
        Value::Error(e) => Err(Value::Error(e.clone())),
        other => Err(Value::Error(FolioError::arg_type(func, arg, "Number", other.type_name()))),
    }
}

fn get_u32(value: &Value, func: &str, arg: &str) -> Result<u32, Value> {
    match value {
        Value::Number(n) => {
            if let Some(v) = n.to_i64() {
                if v >= 0 && v <= u32::MAX as i64 {
                    Ok(v as u32)
                } else {
                    Err(Value::Error(FolioError::domain_error(format!("{} must be non-negative for {}", arg, func))))
                }
            } else {
                Err(Value::Error(FolioError::arg_type(func, arg, "integer", "non-integer")))
            }
        }
        Value::Error(e) => Err(Value::Error(e.clone())),
        other => Err(Value::Error(FolioError::arg_type(func, arg, "Number", other.type_name()))),
    }
}

fn get_i64(value: &Value, func: &str, arg: &str) -> Result<i64, Value> {
    match value {
        Value::Number(n) => {
            if let Some(v) = n.to_i64() {
                Ok(v)
            } else {
                Err(Value::Error(FolioError::arg_type(func, arg, "integer", "non-integer or out of range")))
            }
        }
        Value::Error(e) => Err(Value::Error(e.clone())),
        other => Err(Value::Error(FolioError::arg_type(func, arg, "Number", other.type_name()))),
    }
}
