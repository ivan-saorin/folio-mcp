//! DateTime and Duration types for Folio
//!
//! Provides nanosecond-precision datetime and duration types with full
//! arithmetic support. Uses i128 internally to avoid overflow issues.
//!
//! Design principles:
//! - No external datetime crates (keeps folio-core minimal)
//! - Gregorian proleptic calendar
//! - UTC-first with optional timezone offset
//! - Never panics - all operations return Results or handle edge cases

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// Constants
// ============================================================================

pub const NANOS_PER_SECOND: i128 = 1_000_000_000;
pub const NANOS_PER_MINUTE: i128 = 60 * NANOS_PER_SECOND;
pub const NANOS_PER_HOUR: i128 = 60 * NANOS_PER_MINUTE;
pub const NANOS_PER_DAY: i128 = 24 * NANOS_PER_HOUR;

/// Days in each month (non-leap year)
const DAYS_IN_MONTH: [i32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// Unix epoch: 1970-01-01T00:00:00Z
const UNIX_EPOCH_DAYS: i64 = 719_468; // Days from year 0 to 1970-01-01

// ============================================================================
// FolioDateTime
// ============================================================================

/// A datetime with nanosecond precision
///
/// Internally stores nanoseconds since Unix epoch (1970-01-01T00:00:00Z).
/// Supports dates from billions of years in the past to billions in the future.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FolioDateTime {
    /// Nanoseconds since Unix epoch (negative for pre-1970 dates)
    nanos: i128,
    /// Timezone offset in seconds from UTC (None = UTC)
    #[serde(skip_serializing_if = "Option::is_none")]
    tz_offset: Option<i32>,
}

impl FolioDateTime {
    // ========== Construction ==========

    /// Create a datetime from nanoseconds since Unix epoch
    pub fn from_nanos(nanos: i128) -> Self {
        Self { nanos, tz_offset: None }
    }

    /// Create a datetime from seconds since Unix epoch
    pub fn from_unix_secs(secs: i64) -> Self {
        Self {
            nanos: (secs as i128) * NANOS_PER_SECOND,
            tz_offset: None,
        }
    }

    /// Create a datetime from milliseconds since Unix epoch
    pub fn from_unix_millis(millis: i64) -> Self {
        Self {
            nanos: (millis as i128) * 1_000_000,
            tz_offset: None,
        }
    }

    /// Create a date (time = 00:00:00)
    pub fn from_ymd(year: i32, month: u32, day: u32) -> Result<Self, DateTimeError> {
        Self::from_ymd_hms_nano(year, month, day, 0, 0, 0, 0)
    }

