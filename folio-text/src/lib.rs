//! Folio Text Functions Plugin
//!
//! String manipulation, parsing, and validation functions.
//! All functions follow the never-panic philosophy and return `Value::Error` on failure.

mod helpers;
mod transform;
mod search;
mod extract;
mod modify;
mod join;
mod parse;
mod validate;

use folio_plugin::PluginRegistry;

/// Load text functions into registry
pub fn load_text_library(registry: PluginRegistry) -> PluginRegistry {
    registry
        // Transform (13 functions)
        .with_function(transform::Upper)
        .with_function(transform::Lower)
        .with_function(transform::Capitalize)
        .with_function(transform::TitleCase)
        .with_function(transform::Trim)
        .with_function(transform::Ltrim)
        .with_function(transform::Rtrim)
        .with_function(transform::TrimChars)
        .with_function(transform::PadLeft)
        .with_function(transform::PadRight)
        .with_function(transform::Center)
        .with_function(transform::Repeat)
        .with_function(transform::Reverse)

        // Search (8 functions)
        .with_function(search::Contains)
        .with_function(search::ContainsAny)
        .with_function(search::StartsWith)
        .with_function(search::EndsWith)
        .with_function(search::IndexOf)
        .with_function(search::LastIndexOf)
        .with_function(search::CountMatches)
        .with_function(search::Matches)

        // Extract (13 functions)
        .with_function(extract::Len)
        .with_function(extract::ByteLen)
        .with_function(extract::CharAt)
        .with_function(extract::Substring)
        .with_function(extract::Left)
        .with_function(extract::Right)
        .with_function(extract::Mid)
        .with_function(extract::Split)
        .with_function(extract::SplitLines)
        .with_function(extract::Extract)
        .with_function(extract::ExtractGroup)
        .with_function(extract::ExtractAll)
        .with_function(extract::ExtractGroups)

        // Modify (9 functions)
        .with_function(modify::Replace)
        .with_function(modify::ReplaceAll)
        .with_function(modify::ReplaceRegex)
        .with_function(modify::Remove)
        .with_function(modify::RemoveRegex)
        .with_function(modify::Insert)
        .with_function(modify::Truncate)
        .with_function(modify::Ellipsis)
        .with_function(modify::Squeeze)

        // Join (4 functions)
        .with_function(join::Concat)
        .with_function(join::Join)
        .with_function(join::Format)
        .with_function(join::Template)

        // Parse (7 functions)
        .with_function(parse::ParseNumber)
        .with_function(parse::ParseInt)
        .with_function(parse::ParseFloat)
        .with_function(parse::ParseBool)
        .with_function(parse::ParseDate)
        .with_function(parse::ParseJson)
        .with_function(parse::ParseCsvLine)

        // Validate (11 functions)
        .with_function(validate::IsEmpty)
        .with_function(validate::IsBlank)
        .with_function(validate::IsNumeric)
        .with_function(validate::IsInteger)
        .with_function(validate::IsAlpha)
        .with_function(validate::IsAlphanumeric)
        .with_function(validate::IsEmail)
        .with_function(validate::IsUrl)
        .with_function(validate::IsUuid)
        .with_function(validate::IsPhone)
        .with_function(validate::Validate)
}
