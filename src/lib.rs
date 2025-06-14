extern crate proc_macro;
extern crate quote;

#[allow(unused_imports)]
#[macro_use]
#[cfg(test)]
extern crate derive_builder;

mod dto_builder;
mod entry_validator;
mod mapper_entry;
mod struct_entry;
mod utils;

use entry_validator::validate_entry_data;
use mapper_entry::MapperEntry;
use proc_macro::TokenStream;
use quote::quote;
use struct_entry::StructEntry;
use syn::{parse_macro_input, Attribute, DeriveInput};

/// Procedural macro for generating Data Transfer Objects (DTOs) from structs.
///
/// This macro allows you to create DTOs with customized field mappings, new computed fields,
/// and various attributes for both the struct and its fields.
///

#[proc_macro_derive(DtoMapper, attributes(mapper))]
pub fn dto_mapper_proc_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let input = Box::new(input);

    // Parse the source struct
    let struct_entry = match process_struct_data(input.clone()) {
        Ok(st_entry) => st_entry,
        Err(error) => {
            panic!("Failed parsing structure entry with error: {}", error)
        }
    };

    // Extract mapper entries from attributes
    let mapper_entries = match get_mapper_entries(input.clone()) {
        Ok(map_entries) => map_entries,
        Err(error) => panic!("Error parsing mapper entries: {}", error),
    };

    // Validate the entries
    if let Err(error) = validate_entry_data(&struct_entry, &mapper_entries) {
        panic!("Failed validating mapper entries with error: {:?}", error);
    }

    // Generate DTOs and implementations
    let dtos = dto_builder::generate_dto_stream(&mapper_entries, &struct_entry);
    let dto_stream = quote! {
        #(#dtos)*
    };

    let dto_impls = dto_builder::generate_impl(&mapper_entries, &struct_entry, true);
    let struct_impls = dto_builder::generate_impl(&mapper_entries, &struct_entry, false);

    // Combine all generated code
    let expanded = quote! {
        // DTOs generated
        #dto_stream

        // DTO -> Struct implementations
        #(#dto_impls)*

        // Struct -> DTO implementations
        #(#struct_impls)*
    };

    expanded.into()
}

/// Processes the source struct data to extract field information
fn process_struct_data(input: Box<DeriveInput>) -> syn::Result<StructEntry> {
    StructEntry::build_struct_entry(input)
}

const MAPPER: &str = "mapper";

/// Extracts mapper entries from struct attributes
fn get_mapper_entries(input: Box<DeriveInput>) -> syn::Result<Vec<MapperEntry>> {
    let mapper_attrs: Vec<&Attribute> = input
        .attrs
        .iter()
        .filter(|&a| a.path().is_ident(MAPPER))
        .collect();

    let mut mapper_entries = Vec::new();

    for attr in mapper_attrs {
        match MapperEntry::build(attr) {
            Ok(mapper_entry) => mapper_entries.push(mapper_entry),
            Err(error) => panic!("Error parsing mapper entry: {:?}", error),
        }
    }

    Ok(mapper_entries)
}