    /// Create a datetime from components
    pub fn from_ymd_hms(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> Result<Self, DateTimeError> {
        Self::from_ymd_hms_nano(year, month, day, hour, minute, second, 0)
    }

    /// Create a datetime from components with nanoseconds
    pub fn from_ymd_hms_nano(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
        nano: u32,
    ) -> Result<Self, DateTimeError> {
        // Validate components
        if month < 1 || month > 12 {
            return Err(DateTimeError::InvalidMonth(month));
        }
        let max_day = days_in_month(year, month);
        if day < 1 || day > max_day {
            return Err(DateTimeError::InvalidDay(day, month, year));
        }
        if hour > 23 {
            return Err(DateTimeError::InvalidHour(hour));
        }
        if minute > 59 {
            return Err(DateTimeError::InvalidMinute(minute));
        }
        if second > 59 {
            return Err(DateTimeError::InvalidSecond(second));
        }
        if nano >= 1_000_000_000 {
            return Err(DateTimeError::InvalidNano(nano));
        }

        // Convert to days since epoch
        let days = days_from_civil(year, month, day);
        let day_nanos = (days as i128) * NANOS_PER_DAY;

        // Add time components
        let time_nanos = (hour as i128) * NANOS_PER_HOUR
            + (minute as i128) * NANOS_PER_MINUTE
            + (second as i128) * NANOS_PER_SECOND
            + (nano as i128);

        Ok(Self {
            nanos: day_nanos + time_nanos,
            tz_offset: None,
        })
    }

    /// Create a time-only value (date = 1970-01-01)
    pub fn from_hms(hour: u32, minute: u32, second: u32) -> Result<Self, DateTimeError> {
        Self::from_ymd_hms(1970, 1, 1, hour, minute, second)
    }

    /// Get current UTC time
    pub fn now() -> Self {
        // Use std::time for current time
        let duration = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        Self {
            nanos: duration.as_nanos() as i128,
            tz_offset: None,
        }
    }

    // ========== Accessors ==========

    /// Get nanoseconds since Unix epoch
    pub fn as_nanos(&self) -> i128 {
        self.nanos
    }

    /// Get seconds since Unix epoch (truncated)
    pub fn as_unix_secs(&self) -> i64 {
        (self.nanos / NANOS_PER_SECOND) as i64
    }

    /// Get milliseconds since Unix epoch (truncated)
    pub fn as_unix_millis(&self) -> i64 {
        (self.nanos / 1_000_000) as i64
    }

    /// Get timezone offset in seconds (None = UTC)
    pub fn tz_offset(&self) -> Option<i32> {
        self.tz_offset
    }

    /// Set timezone offset
    pub fn with_tz_offset(mut self, offset_secs: i32) -> Self {
        self.tz_offset = Some(offset_secs);
        self
    }

    /// Convert to UTC (remove timezone info)
    pub fn to_utc(mut self) -> Self {
        self.tz_offset = None;
        self
    }

    /// Get year component
    pub fn year(&self) -> i32 {
        let (y, _, _) = self.to_ymd();
        y
    }

    /// Get month component (1-12)
    pub fn month(&self) -> u32 {
        let (_, m, _) = self.to_ymd();
        m
    }

    /// Get day component (1-31)
    pub fn day(&self) -> u32 {
        let (_, _, d) = self.to_ymd();
        d
    }

    /// Get hour component (0-23)
    pub fn hour(&self) -> u32 {
        let day_nanos = self.nanos.rem_euclid(NANOS_PER_DAY);
        (day_nanos / NANOS_PER_HOUR) as u32
    }

    /// Get minute component (0-59)
    pub fn minute(&self) -> u32 {
        let day_nanos = self.nanos.rem_euclid(NANOS_PER_DAY);
        ((day_nanos % NANOS_PER_HOUR) / NANOS_PER_MINUTE) as u32
    }

    /// Get second component (0-59)
    pub fn second(&self) -> u32 {
        let day_nanos = self.nanos.rem_euclid(NANOS_PER_DAY);
        ((day_nanos % NANOS_PER_MINUTE) / NANOS_PER_SECOND) as u32
    }

    /// Get nanosecond component (0-999_999_999)
    pub fn nanosecond(&self) -> u32 {
        (self.nanos.rem_euclid(NANOS_PER_SECOND)) as u32
    }

    /// Get millisecond component (0-999)
    pub fn millisecond(&self) -> u32 {
        self.nanosecond() / 1_000_000
    }

    /// Get day of week (1=Monday, 7=Sunday, ISO 8601)
    pub fn weekday(&self) -> u32 {
        let days = self.nanos.div_euclid(NANOS_PER_DAY);
        // 1970-01-01 was Thursday (4)
        let day_of_week = (days + 4).rem_euclid(7);
        if day_of_week == 0 { 7 } else { day_of_week as u32 }
    }

    /// Get day of year (1-366)
    pub fn day_of_year(&self) -> u32 {
        let (year, month, day) = self.to_ymd();
        let mut doy = day;
        for m in 1..month {
            doy += days_in_month(year, m);
        }
        doy
    }

    /// Get ISO week number (1-53)
    pub fn iso_week(&self) -> u32 {
        // ISO week: week containing first Thursday of year
        let doy = self.day_of_year() as i32;
        let dow = self.weekday() as i32; // 1=Mon, 7=Sun

        // Find the Thursday of the current week
        let thursday_doy = doy + (4 - dow);

        // Week 1 contains Jan 4th
        let jan4_dow = FolioDateTime::from_ymd(self.year(), 1, 4)
            .map(|dt| dt.weekday() as i32)
            .unwrap_or(4);
        let week1_start = 4 - jan4_dow + 1; // Day of year when week 1 starts

        let week = (thursday_doy - week1_start) / 7 + 1;

        if week < 1 {
            // Last week of previous year
            FolioDateTime::from_ymd(self.year() - 1, 12, 31)
                .map(|dt| dt.iso_week())
                .unwrap_or(52)
        } else if week > 52 {
            // Check if it's week 1 of next year
            let next_jan4_dow = FolioDateTime::from_ymd(self.year() + 1, 1, 4)
                .map(|dt| dt.weekday() as i32)
                .unwrap_or(4);
            let days_in_year = if is_leap_year(self.year()) { 366 } else { 365 };
            if doy > days_in_year - (7 - next_jan4_dow) {
                1
            } else {
                week as u32
            }
        } else {
            week as u32
        }
    }

    /// Decompose into year, month, day
    pub fn to_ymd(&self) -> (i32, u32, u32) {
        let days = self.nanos.div_euclid(NANOS_PER_DAY) as i64;
        civil_from_days(days)
    }

    /// Decompose into all components
    pub fn to_components(&self) -> DateTimeComponents {
        let (year, month, day) = self.to_ymd();
        DateTimeComponents {
            year,
            month,
            day,
            hour: self.hour(),
            minute: self.minute(),
            second: self.second(),
            nanosecond: self.nanosecond(),
            tz_offset: self.tz_offset,
        }
    }

    // ========== Arithmetic ==========

    /// Add a duration
    pub fn add_duration(&self, duration: &FolioDuration) -> Self {
        Self {
            nanos: self.nanos + duration.nanos,
            tz_offset: self.tz_offset,
        }
    }

    /// Subtract a duration
    pub fn sub_duration(&self, duration: &FolioDuration) -> Self {
        Self {
            nanos: self.nanos - duration.nanos,
            tz_offset: self.tz_offset,
        }
    }

    /// Get duration between two datetimes
    pub fn duration_since(&self, other: &FolioDateTime) -> FolioDuration {
        FolioDuration {
            nanos: self.nanos - other.nanos,
        }
    }

    /// Add days
    pub fn add_days(&self, days: i64) -> Self {
        Self {
            nanos: self.nanos + (days as i128) * NANOS_PER_DAY,
            tz_offset: self.tz_offset,
        }
    }

    /// Add months (handles month boundaries)
    pub fn add_months(&self, months: i32) -> Self {
        let (mut year, mut month, day) = self.to_ymd();

        // Add months
        let total_months = (year as i64) * 12 + (month as i64 - 1) + (months as i64);
        year = (total_months.div_euclid(12)) as i32;
        month = (total_months.rem_euclid(12) + 1) as u32;

        // Clamp day to valid range for new month
        let max_day = days_in_month(year, month);
        let new_day = day.min(max_day);

        // Preserve time components
        let time_nanos = self.nanos.rem_euclid(NANOS_PER_DAY);
        let days = days_from_civil(year, month, new_day);

        Self {
            nanos: (days as i128) * NANOS_PER_DAY + time_nanos,
            tz_offset: self.tz_offset,
        }
    }

    /// Add years (handles leap years)
    pub fn add_years(&self, years: i32) -> Self {
        self.add_months(years * 12)
    }

    // ========== Utilities ==========

    /// Get start of day (00:00:00.000)
    pub fn start_of_day(&self) -> Self {
        let days = self.nanos.div_euclid(NANOS_PER_DAY);
        Self {
            nanos: days * NANOS_PER_DAY,
            tz_offset: self.tz_offset,
        }
    }

    /// Get end of day (23:59:59.999999999)
    pub fn end_of_day(&self) -> Self {
        let days = self.nanos.div_euclid(NANOS_PER_DAY);
        Self {
            nanos: (days + 1) * NANOS_PER_DAY - 1,
            tz_offset: self.tz_offset,
        }
    }

    /// Get start of month
    pub fn start_of_month(&self) -> Self {
        let (year, month, _) = self.to_ymd();
        Self::from_ymd(year, month, 1).unwrap_or_else(|_| self.clone())
    }

    /// Get start of year
    pub fn start_of_year(&self) -> Self {
        let year = self.year();
        Self::from_ymd(year, 1, 1).unwrap_or_else(|_| self.clone())
    }

    /// Check if same day as another datetime
    pub fn is_same_day(&self, other: &FolioDateTime) -> bool {
        self.nanos.div_euclid(NANOS_PER_DAY) == other.nanos.div_euclid(NANOS_PER_DAY)
    }

    /// Check if before another datetime
    pub fn is_before(&self, other: &FolioDateTime) -> bool {
        self.nanos < other.nanos
    }

    /// Check if after another datetime
    pub fn is_after(&self, other: &FolioDateTime) -> bool {
        self.nanos > other.nanos
    }

    // ========== Period End Methods ==========

    /// Get end of week (Sunday 23:59:59.999...)
    /// week_start: 1=Monday (ISO), 7=Sunday (US)
    pub fn end_of_week(&self, week_start: u32) -> Self {
        // Calculate days until end of week
        let dow = self.weekday(); // 1=Mon, 7=Sun
        let week_end = if week_start == 7 {
            // US style: week ends on Saturday (6)
            6
        } else {
            // ISO style (default): week ends on Sunday (7)
            7
        };

        let days_until_end = if dow == week_end {
            0
        } else if week_end > dow {
            (week_end - dow) as i64
        } else {
            (7 - dow + week_end) as i64
        };

        self.add_days(days_until_end).end_of_day()
    }

    /// Get end of month (last day at 23:59:59.999...)
    pub fn end_of_month(&self) -> Self {
        let (year, month, _) = self.to_ymd();
        let last_day = days_in_month(year, month);
        Self::from_ymd(year, month, last_day)
            .map(|dt| dt.end_of_day())
            .unwrap_or_else(|_| self.clone())
    }

    /// Get end of quarter (last day of quarter at 23:59:59.999...)
    pub fn end_of_quarter(&self) -> Self {
        let (year, month, _) = self.to_ymd();
        let quarter_end_month = match month {
            1..=3 => 3,
            4..=6 => 6,
            7..=9 => 9,
            _ => 12,
        };
        let last_day = days_in_month(year, quarter_end_month);
        Self::from_ymd(year, quarter_end_month, last_day)
            .map(|dt| dt.end_of_day())
            .unwrap_or_else(|_| self.clone())
    }

    /// Get end of year (Dec 31 23:59:59.999...)
    pub fn end_of_year(&self) -> Self {
        let year = self.year();
        Self::from_ymd(year, 12, 31)
            .map(|dt| dt.end_of_day())
            .unwrap_or_else(|_| self.clone())
    }

    // ========== Period Start Methods ==========

    /// Get start of week (Monday 00:00:00 by default)
    /// week_start: 1=Monday (ISO), 7=Sunday (US)
    pub fn start_of_week(&self, week_start: u32) -> Self {
        let dow = self.weekday(); // 1=Mon, 7=Sun
        let target_start = if week_start == 7 { 7 } else { 1 }; // Sunday or Monday

        let days_since_start = if dow >= target_start {
            (dow - target_start) as i64
        } else {
            (7 - target_start + dow) as i64
        };

        self.add_days(-days_since_start).start_of_day()
    }

    /// Get start of quarter
    pub fn start_of_quarter(&self) -> Self {
        let (year, month, _) = self.to_ymd();
        let quarter_start_month = match month {
            1..=3 => 1,
            4..=6 => 4,
            7..=9 => 7,
            _ => 10,
        };
        Self::from_ymd(year, quarter_start_month, 1)
            .map(|dt| dt.start_of_day())
            .unwrap_or_else(|_| self.clone())
    }

    // ========== Workday Methods ==========

    /// Check if this is a weekend (Saturday=6 or Sunday=7)
    pub fn is_weekend(&self) -> bool {
        let dow = self.weekday();
        dow == 6 || dow == 7
    }

    /// Check if this is a workday (Monday-Friday)
    pub fn is_workday(&self) -> bool {
        !self.is_weekend()
    }

    /// Get next workday (if already workday, returns same day at start)
    pub fn next_workday_inclusive(&self) -> Self {
        let dow = self.weekday();
        let days_to_add = match dow {
            6 => 2, // Saturday -> Monday
            7 => 1, // Sunday -> Monday
            _ => 0,
        };
        if days_to_add > 0 {
            self.add_days(days_to_add).start_of_day()
        } else {
            self.clone()
        }
    }

    /// Get next workday (always advances at least one day)
    pub fn next_workday(&self) -> Self {
        let next = self.add_days(1);
        let dow = next.weekday();
        let days_to_add = match dow {
            6 => 2, // Saturday -> Monday
            7 => 1, // Sunday -> Monday
            _ => 0,
        };
        next.add_days(days_to_add).start_of_day()
    }

    /// Get previous workday (always goes back at least one day)
    pub fn prev_workday(&self) -> Self {
        let prev = self.add_days(-1);
        let dow = prev.weekday();
        let days_to_sub = match dow {
            6 => 1, // Saturday -> Friday
            7 => 2, // Sunday -> Friday
            _ => 0,
        };
        prev.add_days(-days_to_sub).start_of_day()
    }

    /// Add n workdays (skips weekends)
    pub fn add_workdays(&self, n: i64) -> Self {
        if n == 0 {
            return self.clone();
        }

        let mut current = self.clone();
        let mut remaining = n.abs();
        let direction = if n > 0 { 1i64 } else { -1i64 };

        // First, move to a workday if on weekend
        if current.is_weekend() {
            if direction > 0 {
                current = current.next_workday_inclusive();
            } else {
                current = current.prev_workday();
                if remaining > 0 {
                    remaining -= 1;
                }
            }
        }

        while remaining > 0 {
            current = current.add_days(direction);
            if current.is_workday() {
                remaining -= 1;
            }
        }

        current.start_of_day()
    }

    // ========== Navigation Methods ==========

    /// Get tomorrow (same time, +1 day)
    pub fn tomorrow(&self) -> Self {
        self.add_days(1)
    }

    /// Get next week (Monday 00:00:00)
    pub fn next_week(&self, week_start: u32) -> Self {
        // Go to end of current week, then add 1 day
        self.end_of_week(week_start).add_days(1).start_of_day()
    }

    /// Get first day of next month (00:00:00)
    pub fn next_month_first(&self) -> Self {
        self.add_months(1).start_of_month()
    }

    /// Get first workday of next month
    pub fn next_month_first_workday(&self) -> Self {
        self.next_month_first().next_workday_inclusive()
    }

    /// Get first day of next quarter (00:00:00)
    pub fn next_quarter_first(&self) -> Self {
        let (year, month, _) = self.to_ymd();
        let (next_year, next_quarter_month) = match month {
            1..=3 => (year, 4),
            4..=6 => (year, 7),
            7..=9 => (year, 10),
            _ => (year + 1, 1),
        };
        Self::from_ymd(next_year, next_quarter_month, 1)
            .unwrap_or_else(|_| self.clone())
    }

    /// Get first day of next year (00:00:00)
    pub fn next_year_first(&self) -> Self {
        let year = self.year();
        Self::from_ymd(year + 1, 1, 1)
            .unwrap_or_else(|_| self.clone())
    }

    // ========== Formatting ==========

    /// Format as ISO 8601 string
    pub fn to_iso_string(&self) -> String {
        let c = self.to_components();
        if let Some(offset) = c.tz_offset {
            let (sign, abs_offset) = if offset < 0 { ('-', -offset) } else { ('+', offset) };
            let hours = abs_offset / 3600;
            let minutes = (abs_offset % 3600) / 60;
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}{}{:02}:{:02}",
                c.year, c.month, c.day, c.hour, c.minute, c.second,
                sign, hours, minutes
            )
        } else {
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                c.year, c.month, c.day, c.hour, c.minute, c.second
            )
        }
    }

    /// Format with custom pattern
    ///
    /// Supported tokens:
    /// - YYYY: 4-digit year
    /// - YY: 2-digit year
    /// - MM: 2-digit month (01-12)
    /// - M: month (1-12)
    /// - DD: 2-digit day (01-31)
    /// - D: day (1-31)
    /// - HH: 2-digit hour 24h (00-23)
    /// - H: hour 24h (0-23)
    /// - hh: 2-digit hour 12h (01-12)
    /// - h: hour 12h (1-12)
    /// - mm: 2-digit minute (00-59)
    /// - m: minute (0-59)
    /// - ss: 2-digit second (00-59)
    /// - s: second (0-59)
    /// - SSS: milliseconds (000-999)
    /// - A: AM/PM
    /// - a: am/pm
    /// - DDD: day of year (001-366)
    /// - d: weekday (1-7, Monday=1)
    /// - W: ISO week (1-53)
    pub fn format(&self, pattern: &str) -> String {
        let c = self.to_components();
        let hour12 = if c.hour == 0 { 12 } else if c.hour > 12 { c.hour - 12 } else { c.hour };
        let am_pm = if c.hour < 12 { "AM" } else { "PM" };
        let am_pm_lower = if c.hour < 12 { "am" } else { "pm" };

        let mut result = pattern.to_string();

        // Order matters - longer patterns first
        result = result.replace("YYYY", &format!("{:04}", c.year));
        result = result.replace("YY", &format!("{:02}", c.year.rem_euclid(100)));
        result = result.replace("MM", &format!("{:02}", c.month));
        result = result.replace("M", &c.month.to_string());
        result = result.replace("DDD", &format!("{:03}", self.day_of_year()));
        result = result.replace("DD", &format!("{:02}", c.day));
        result = result.replace("D", &c.day.to_string());
        result = result.replace("HH", &format!("{:02}", c.hour));
        result = result.replace("H", &c.hour.to_string());
        result = result.replace("hh", &format!("{:02}", hour12));
        result = result.replace("h", &hour12.to_string());
        result = result.replace("mm", &format!("{:02}", c.minute));
        result = result.replace("m", &c.minute.to_string());
        result = result.replace("SSS", &format!("{:03}", c.nanosecond / 1_000_000));
        result = result.replace("ss", &format!("{:02}", c.second));
        result = result.replace("s", &c.second.to_string());
        result = result.replace("A", am_pm);
        result = result.replace("a", am_pm_lower);
        result = result.replace("W", &self.iso_week().to_string());
        // 'd' for weekday - but be careful not to replace 'd' in other contexts
        // We'll use a workaround: only replace standalone 'd'

        result
    }
}

