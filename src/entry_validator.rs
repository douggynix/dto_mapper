use std::collections::{HashMap, HashSet};

use crate::{mapper_entry::MapperEntry, struct_entry::StructEntry};

/// Represents specific field-related validation errors
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum FieldError {
    /// Indicates a duplicate field was found (with error message)
    DupField(String),
    /// Indicates a required field is missing (with error message)
    MissingField(String),
}

/// Represents validation errors that can occur during DTO mapping validation
#[derive(Debug)]
#[allow(dead_code)]
pub enum ValidationError {
    /// Errors related to mapper entry field validation
    MapperEntryError(Vec<FieldError>),
    /// Errors related to struct entry field validation
    StructEntryError(Vec<FieldError>),
    /// Duplicate DTO names were found
    DtoNameDuplicated(Vec<String>),
    /// A required property is missing from a mapper
    MissingPropertyError(String),
}

/// Validates all aspects of the DTO mapping configuration
///
/// # Arguments
///
/// * `st_entry` - The source struct entry to validate against
/// * `mp_entries` - The mapper entries to validate
///
/// # Returns
///
/// * `Ok(())` if validation passes
/// * `Err(ValidationError)` with the first validation error encountered
pub fn validate_entry_data(
    st_entry: &StructEntry,
    mp_entries: &[MapperEntry],
) -> Result<(), ValidationError> {
    // Run all validations in sequence, returning early on first error
    validate_mapper_entries(mp_entries)?;
    validate_struct_entry(st_entry, mp_entries)?;
    validate_dto_name(mp_entries)?;
    validate_map_ignore(mp_entries)?;

    Ok(())
}

/// Validates that each mapper entry has at least one mapping property
///
/// Each mapper entry must have either:
/// - At least one map attribute, or
/// - At least one ignore attribute, or
/// - The exactly flag set to true
fn validate_map_ignore(mp_entries: &[MapperEntry]) -> Result<(), ValidationError> {
    let invalid_entries: Vec<String> = mp_entries
        .iter()
        .filter(|entry| entry.map.is_empty() && entry.ignore.is_empty() && !entry.exactly)
        .map(|entry| entry.dto.clone())
        .collect();

    if !invalid_entries.is_empty() {
        return Err(ValidationError::MissingPropertyError(format!(
            "mapper requires a `map` or an `ignore` property for DTOs: {:?}",
            invalid_entries
        )));
    }

    Ok(())
}

/// Validates that DTO names are unique across all mapper entries
fn validate_dto_name(mp_entries: &[MapperEntry]) -> Result<(), ValidationError> {
    // Count occurrences of each DTO name
    let mut dto_counts = HashMap::new();
    for entry in mp_entries {
        *dto_counts.entry(entry.dto.clone()).or_insert(0) += 1;
    }

    // Collect names that appear more than once
    let duplicates: Vec<String> = dto_counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(name, _)| name)
        .collect();

    if !duplicates.is_empty() {
        return Err(ValidationError::DtoNameDuplicated(duplicates));
    }

    Ok(())
}

/// Validates that all fields referenced in mapper entries exist in the source struct
fn validate_struct_entry(
    st_entry: &StructEntry,
    mp_entries: &[MapperEntry],
) -> Result<(), ValidationError> {
    // Create a set of valid field names from the struct
    let valid_fields: HashSet<&String> = st_entry
        .field_entries
        .iter()
        .map(|f| &f.field_name)
        .collect();

    let mut errors = Vec::new();

    // Check each mapper entry for fields that don't exist in the struct
    for entry in mp_entries {
        let missing_fields: Vec<String> = entry
            .map
            .iter()
            .filter(|value| !valid_fields.contains(&value.from_field))
            .map(|value| value.from_field.clone())
            .collect();

        if !missing_fields.is_empty() {
            errors.push(FieldError::MissingField(format!(
                "{} field name doesn't exist in structure={}. List of wrong map field names: {:?}",
                entry.dto, st_entry.name, missing_fields
            )));
        }
    }

    if !errors.is_empty() {
        return Err(ValidationError::StructEntryError(errors));
    }

    Ok(())
}

/// Validates mapper entries for duplicate fields and other mapping errors
fn validate_mapper_entries(mp_entries: &[MapperEntry]) -> Result<(), ValidationError> {
    let mut errors = Vec::new();

    for entry in mp_entries {
        validate_single_mapper_entry(entry, &mut errors);
    }

    if !errors.is_empty() {
        return Err(ValidationError::MapperEntryError(errors));
    }

    Ok(())
}

/// Validates a single mapper entry for various field errors
fn validate_single_mapper_entry(entry: &MapperEntry, errors: &mut Vec<FieldError>) {
    // Count occurrences of source and destination fields
    let mut source_counts = HashMap::new();
    let mut dest_counts = HashMap::new();

    // Collect field counts
    for map_value in &entry.map {
        // Count source fields
        *source_counts.entry(&map_value.from_field).or_insert(0) += 1;

        // Count destination fields if they exist
        if let Some(ref to_field) = map_value.to_field {
            *dest_counts.entry(to_field).or_insert(0) += 1;
        }
    }

    // Check for various types of field errors
    check_overlapping_fields(entry, &source_counts, &dest_counts, errors);
    check_duplicate_source_fields(entry, &source_counts, errors);
    check_duplicate_dest_fields(entry, &dest_counts, errors);
}

