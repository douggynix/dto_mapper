use std::{
    collections::{HashMap, HashSet},
    mem,
};

use ::syn::parse_str;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    mapper_entry::{MapValue, MapperEntry},
    struct_entry::{FieldEntry, StructEntry},
};

pub struct DtoBuilder;

impl DtoBuilder {
    //this is to generate the dto structure along with the fields
    pub fn generate_dto_stream(
        mapper_entries: &Vec<MapperEntry>,
        struct_entry: &StructEntry,
    ) -> Vec<TokenStream> {
        mapper_entries
            .iter()
            .map(|mapper_entry| {
                let mappings = Self::build_fields(struct_entry, mapper_entry);
                let dto = format_ident!("{}", mapper_entry.dto.as_str());
                let derive_idents = Self::get_derive_idents(mapper_entry);
                let macro_attr = Self::parse_macro_attributes(mapper_entry);

                if !mapper_entry.no_builder {
                    quote! {
                        #[derive( #(#derive_idents),* )]
                        #[builder(default)]
                        #(#macro_attr)*
                        pub struct #dto {
                            #(#mappings),*
                        }
                    }
                } else {
                    quote! {
                        #[derive( #(#derive_idents),* )]
                        #(#macro_attr)*
                        pub struct #dto {
                            #(#mappings),*
                        }
                    }
                }
            })
            .collect()
    }

    //this is to build the implementation of Into trait for Dto and original structure
    pub fn generate_impl(
        mapper_entries: &Vec<MapperEntry>,
        struct_entry: &StructEntry,
        is_dto: bool,
    ) -> Vec<TokenStream> {
        mapper_entries
            .iter()
            .map(|mp_entry| {
                let mut init_fields = Self::build_into_fields(struct_entry, mp_entry, is_dto);
                if is_dto {
                    init_fields.extend(Self::build_init_new_fields_token(mp_entry));
                }

                let struct_name = format_ident!("{}", struct_entry.name.as_str());
                let dto = format_ident!("{}", mp_entry.dto.as_str());

                if is_dto {
                    quote! {
                        impl Into<#dto> for #struct_name{
                            fn into(self) -> #dto {
                                #dto {
                                    #(#init_fields),*
                                }
                            }
                        }
                    }
                } else {
                    quote! {
                        impl Into<#struct_name> for #dto{
                            fn into(self) -> #struct_name {
                                #struct_name {
                                    #(#init_fields),* ,
                                    ..#struct_name::default()
                                }
                            }
                        }
                    }
                }
            })
            .collect()
    }

    //this is a fundamental function to build the fields for Into trait traits such as field1 : field2
    fn build_into_fields(
        st_entry: &StructEntry,
        mp_entry: &MapperEntry,
        is_dto: bool,
    ) -> Vec<TokenStream> {
        let map_fields = Self::get_map_of_mapvalue(mp_entry);
        let ignore_fields = Self::get_ignore_fields(mp_entry);
        let selected_fields =
            Self::extract_selected_fields(st_entry, mp_entry, &map_fields, &ignore_fields);

        selected_fields
            .iter()
            .map(|field| Self::build_into_field(field, &map_fields, is_dto))
            .collect()
    }

    fn build_into_field(
        field: &FieldEntry,
        map_fields: &HashMap<String, MapValue>,
        is_dto: bool,
    ) -> TokenStream {
        let mut name = field.field_name.to_string();
        let mut right_name = format_ident!("{}", name.as_str());
        let mut left_name = right_name.clone();

        if let Some(m_value) = map_fields.get(&name) {
            if let Some(ref new_name) = m_value.to_field {
                name = new_name.clone();
                left_name = format_ident!("{}", name.as_str());
            }

            if !is_dto {
                mem::swap(&mut right_name, &mut left_name);
            }

            let is_optional = !m_value.required && !field.is_optional;

            if is_dto && is_optional {
                return quote! { #left_name: Some(self.#right_name) };
            } else if !is_dto && is_optional {
                return quote! { #left_name: self.#right_name.unwrap_or_default() };
            }
        }

        quote! { #left_name: self.#right_name }
    }

    fn extract_selected_fields(
        st_entry: &StructEntry,
        mp_entry: &MapperEntry,
        map_fields: &HashMap<String, MapValue>,
        ignore_fields: &HashSet<String>,
    ) -> Vec<FieldEntry> {
        if mp_entry.exactly && map_fields.is_empty() && ignore_fields.is_empty() {
            Self::get_all_fields(st_entry)
        } else {
            Self::get_selected_fields(st_entry, ignore_fields, map_fields)
        }
    }

    fn build_fields(st_entry: &StructEntry, mp_entry: &MapperEntry) -> Vec<TokenStream> {
        let map_fields = Self::get_map_of_mapvalue(mp_entry);
        let ignore_fields = Self::get_ignore_fields(mp_entry);
        let selected_fields =
            Self::extract_selected_fields(st_entry, mp_entry, &map_fields, &ignore_fields);

        let mut struct_fields = selected_fields
            .iter()
            .map(|field| Self::build_field(field, &map_fields))
            .collect::<Vec<TokenStream>>();

        struct_fields.extend(Self::build_new_fields_token(mp_entry));
        struct_fields
    }

    fn build_field(field: &FieldEntry, map_fields: &HashMap<String, MapValue>) -> TokenStream {
        let mut name = field.field_name.to_string();
        let mut name_ident = format_ident!("{}", name.as_str());
        let ty = &field.field_type;
        let mut attributes: Vec<TokenStream> = Vec::new();

        if let Some(m_value) = map_fields.get(&name) {
            if let Some(ref new_name) = m_value.to_field {
                name = new_name.clone();
                name_ident = format_ident!("{}", name.as_str());
            }

            attributes = m_value
                .macro_attr
                .iter()
                .map(|attr| parse_str(attr).unwrap())
                .collect();

            if !m_value.required && !field.is_optional {
                return quote! {
                    #(#attributes)*
                    pub #name_ident: Option<#ty>
                };
            }
        }

        quote! {
            #(#attributes)*
            pub #name_ident: #ty
        }
    }

    fn build_new_fields_token(mp_entry: &MapperEntry) -> Vec<TokenStream> {
        mp_entry
            .new_fields
            .iter()
            .map(|new_field| {
                let new_field_ident = format_ident!("{}", new_field.field_name.as_str());
                let field_type: syn::Type = parse_str(&new_field.field_type)
                    .unwrap_or_else(|_| panic!("Failed to parse type: {}", new_field.field_type));

                let attributes: Vec<TokenStream> = new_field
                    .attributes
                    .iter()
                    .map(|attr| parse_str(attr).unwrap())
                    .collect();

                quote! {
                    #(#attributes)*
                    pub #new_field_ident: #field_type
                }
            })
            .collect()
    }

    fn build_init_new_fields_token(mp_entry: &MapperEntry) -> Vec<TokenStream> {
        mp_entry
            .new_fields
            .iter()
            .map(|new_field| {
                let name = format_ident!("{}", new_field.field_name.as_str());
                let expr: syn::Expr = match parse_str(new_field.expression_value.as_str()) {
                    Ok(assign_expr) => assign_expr,
                    Err(expr_error) => {
                        panic!(
                            r#"Failed to parse new field '{}' expression value "{}" : {}"#,
                            new_field.field_name, new_field.expression_value, expr_error
                        );
                    }
                };

                quote! { #name: #expr }
            })
            .collect()
    }

    fn get_ignore_fields(mp_entry: &MapperEntry) -> HashSet<String> {
        mp_entry
            .ignore
            .iter()
            .map(|elem| elem.to_string())
            .collect()
    }

    fn get_selected_fields(
        st_entry: &StructEntry,
        ignore_fields: &HashSet<String>,
        map_fields: &HashMap<String, MapValue>,
    ) -> Vec<FieldEntry> {
        let is_ignore = !ignore_fields.is_empty();
        st_entry
            .field_entries
            .iter()
            .filter(|&field| {
                (is_ignore && !ignore_fields.contains(&field.field_name.to_string()))
                    || (!is_ignore && map_fields.contains_key(&field.field_name.to_string()))
            })
            .cloned()
            .collect()
    }

    fn get_all_fields(st_entry: &StructEntry) -> Vec<FieldEntry> {
        st_entry.field_entries.iter().cloned().collect()
    }

    fn get_map_of_mapvalue(mp_entry: &MapperEntry) -> HashMap<String, MapValue> {
        let mut map_fields: HashMap<String, MapValue> = HashMap::new();
        mp_entry.map.iter().for_each(|mp_val| {
            map_fields.insert(mp_val.from_field.to_string(), mp_val.clone());
        });
        map_fields
    }

    fn get_derive_idents(mapper_entry: &MapperEntry) -> Vec<syn::Ident> {
        mapper_entry
            .derive
            .iter()
            .map(|derive| format_ident!("{}", derive.as_str()))
            .collect()
    }

    fn parse_macro_attributes(mapper_entry: &MapperEntry) -> Vec<syn::Attribute> {
        mapper_entry
            .macro_attr
            .iter()
            .filter_map(|attr_str| {
                let stripped = attr_str.trim_start_matches("#[").trim_end_matches("]");
                if let Ok(meta) = syn::parse_str::<syn::Meta>(stripped) {
                    Some(syn::Attribute {
                        pound_token: syn::Token![#](proc_macro2::Span::call_site()),
                        style: syn::AttrStyle::Outer,
                        bracket_token: syn::token::Bracket(proc_macro2::Span::call_site()),
                        meta,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

// Keep these as free functions for backward compatibility
pub fn generate_dto_stream(
    mapper_entries: &Vec<MapperEntry>,
    struct_entry: &StructEntry,
) -> Vec<TokenStream> {
    DtoBuilder::generate_dto_stream(mapper_entries, struct_entry)
}

pub fn generate_impl(
    mapper_entries: &Vec<MapperEntry>,
    struct_entry: &StructEntry,
    is_dto: bool,
) -> Vec<TokenStream> {
    DtoBuilder::generate_impl(mapper_entries, struct_entry, is_dto)
}