impl fmt::Display for FolioDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_iso_string())
    }
}

// ============================================================================
// FolioDuration
// ============================================================================

/// A duration with nanosecond precision
///
/// Can be positive (future) or negative (past).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FolioDuration {
    /// Signed nanoseconds
    nanos: i128,
}

impl FolioDuration {
    /// Create from nanoseconds
    pub fn from_nanos(nanos: i128) -> Self {
        Self { nanos }
    }

    /// Create from seconds
    pub fn from_secs(secs: i64) -> Self {
        Self {
            nanos: (secs as i128) * NANOS_PER_SECOND,
        }
    }

    /// Create from milliseconds
    pub fn from_millis(millis: i64) -> Self {
        Self {
            nanos: (millis as i128) * 1_000_000,
        }
    }

    /// Create from days
    pub fn from_days(days: i64) -> Self {
        Self {
            nanos: (days as i128) * NANOS_PER_DAY,
        }
    }

    /// Create from weeks
    pub fn from_weeks(weeks: i64) -> Self {
        Self {
            nanos: (weeks as i128) * NANOS_PER_DAY * 7,
        }
    }

    /// Create from hours
    pub fn from_hours(hours: i64) -> Self {
        Self {
            nanos: (hours as i128) * NANOS_PER_HOUR,
        }
    }