/// Checks for fields used as both source and destination in a mapper entry
fn check_overlapping_fields(
    entry: &MapperEntry,
    source_counts: &HashMap<&String, u8>,
    dest_counts: &HashMap<&String, u8>,
    errors: &mut Vec<FieldError>,
) {
    let overlapping_fields: Vec<String> = dest_counts
        .keys()
        .filter(|key| source_counts.contains_key(*key))
        .map(|&key| key.clone())
        .collect();

    if !overlapping_fields.is_empty() {
        errors.push(FieldError::DupField(format!(
            "duplicate mapping destination keys found in dto={} entry: {:?}",
            entry.dto, overlapping_fields
        )));
    }
}

/// Extracts duplicate fields from a field count map
///
/// Returns a vector of field names that appear more than once in the map
fn get_duplicate_fields<'a>(field_counts: &HashMap<&'a String, u8>) -> Vec<String> {
    field_counts
        .iter()
        .filter(|(_, &count)| count > 1)
        .map(|(&field, _)| field.clone())
        .collect()
}

/// Checks for duplicate source fields in a mapper entry
fn check_duplicate_source_fields(
    entry: &MapperEntry,
    source_counts: &HashMap<&String, u8>,
    errors: &mut Vec<FieldError>,
) {
    let duplicate_sources = get_duplicate_fields(source_counts);

    if !duplicate_sources.is_empty() {
        errors.push(FieldError::DupField(format!(
            "duplicate source key names found in dto={} entry: {:?}",
            entry.dto, duplicate_sources
        )));
    }
}

