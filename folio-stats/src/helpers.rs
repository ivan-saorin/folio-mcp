//! Helper functions for statistical operations
//!
//! Common utilities for extracting and validating inputs.

use folio_core::{Number, Value, FolioError};

/// Extract numbers from arguments, handling both varargs and List
pub fn extract_numbers(args: &[Value]) -> Result<Vec<Number>, FolioError> {
    let mut numbers = Vec::new();

    for arg in args {
        match arg {
            Value::Number(n) => numbers.push(n.clone()),
            Value::List(list) => {
                for item in list {
                    match item {
                        Value::Number(n) => numbers.push(n.clone()),
                        Value::Error(e) => return Err(e.clone()),
                        other => return Err(FolioError::type_error("Number", other.type_name())),
                    }
                }
            }
            Value::Error(e) => return Err(e.clone()),
            other => return Err(FolioError::type_error("Number or List", other.type_name())),
        }
    }

    Ok(numbers)
}

/// Extract exactly two equal-length lists for bivariate functions
pub fn extract_two_lists(args: &[Value]) -> Result<(Vec<Number>, Vec<Number>), FolioError> {
    if args.len() != 2 {
        return Err(FolioError::new(
            "ARG_COUNT",
            format!("Expected 2 lists, got {} arguments", args.len()),
        ));
    }

    let x = extract_numbers(&args[0..1])?;
    let y = extract_numbers(&args[1..2])?;

    if x.len() != y.len() {
        return Err(FolioError::domain_error(format!(
            "Lists must have equal length: {} vs {}",
            x.len(),
            y.len()
        )));
    }

    Ok((x, y))
}

/// Require non-empty list
pub fn require_non_empty(numbers: &[Number], func: &str) -> Result<(), FolioError> {
    if numbers.is_empty() {
        return Err(FolioError::domain_error(format!(
            "{}() requires at least one value",
            func
        )));
    }
    Ok(())
}

/// Require minimum count
pub fn require_min_count(numbers: &[Number], min: usize, func: &str) -> Result<(), FolioError> {
    if numbers.len() < min {
        return Err(FolioError::domain_error(format!(
            "{}() requires at least {} values, got {}",
            func,
            min,
            numbers.len()
        )));
    }
    Ok(())
}

/// Calculate sum of numbers
pub fn sum(numbers: &[Number]) -> Number {
    numbers
        .iter()
        .fold(Number::from_i64(0), |acc, n| acc.add(n))
}

/// Calculate mean of numbers
pub fn mean(numbers: &[Number]) -> Result<Number, FolioError> {
    if numbers.is_empty() {
        return Err(FolioError::domain_error("Cannot calculate mean of empty list"));
    }
    let s = sum(numbers);
    let count = Number::from_i64(numbers.len() as i64);
    s.checked_div(&count).map_err(|e| e.into())
}

/// Calculate variance (sample or population)
pub fn variance_impl(numbers: &[Number], sample: bool) -> Result<Number, FolioError> {
    let n = numbers.len();
    if n == 0 {
        return Err(FolioError::domain_error("Cannot calculate variance of empty list"));
    }
    if sample && n < 2 {
        return Err(FolioError::domain_error(
            "Sample variance requires at least 2 values",
        ));
    }

    let m = mean(numbers)?;
    let mut ss = Number::from_i64(0);
    for x in numbers {
        let dev = x.sub(&m);
        ss = ss.add(&dev.mul(&dev));
    }

    let divisor = if sample {
        Number::from_i64((n - 1) as i64)
    } else {
        Number::from_i64(n as i64)
    };

    ss.checked_div(&divisor).map_err(|e| e.into())
}