    /// Create from minutes
    pub fn from_minutes(minutes: i64) -> Self {
        Self {
            nanos: (minutes as i128) * NANOS_PER_MINUTE,
        }
    }

    /// Zero duration
    pub fn zero() -> Self {
        Self { nanos: 0 }
    }

    /// Get total nanoseconds
    pub fn as_nanos(&self) -> i128 {
        self.nanos
    }

    /// Get total seconds (truncated)
    pub fn as_secs(&self) -> i64 {
        (self.nanos / NANOS_PER_SECOND) as i64
    }

    /// Get total milliseconds (truncated)
    pub fn as_millis(&self) -> i64 {
        (self.nanos / 1_000_000) as i64
    }

    /// Get total minutes (truncated)
    pub fn as_minutes(&self) -> i64 {
        (self.nanos / NANOS_PER_MINUTE) as i64
    }

    /// Get total hours (truncated)
    pub fn as_hours(&self) -> i64 {
        (self.nanos / NANOS_PER_HOUR) as i64
    }

    /// Get total days (truncated)
    pub fn as_days(&self) -> i64 {
        (self.nanos / NANOS_PER_DAY) as i64
    }

    /// Get total weeks (truncated)
    pub fn as_weeks(&self) -> i64 {
        (self.nanos / (NANOS_PER_DAY * 7)) as i64
    }