/// Checks for duplicate destination fields in a mapper entry
fn check_duplicate_dest_fields(
    entry: &MapperEntry,
    dest_counts: &HashMap<&String, u8>,
    errors: &mut Vec<FieldError>,
) {
    // Find fields that appear more than once as destination fields
    let duplicate_destinations = get_duplicate_fields(dest_counts);

    // If duplicates were found, add an error
    if !duplicate_destinations.is_empty() {
        errors.push(FieldError::DupField(format!(
            "duplicate destination key names found in dto={} entry: {:?}",
            entry.dto, duplicate_destinations
        )));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapper_entry::MapValue;
    use crate::struct_entry::FieldEntry;
    use proc_macro2::TokenStream;
    use std::str::FromStr;
    use syn::Type;

    /// Creates a test struct entry with the given fields
    fn create_test_struct_entry(fields: Vec<&str>) -> StructEntry {
        let field_entries = fields
            .into_iter()
            .map(|name| FieldEntry {
                field_name: name.to_string(),
                field_type: Type::Verbatim(TokenStream::from_str("String").unwrap()),
                is_optional: false,
            })
            .collect();

        StructEntry {
            name: "TestStruct".to_string(),
            field_entries,
        }
    }

    /// Creates a test mapper entry with the given DTO name and mappings
    fn create_test_mapper_entry(
        dto_name: &str,
        mappings: Vec<(&str, Option<&str>, bool)>,
    ) -> MapperEntry {
        let map = mappings
            .into_iter()
            .map(|(from, to, required)| MapValue {
                from_field: from.to_string(),
                to_field: to.map(|s| s.to_string()),
                required,
                macro_attr: vec![],
            })
            .collect();

        MapperEntry {
            dto: dto_name.to_string(),
            map,
            ignore: vec![],
            derive: vec![],
            no_builder: false,
            new_fields: vec![],
            exactly: false,
            macro_attr: vec![],
        }
    }

    #[test]
    fn test_validate_dto_name_success() {
        let entries = vec![
            create_test_mapper_entry("Dto1", vec![("field1", None, true)]),
            create_test_mapper_entry("Dto2", vec![("field2", None, true)]),
        ];

        let result = validate_dto_name(&entries);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_dto_name_failure() {
        let entries = vec![
            create_test_mapper_entry("Dto1", vec![("field1", None, true)]),
            create_test_mapper_entry("Dto1", vec![("field2", None, true)]),
        ];

        let result = validate_dto_name(&entries);
        assert!(result.is_err());
        if let Err(ValidationError::DtoNameDuplicated(dups)) = result {
            assert_eq!(dups, vec!["Dto1"]);
        } else {
            panic!("Expected DtoNameDuplicated error");
        }
    }

    #[test]
    fn test_validate_struct_entry_success() {
        let struct_entry = create_test_struct_entry(vec!["field1", "field2"]);
        let mapper_entries = vec![
            create_test_mapper_entry("Dto1", vec![("field1", None, true)]),
            create_test_mapper_entry("Dto2", vec![("field2", None, true)]),
        ];

        let result = validate_struct_entry(&struct_entry, &mapper_entries);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_struct_entry_failure() {
        let struct_entry = create_test_struct_entry(vec!["field1", "field2"]);
        let mapper_entries = vec![
            create_test_mapper_entry("Dto1", vec![("field1", None, true)]),
            create_test_mapper_entry("Dto2", vec![("field3", None, true)]), // field3 doesn't exist
        ];

        let result = validate_struct_entry(&struct_entry, &mapper_entries);
        assert!(result.is_err());

        // Verify the specific error type and content
        if let Err(ValidationError::StructEntryError(errors)) = result {
            assert_eq!(errors.len(), 1);
            if let FieldError::MissingField(msg) = &errors[0] {
                assert!(msg.contains("field3"));
                assert!(msg.contains("Dto2"));
                assert!(msg.contains("TestStruct"));
            } else {
                panic!("Expected MissingField error");
            }
        } else {
            panic!("Expected StructEntryError");
        }
    }

    #[test]
    fn test_validate_map_ignore_success() {
        let entries = vec![
            create_test_mapper_entry("Dto1", vec![("field1", None, true)]),
            create_test_mapper_entry("Dto2", vec![]),
        ];

        // Set exactly=true for the second entry to make it valid
        let mut entries = entries;
        entries[1].exactly = true;

        let result = validate_map_ignore(&entries);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_map_ignore_failure() {
        // Create a mapper entry with no map, no ignore, and exactly=false
        let mut entry = create_test_mapper_entry("EmptyDto", vec![]);
        entry.exactly = false;

        let result = validate_map_ignore(&[entry]);
        assert!(result.is_err());

        if let Err(ValidationError::MissingPropertyError(msg)) = result {
            assert!(msg.contains("EmptyDto"));
        } else {
            panic!("Expected MissingPropertyError");
        }
    }

    #[test]
    fn test_validate_mapper_entries_success() {
        let entries = vec![create_test_mapper_entry(
            "Dto1",
            vec![("field1", None, true), ("field2", Some("renamed"), true)],
        )];

        let result = validate_mapper_entries(&entries);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_mapper_entries_failure_duplicate_source() {
        let entries = vec![create_test_mapper_entry(
            "Dto1",
            vec![
                ("field1", None, true),
                ("field1", Some("renamed"), true), // Duplicate source field
            ],
        )];

        let result = validate_mapper_entries(&entries);
        assert!(result.is_err());

        if let Err(ValidationError::MapperEntryError(errors)) = result {
            assert_eq!(errors.len(), 1);
            if let FieldError::DupField(msg) = &errors[0] {
                assert!(msg.contains("duplicate source"));
                assert!(msg.contains("field1"));
            } else {
                panic!("Expected DupField error");
            }
        } else {
            panic!("Expected MapperEntryError");
        }
    }

    #[test]
    fn test_validate_mapper_entries_failure_duplicate_dest() {
        let entries = vec![create_test_mapper_entry(
            "Dto1",
            vec![
                ("field1", Some("same_dest"), true),
                ("field2", Some("same_dest"), true), // Duplicate destination field
            ],
        )];

        let result = validate_mapper_entries(&entries);
        assert!(result.is_err());

        if let Err(ValidationError::MapperEntryError(errors)) = result {
            assert_eq!(errors.len(), 1);
            if let FieldError::DupField(msg) = &errors[0] {
                assert!(msg.contains("duplicate destination"));
                assert!(msg.contains("same_dest"));
            } else {
                panic!("Expected DupField error");
            }
        } else {
            panic!("Expected MapperEntryError");
        }
    }

    #[test]
    fn test_validate_mapper_entries_failure_overlapping_fields() {
        let entries = vec![create_test_mapper_entry(
            "Dto1",
            vec![
                ("overlap_field", None, true),
                ("field2", Some("overlap_field"), true), // Field used as both source and destination
            ],
        )];

        let result = validate_mapper_entries(&entries);
        assert!(result.is_err());

        if let Err(ValidationError::MapperEntryError(errors)) = result {
            assert_eq!(errors.len(), 1);
            if let FieldError::DupField(msg) = &errors[0] {
                assert!(msg.contains("duplicate mapping destination"));
                assert!(msg.contains("overlap_field"));
            } else {
                panic!("Expected DupField error");
            }
        } else {
            panic!("Expected MapperEntryError");
        }
    }

    #[test]
    fn test_validate_entry_data_success() {
        let struct_entry = create_test_struct_entry(vec!["field1", "field2"]);
        let mapper_entries = vec![
            create_test_mapper_entry("Dto1", vec![("field1", None, true)]),
            create_test_mapper_entry("Dto2", vec![("field2", None, true)]),
        ];

        let result = validate_entry_data(&struct_entry, &mapper_entries);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_entry_data_failure() {
        let struct_entry = create_test_struct_entry(vec!["field1", "field2"]);
        let mapper_entries = vec![
            create_test_mapper_entry("Dto1", vec![("field1", None, true)]),
            create_test_mapper_entry("Dto1", vec![("field2", None, true)]), // Duplicate DTO name
        ];

        let result = validate_entry_data(&struct_entry, &mapper_entries);
        assert!(result.is_err());

        // Should fail with DtoNameDuplicated error
        if let Err(ValidationError::DtoNameDuplicated(_)) = result {
            // Test passes
        } else {
            panic!("Expected DtoNameDuplicated error");
        }
    }
}