/// Sort numbers (returns new sorted vector)
pub fn sorted(numbers: &[Number]) -> Vec<Number> {
    let mut sorted = numbers.to_vec();
    sorted.sort_by(|a, b| {
        let diff = a.sub(b);
        if diff.is_zero() {
            std::cmp::Ordering::Equal
        } else if diff.is_negative() {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });
    sorted
}

/// Calculate percentile using linear interpolation
pub fn percentile_impl(numbers: &[Number], p: &Number) -> Result<Number, FolioError> {
    if numbers.is_empty() {
        return Err(FolioError::domain_error("Cannot calculate percentile of empty list"));
    }

    // Validate p is in [0, 100]
    let zero = Number::from_i64(0);
    let hundred = Number::from_i64(100);
    if p.sub(&zero).is_negative() || p.sub(&hundred).is_negative() == false && !p.sub(&hundred).is_zero() {
        return Err(FolioError::domain_error(
            "Percentile must be between 0 and 100",
        ));
    }

    let sorted_nums = sorted(numbers);
    let n = sorted_nums.len();

    if n == 1 {
        return Ok(sorted_nums[0].clone());
    }

    // Convert percentile to rank
    // rank = p/100 * (n-1)
    let n_minus_1 = Number::from_i64((n - 1) as i64);
    let rank = p.mul(&n_minus_1).checked_div(&hundred)?;

    // Get floor and ceiling indices
    let floor_rank = rank.floor();
    let ceil_rank = rank.ceil();

    let floor_idx = floor_rank.to_i64().unwrap_or(0) as usize;
    let ceil_idx = ceil_rank.to_i64().unwrap_or(0) as usize;

    if floor_idx >= n {
        return Ok(sorted_nums[n - 1].clone());
    }
    if ceil_idx >= n {
        return Ok(sorted_nums[n - 1].clone());
    }

    if floor_idx == ceil_idx {
        return Ok(sorted_nums[floor_idx].clone());
    }

    // Linear interpolation
    let lower = &sorted_nums[floor_idx];
    let upper = &sorted_nums[ceil_idx];
    let frac = rank.sub(&floor_rank);
    let interpolated = lower.add(&upper.sub(lower).mul(&frac));

    Ok(interpolated)
}

/// Calculate ranks for a list (1-indexed, average for ties)
pub fn ranks(numbers: &[Number]) -> Vec<Number> {
    let n = numbers.len();
    if n == 0 {
        return vec![];
    }

    // Create pairs of (value, original_index)
    let mut indexed: Vec<(Number, usize)> = numbers.iter().cloned().enumerate().map(|(i, n)| (n, i)).collect();

    // Sort by value
    indexed.sort_by(|(a, _), (b, _)| {
        let diff = a.sub(b);
        if diff.is_zero() {
            std::cmp::Ordering::Equal
        } else if diff.is_negative() {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    let mut result = vec![Number::from_i64(0); n];
    let mut i = 0;

    while i < n {
        let mut j = i;
        // Find all elements with same value (ties)
        while j < n && indexed[j].0.sub(&indexed[i].0).is_zero() {
            j += 1;
        }

        // Average rank for ties: (i+1 + j) / 2
        let avg_rank = Number::from_i64((i + j + 1) as i64)
            .checked_div(&Number::from_i64(2))
            .unwrap_or(Number::from_i64(1));

        // Assign average rank to all tied elements
        for k in i..j {
            result[indexed[k].1] = avg_rank.clone();
        }

        i = j;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_numbers_list() {
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
        ])];
        let result = extract_numbers(&args).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_extract_numbers_varargs() {
        let args = vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
        ];
        let result = extract_numbers(&args).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_sum() {
        let numbers = vec![
            Number::from_i64(1),
            Number::from_i64(2),
            Number::from_i64(3),
        ];
        let result = sum(&numbers);
        assert_eq!(result.to_i64(), Some(6));
    }

    #[test]
    fn test_mean() {
        let numbers = vec![
            Number::from_i64(2),
            Number::from_i64(4),
            Number::from_i64(6),
        ];
        let result = mean(&numbers).unwrap();
        assert_eq!(result.to_i64(), Some(4));
    }

    #[test]
    fn test_sorted() {
        let numbers = vec![
            Number::from_i64(3),
            Number::from_i64(1),
            Number::from_i64(2),
        ];
        let result = sorted(&numbers);
        assert_eq!(result[0].to_i64(), Some(1));
        assert_eq!(result[1].to_i64(), Some(2));
        assert_eq!(result[2].to_i64(), Some(3));
    }
}