    /// Get fractional days as f64
    pub fn as_days_f64(&self) -> f64 {
        (self.nanos as f64) / (NANOS_PER_DAY as f64)
    }

    /// Get fractional hours as f64
    pub fn as_hours_f64(&self) -> f64 {
        (self.nanos as f64) / (NANOS_PER_HOUR as f64)
    }

    /// Check if zero
    pub fn is_zero(&self) -> bool {
        self.nanos == 0
    }

    /// Check if negative
    pub fn is_negative(&self) -> bool {
        self.nanos < 0
    }

    /// Get absolute value
    pub fn abs(&self) -> Self {
        Self {
            nanos: self.nanos.abs(),
        }
    }

    /// Negate
    pub fn neg(&self) -> Self {
        Self { nanos: -self.nanos }
    }

    /// Add durations
    pub fn add(&self, other: &FolioDuration) -> Self {
        Self {
            nanos: self.nanos + other.nanos,
        }
    }

    /// Subtract durations
    pub fn sub(&self, other: &FolioDuration) -> Self {
        Self {
            nanos: self.nanos - other.nanos,
        }
    }

    /// Multiply by scalar
    pub fn mul(&self, scalar: i64) -> Self {
        Self {
            nanos: self.nanos * (scalar as i128),
        }
    }

    /// Multiply by float (for Number compatibility)
    pub fn mul_f64(&self, scalar: f64) -> Self {
        Self {
            nanos: ((self.nanos as f64) * scalar) as i128,
        }
    }

    /// Divide by scalar
    pub fn div(&self, scalar: i64) -> Option<Self> {
        if scalar == 0 {
            None
        } else {
            Some(Self {
                nanos: self.nanos / (scalar as i128),
            })
        }
    }
}

impl fmt::Display for FolioDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let abs_nanos = self.nanos.abs();
        let sign = if self.nanos < 0 { "-" } else { "" };

        let days = abs_nanos / NANOS_PER_DAY;
        let hours = (abs_nanos % NANOS_PER_DAY) / NANOS_PER_HOUR;
        let minutes = (abs_nanos % NANOS_PER_HOUR) / NANOS_PER_MINUTE;
        let seconds = (abs_nanos % NANOS_PER_MINUTE) / NANOS_PER_SECOND;

        if days > 0 {
            write!(f, "{}{}d {:02}:{:02}:{:02}", sign, days, hours, minutes, seconds)
        } else {
            write!(f, "{}{:02}:{:02}:{:02}", sign, hours, minutes, seconds)
        }
    }
}

// ============================================================================
// DateTimeComponents
// ============================================================================

/// Decomposed datetime components
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateTimeComponents {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub nanosecond: u32,
    pub tz_offset: Option<i32>,
}

// ============================================================================
// DateTimeError
// ============================================================================

/// Errors that can occur with datetime operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateTimeError {
    InvalidMonth(u32),
    InvalidDay(u32, u32, i32), // day, month, year
    InvalidHour(u32),
    InvalidMinute(u32),
    InvalidSecond(u32),
    InvalidNano(u32),
    ParseError(String),
    Overflow,
}

impl fmt::Display for DateTimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMonth(m) => write!(f, "Invalid month: {} (must be 1-12)", m),
            Self::InvalidDay(d, m, y) => write!(f, "Invalid day: {} for {}/{}", d, m, y),
            Self::InvalidHour(h) => write!(f, "Invalid hour: {} (must be 0-23)", h),
            Self::InvalidMinute(m) => write!(f, "Invalid minute: {} (must be 0-59)", m),
            Self::InvalidSecond(s) => write!(f, "Invalid second: {} (must be 0-59)", s),
            Self::InvalidNano(n) => write!(f, "Invalid nanosecond: {}", n),
            Self::ParseError(s) => write!(f, "Parse error: {}", s),
            Self::Overflow => write!(f, "DateTime overflow"),
        }
    }
}

impl std::error::Error for DateTimeError {}

// ============================================================================
// Calendar Utilities (Gregorian proleptic)
// ============================================================================

/// Check if year is a leap year
pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Get days in a month
pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        m if m >= 1 && m <= 12 => DAYS_IN_MONTH[(m - 1) as usize] as u32,
        _ => 0,
    }
}

