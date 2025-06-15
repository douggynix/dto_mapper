use syn::{Data, DataStruct, DeriveInput, Fields, Type};

/// Represents a struct with its name and fields
#[derive(Default, Debug)]
pub struct StructEntry {
    pub name: String,
    pub field_entries: Vec<FieldEntry>,
}

/// Represents a field in a struct with its name, type, and optionality
#[derive(Clone, Debug)]
pub struct FieldEntry {
    pub field_name: String,
    pub field_type: Type,
    pub is_optional: bool,
}

impl StructEntry {
    /// Builds a StructEntry from a DeriveInput
    ///
    /// # Arguments
    ///
    /// * `input` - The parsed derive input
    ///
    /// # Returns
    ///
    /// * `syn::Result<Self>` - The constructed StructEntry or an error
    pub fn build_struct_entry(input: Box<DeriveInput>) -> syn::Result<Self> {
        let struct_name = input.ident.to_string();
        let fields = Self::extract_fields(&input)?;

        let field_entries = fields
            .iter()
            .map(|field| {
                let field_name = field.ident.clone().unwrap().to_string();
                let is_optional = is_type_option(&field.ty);

                FieldEntry {
                    field_name,
                    field_type: field.ty.clone(),
                    is_optional,
                }
            })
            .collect();

        Ok(Self {
            name: struct_name,
            field_entries,
        })
    }

    /// Extracts fields from a DeriveInput
    ///
    /// # Arguments
    ///
    /// * `input` - The parsed derive input
    ///
    /// # Returns
    ///
    /// * `syn::Result<&Punctuated<Field, Comma>>` - The fields or an error
    fn extract_fields(
        input: &DeriveInput,
    ) -> syn::Result<&syn::punctuated::Punctuated<syn::Field, syn::token::Comma>> {
        match &input.data {
            Data::Struct(DataStruct {
                fields: Fields::Named(named_fields),
                ..
            }) => Ok(&named_fields.named),
            _ => Err(syn::Error::new_spanned(
                input.ident.clone(),
                "DtoMapper only supports structs with named fields",
            )),
        }
    }
}

/// Determines if a type is an Option<T>
///
/// # Arguments
///
/// * `a_type` - The type to check
///
/// # Returns
///
/// * `bool` - True if the type is an Option<T>, false otherwise
pub fn is_type_option(a_type: &Type) -> bool {
    if let Type::Path(path_type) = a_type {
        if let Some(segment) = path_type.path.segments.first() {
            if segment.ident == "Option" {
                return true;
            }
        }
    }

    false
}
