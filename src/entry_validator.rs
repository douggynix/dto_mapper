use std::collections::{HashMap, HashSet};

use crate::{mapper_entry::MapperEntry, struct_entry::StructEntry};

/// Represents specific field-related validation errors
#[derive(Debug)]
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
    // Run all validations and collect any errors
    let validation_results = [
        validate_mapper_entries(mp_entries),
        validate_struct_entry(st_entry, mp_entries),
        validate_dto_name(mp_entries),
        validate_map_ignore(mp_entries),
    ];
    
    // Return the first error encountered, if any
    for result in validation_results {
        if let Err(err) = result {
            return Err(err);
        }
    }
    
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
        return Err(ValidationError::MissingPropertyError(
            "mapper requires a `map` or an `ignore` property".to_string(),
        ));
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
    let valid_fields: HashSet<String> = st_entry
        .field_entries
        .iter()
        .map(|f| f.field_name.clone())
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
        check_duplicate_fields(entry, &mut errors);
    }

    if !errors.is_empty() {
        return Err(ValidationError::MapperEntryError(errors));
    }

    Ok(())
}

/// Checks for various types of duplicate field errors in a mapper entry
///
/// This function checks for:
/// - Fields used as both source and destination
/// - Duplicate source fields
/// - Duplicate destination fields
fn check_duplicate_fields(entry: &MapperEntry, errors: &mut Vec<FieldError>) {
    // Count occurrences of source and destination fields
    let mut source_counts = HashMap::new();
    let mut dest_counts = HashMap::new();
    
    for map_value in &entry.map {
        // Count source fields
        *source_counts.entry(map_value.from_field.clone()).or_insert(0) += 1;
        
        // Count destination fields if they exist
        if let Some(ref to_field) = map_value.to_field {
            *dest_counts.entry(to_field.clone()).or_insert(0) += 1;
        }
    }

    check_overlapping_fields(entry, &source_counts, &dest_counts, errors);
    check_duplicate_source_fields(entry, source_counts, errors);
    check_duplicate_dest_fields(entry, dest_counts, errors);
}

/// Checks for fields used as both source and destination in a mapper entry
fn check_overlapping_fields(
    entry: &MapperEntry,
    source_counts: &HashMap<String, u8>,
    dest_counts: &HashMap<String, u8>,
    errors: &mut Vec<FieldError>
) {
    let overlapping_fields: Vec<String> = dest_counts
        .keys()
        .filter(|key| source_counts.contains_key(*key))
        .cloned()
        .collect();

    if !overlapping_fields.is_empty() {
        errors.push(FieldError::DupField(format!(
            "duplicate mapping destination keys found in dto={} entry: {:?}",
            entry.dto, overlapping_fields
        )));
    }
}

/// Checks for duplicate source fields in a mapper entry
fn check_duplicate_source_fields(
    entry: &MapperEntry,
    source_counts: HashMap<String, u8>,
    errors: &mut Vec<FieldError>
) {
    let duplicate_sources: Vec<String> = source_counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(field, _)| field)
        .collect();

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
    dest_counts: HashMap<String, u8>,
    errors: &mut Vec<FieldError>
) {
    let duplicate_destinations: Vec<String> = dest_counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(field, _)| field)
        .collect();

    if !duplicate_destinations.is_empty() {
        errors.push(FieldError::DupField(format!(
            "duplicate destination key names found in dto={} entry: {:?}",
            entry.dto, duplicate_destinations
        )));
    }
}