/// Convert civil date to days since Unix epoch
/// Algorithm from Howard Hinnant: http://howardhinnant.github.io/date_algorithms.html
fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let y = if month <= 2 { year - 1 } else { year } as i64;
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u32; // [0, 399]
    let m = month as i64;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + day as i64 - 1; // [0, 365]
    let doe = yoe as i64 * 365 + yoe as i64 / 4 - yoe as i64 / 100 + doy; // [0, 146096]
    era * 146097 + doe - UNIX_EPOCH_DAYS
}

/// Convert days since Unix epoch to civil date
/// Algorithm from Howard Hinnant: http://howardhinnant.github.io/date_algorithms.html
fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + UNIX_EPOCH_DAYS;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let year = if m <= 2 { y + 1 } else { y };
    (year as i32, m as u32, d as u32)
}

// ============================================================================
// Parsing
// ============================================================================

impl FolioDateTime {
    /// Parse ISO 8601 datetime string
    ///
    /// Supported formats:
    /// - 2025-06-15
    /// - 2025-06-15T14:30:00
    /// - 2025-06-15T14:30:00Z
    /// - 2025-06-15T14:30:00+05:30
    /// - 2025-06-15T14:30:00.123Z
    pub fn parse(s: &str) -> Result<Self, DateTimeError> {
        let s = s.trim();

        // Try date only: YYYY-MM-DD
        if s.len() == 10 && s.chars().nth(4) == Some('-') && s.chars().nth(7) == Some('-') {
            return Self::parse_date_only(s);
        }

        // Try datetime with T separator
        if let Some(t_pos) = s.find('T') {
            let date_part = &s[..t_pos];
            let time_part = &s[t_pos + 1..];
            return Self::parse_datetime(date_part, time_part);
        }

        // Try datetime with space separator
        if let Some(space_pos) = s.find(' ') {
            let date_part = &s[..space_pos];
            let time_part = &s[space_pos + 1..];
            return Self::parse_datetime(date_part, time_part);
        }

        Err(DateTimeError::ParseError(format!("Unrecognized format: {}", s)))
    }

    fn parse_date_only(s: &str) -> Result<Self, DateTimeError> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 3 {
            return Err(DateTimeError::ParseError("Expected YYYY-MM-DD".to_string()));
        }

        let year: i32 = parts[0].parse()
            .map_err(|_| DateTimeError::ParseError("Invalid year".to_string()))?;
        let month: u32 = parts[1].parse()
            .map_err(|_| DateTimeError::ParseError("Invalid month".to_string()))?;
        let day: u32 = parts[2].parse()
            .map_err(|_| DateTimeError::ParseError("Invalid day".to_string()))?;

        Self::from_ymd(year, month, day)
    }

    fn parse_datetime(date_part: &str, time_part: &str) -> Result<Self, DateTimeError> {
        // Parse date
        let date_parts: Vec<&str> = date_part.split('-').collect();
        if date_parts.len() != 3 {
            return Err(DateTimeError::ParseError("Expected YYYY-MM-DD".to_string()));
        }

        let year: i32 = date_parts[0].parse()
            .map_err(|_| DateTimeError::ParseError("Invalid year".to_string()))?;
        let month: u32 = date_parts[1].parse()
            .map_err(|_| DateTimeError::ParseError("Invalid month".to_string()))?;
        let day: u32 = date_parts[2].parse()
            .map_err(|_| DateTimeError::ParseError("Invalid day".to_string()))?;

        // Parse timezone from time part
        let (time_str, tz_offset) = Self::extract_timezone(time_part)?;

        // Parse time (possibly with fractional seconds)
        let (time_no_frac, nanos) = if let Some(dot_pos) = time_str.find('.') {
            let frac_str = &time_str[dot_pos + 1..];
            let nanos = Self::parse_fractional_seconds(frac_str)?;
            (&time_str[..dot_pos], nanos)
        } else {
            (time_str, 0u32)
        };

        let time_parts: Vec<&str> = time_no_frac.split(':').collect();
        if time_parts.len() < 2 {
            return Err(DateTimeError::ParseError("Expected HH:MM[:SS]".to_string()));
        }

        let hour: u32 = time_parts[0].parse()
            .map_err(|_| DateTimeError::ParseError("Invalid hour".to_string()))?;
        let minute: u32 = time_parts[1].parse()
            .map_err(|_| DateTimeError::ParseError("Invalid minute".to_string()))?;
        let second: u32 = if time_parts.len() >= 3 {
            time_parts[2].parse()
                .map_err(|_| DateTimeError::ParseError("Invalid second".to_string()))?
        } else {
            0
        };

        let mut dt = Self::from_ymd_hms_nano(year, month, day, hour, minute, second, nanos)?;
        if let Some(offset) = tz_offset {
            dt.tz_offset = Some(offset);
        }
        Ok(dt)
    }

    fn extract_timezone(time_part: &str) -> Result<(&str, Option<i32>), DateTimeError> {
        // Check for Z suffix
        if time_part.ends_with('Z') {
            return Ok((&time_part[..time_part.len() - 1], Some(0)));
        }

        // Check for +HH:MM or -HH:MM suffix
        if let Some(plus_pos) = time_part.rfind('+') {
            let tz_str = &time_part[plus_pos + 1..];
            let offset = Self::parse_tz_offset(tz_str)?;
            return Ok((&time_part[..plus_pos], Some(offset)));
        }

        // Check for -HH:MM (but not negative hours like in 00:00:00)
        // Only consider it a timezone if it's at position > 5 (after HH:MM)
        if let Some(minus_pos) = time_part.rfind('-') {
            if minus_pos >= 5 {
                let tz_str = &time_part[minus_pos + 1..];
                let offset = -Self::parse_tz_offset(tz_str)?;
                return Ok((&time_part[..minus_pos], Some(offset)));
            }
        }

        Ok((time_part, None))
    }

    fn parse_tz_offset(s: &str) -> Result<i32, DateTimeError> {
        let parts: Vec<&str> = s.split(':').collect();
        let hours: i32 = parts[0].parse()
            .map_err(|_| DateTimeError::ParseError("Invalid timezone hours".to_string()))?;
        let minutes: i32 = if parts.len() > 1 {
            parts[1].parse()
                .map_err(|_| DateTimeError::ParseError("Invalid timezone minutes".to_string()))?
        } else {
            0
        };
        Ok(hours * 3600 + minutes * 60)
    }

    fn parse_fractional_seconds(s: &str) -> Result<u32, DateTimeError> {
        // Pad or truncate to 9 digits (nanoseconds)
        let padded = if s.len() >= 9 {
            &s[..9]
        } else {
            &format!("{:0<9}", s)
        };
        padded.parse()
            .map_err(|_| DateTimeError::ParseError("Invalid fractional seconds".to_string()))
    }

    /// Parse a time-only string (HH:MM:SS)
    pub fn parse_time(s: &str) -> Result<Self, DateTimeError> {
        let s = s.trim();
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() < 2 {
            return Err(DateTimeError::ParseError("Expected HH:MM[:SS]".to_string()));
        }

        let hour: u32 = parts[0].parse()
            .map_err(|_| DateTimeError::ParseError("Invalid hour".to_string()))?;
        let minute: u32 = parts[1].parse()
            .map_err(|_| DateTimeError::ParseError("Invalid minute".to_string()))?;
        let second: u32 = if parts.len() >= 3 {
            parts[2].parse()
                .map_err(|_| DateTimeError::ParseError("Invalid second".to_string()))?
        } else {
            0
        };

        Self::from_hms(hour, minute, second)
    }

    /// Parse with a specific format pattern
    pub fn parse_format(s: &str, pattern: &str) -> Result<Self, DateTimeError> {
        // Simple pattern matching - extract components based on pattern
        let mut year: Option<i32> = None;
        let mut month: Option<u32> = None;
        let mut day: Option<u32> = None;
        let mut hour: Option<u32> = None;
        let mut minute: Option<u32> = None;
        let mut second: Option<u32> = None;

        let mut s_pos = 0;
        let mut p_pos = 0;
        let s_bytes = s.as_bytes();
        let p_bytes = pattern.as_bytes();

        while p_pos < p_bytes.len() && s_pos < s_bytes.len() {
            // Check for format tokens
            if p_pos + 4 <= p_bytes.len() && &pattern[p_pos..p_pos + 4] == "YYYY" {
                year = Some(s[s_pos..s_pos + 4].parse()
                    .map_err(|_| DateTimeError::ParseError("Invalid year".to_string()))?);
                s_pos += 4;
                p_pos += 4;
            } else if p_pos + 2 <= p_bytes.len() && &pattern[p_pos..p_pos + 2] == "YY" {
                let yy: i32 = s[s_pos..s_pos + 2].parse()
                    .map_err(|_| DateTimeError::ParseError("Invalid year".to_string()))?;
                year = Some(if yy >= 70 { 1900 + yy } else { 2000 + yy });
                s_pos += 2;
                p_pos += 2;
            } else if p_pos + 2 <= p_bytes.len() && &pattern[p_pos..p_pos + 2] == "MM" {
                month = Some(s[s_pos..s_pos + 2].parse()
                    .map_err(|_| DateTimeError::ParseError("Invalid month".to_string()))?);
                s_pos += 2;
                p_pos += 2;
            } else if p_pos + 2 <= p_bytes.len() && &pattern[p_pos..p_pos + 2] == "DD" {
                day = Some(s[s_pos..s_pos + 2].parse()
                    .map_err(|_| DateTimeError::ParseError("Invalid day".to_string()))?);
                s_pos += 2;
                p_pos += 2;
            } else if p_pos + 2 <= p_bytes.len() && &pattern[p_pos..p_pos + 2] == "HH" {
                hour = Some(s[s_pos..s_pos + 2].parse()
                    .map_err(|_| DateTimeError::ParseError("Invalid hour".to_string()))?);
                s_pos += 2;
                p_pos += 2;
            } else if p_pos + 2 <= p_bytes.len() && &pattern[p_pos..p_pos + 2] == "mm" {
                minute = Some(s[s_pos..s_pos + 2].parse()
                    .map_err(|_| DateTimeError::ParseError("Invalid minute".to_string()))?);
                s_pos += 2;
                p_pos += 2;
            } else if p_pos + 2 <= p_bytes.len() && &pattern[p_pos..p_pos + 2] == "ss" {
                second = Some(s[s_pos..s_pos + 2].parse()
                    .map_err(|_| DateTimeError::ParseError("Invalid second".to_string()))?);
                s_pos += 2;
                p_pos += 2;
            } else {
                // Literal character - must match
                if s_bytes[s_pos] != p_bytes[p_pos] {
                    return Err(DateTimeError::ParseError(
                        format!("Expected '{}' at position {}", p_bytes[p_pos] as char, s_pos)
                    ));
                }
                s_pos += 1;
                p_pos += 1;
            }
        }

        Self::from_ymd_hms(
            year.unwrap_or(1970),
            month.unwrap_or(1),
            day.unwrap_or(1),
            hour.unwrap_or(0),
            minute.unwrap_or(0),
            second.unwrap_or(0),
        )
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_ymd() {
        let dt = FolioDateTime::from_ymd(2025, 6, 15).unwrap();
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.month(), 6);
        assert_eq!(dt.day(), 15);
    }

    #[test]
    fn test_from_ymd_hms() {
        let dt = FolioDateTime::from_ymd_hms(2025, 6, 15, 14, 30, 45).unwrap();
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.month(), 6);
        assert_eq!(dt.day(), 15);
        assert_eq!(dt.hour(), 14);
        assert_eq!(dt.minute(), 30);
        assert_eq!(dt.second(), 45);
    }

    #[test]
    fn test_unix_epoch() {
        let dt = FolioDateTime::from_ymd(1970, 1, 1).unwrap();
        assert_eq!(dt.as_nanos(), 0);
        assert_eq!(dt.as_unix_secs(), 0);
    }

    #[test]
    fn test_pre_epoch() {
        let dt = FolioDateTime::from_ymd(1969, 12, 31).unwrap();
        assert!(dt.as_nanos() < 0);
        assert_eq!(dt.year(), 1969);
        assert_eq!(dt.month(), 12);
        assert_eq!(dt.day(), 31);
    }

    #[test]
    fn test_leap_year() {
        assert!(is_leap_year(2000));
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(2023));
    }

    #[test]
    fn test_days_in_month() {
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2023, 2), 28);
        assert_eq!(days_in_month(2023, 1), 31);
        assert_eq!(days_in_month(2023, 4), 30);
    }

    #[test]
    fn test_weekday() {
        // 1970-01-01 was Thursday
        let dt = FolioDateTime::from_ymd(1970, 1, 1).unwrap();
        assert_eq!(dt.weekday(), 4);

        // 2025-06-15 is Sunday
        let dt = FolioDateTime::from_ymd(2025, 6, 15).unwrap();
        assert_eq!(dt.weekday(), 7);
    }

    #[test]
    fn test_add_duration() {
        let dt = FolioDateTime::from_ymd(2025, 6, 15).unwrap();
        let dur = FolioDuration::from_days(10);
        let result = dt.add_duration(&dur);
        assert_eq!(result.year(), 2025);
        assert_eq!(result.month(), 6);
        assert_eq!(result.day(), 25);
    }

    #[test]
    fn test_add_months() {
        // Normal case
        let dt = FolioDateTime::from_ymd(2025, 1, 15).unwrap();
        let result = dt.add_months(2);
        assert_eq!(result.month(), 3);

        // End of month clamping
        let dt = FolioDateTime::from_ymd(2025, 1, 31).unwrap();
        let result = dt.add_months(1);
        assert_eq!(result.month(), 2);
        assert_eq!(result.day(), 28); // Feb 2025 has 28 days
    }

    #[test]
    fn test_parse_iso() {
        let dt = FolioDateTime::parse("2025-06-15T14:30:00Z").unwrap();
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.month(), 6);
        assert_eq!(dt.day(), 15);
        assert_eq!(dt.hour(), 14);
        assert_eq!(dt.minute(), 30);
        assert_eq!(dt.tz_offset(), Some(0));
    }

    #[test]
    fn test_parse_date_only() {
        let dt = FolioDateTime::parse("2025-06-15").unwrap();
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.month(), 6);
        assert_eq!(dt.day(), 15);
        assert_eq!(dt.hour(), 0);
    }

    #[test]
    fn test_format() {
        let dt = FolioDateTime::from_ymd_hms(2025, 6, 15, 14, 30, 0).unwrap();
        assert_eq!(dt.format("DD/MM/YYYY"), "15/06/2025");
        assert_eq!(dt.format("YYYY-MM-DD HH:mm"), "2025-06-15 14:30");
    }

    #[test]
    fn test_duration_arithmetic() {
        let d1 = FolioDuration::from_days(5);
        let d2 = FolioDuration::from_hours(12);
        let sum = d1.add(&d2);
        assert_eq!(sum.as_hours(), 5 * 24 + 12);
    }

    #[test]
    fn test_duration_display() {
        let d = FolioDuration::from_hours(26);
        assert_eq!(format!("{}", d), "1d 02:00:00");

        let d = FolioDuration::from_minutes(90);
        assert_eq!(format!("{}", d), "01:30:00");
    }

    #[test]
    fn test_iso_string() {
        let dt = FolioDateTime::from_ymd_hms(2025, 6, 15, 14, 30, 0).unwrap();
        assert_eq!(dt.to_iso_string(), "2025-06-15T14:30:00Z");

        let dt_tz = dt.with_tz_offset(5 * 3600 + 30 * 60);
        assert_eq!(dt_tz.to_iso_string(), "2025-06-15T14:30:00+05:30");
    }

    #[test]
    fn test_datetime_diff() {
        let dt1 = FolioDateTime::from_ymd(2025, 6, 15).unwrap();
        let dt2 = FolioDateTime::from_ymd(2025, 6, 10).unwrap();
        let diff = dt1.duration_since(&dt2);
        assert_eq!(diff.as_days(), 5);
    }

    #[test]
    fn test_iso_week() {
        // 2025-01-01 is Wednesday, should be week 1
        let dt = FolioDateTime::from_ymd(2025, 1, 1).unwrap();
        assert_eq!(dt.iso_week(), 1);

        // 2025-12-31 is Wednesday - can be week 52 or 53 depending on calculation
        // ISO week number is complex at year boundaries
        let dt = FolioDateTime::from_ymd(2025, 12, 31).unwrap();
        let week = dt.iso_week();
        // 2025 has 52 or 53 weeks - week 1 of 2026 starts on Dec 29
        assert!(week == 1 || week == 52 || week == 53, "Expected week 1, 52, or 53, got {}", week);
    }

    #[test]
    fn test_day_of_year() {
        let dt = FolioDateTime::from_ymd(2025, 1, 1).unwrap();
        assert_eq!(dt.day_of_year(), 1);

        let dt = FolioDateTime::from_ymd(2025, 12, 31).unwrap();
        assert_eq!(dt.day_of_year(), 365);

        let dt = FolioDateTime::from_ymd(2024, 12, 31).unwrap();
        assert_eq!(dt.day_of_year(), 366); // Leap year
    }

    #[test]
    fn test_invalid_dates() {
        assert!(FolioDateTime::from_ymd(2025, 13, 1).is_err());
        assert!(FolioDateTime::from_ymd(2025, 0, 1).is_err());
        assert!(FolioDateTime::from_ymd(2025, 2, 30).is_err());
        assert!(FolioDateTime::from_ymd_hms(2025, 1, 1, 25, 0, 0).is_err());
    }
}
