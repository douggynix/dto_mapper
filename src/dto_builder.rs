use std::{
  collections::{HashMap, HashSet},
  mem,
};

use ::syn::parse_str;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Path, PathSegment};

use crate::{
  mapper_entry::{MapValue, MapperEntry},
  struct_entry::{FieldEntry, StructEntry},
};

//this is to generate the dto structure along with the fields
pub fn generate_dto_stream(
  mapper_entries: &Vec<MapperEntry>,
  struct_entry: &StructEntry,
) -> Vec<TokenStream> {
  let dtos = mapper_entries.iter().map(|mapper_entry| {
    let mappings = build_fields(&struct_entry, &mapper_entry);
    let dto = format_ident!("{}", mapper_entry.dto.as_str());

    let derive_idents: Vec<syn::Ident> = mapper_entry
      .derive
      .iter()
      .map(|derive| {
        let ident: syn::Ident = format_ident!("{}", derive.as_str());
        ident
      })
      .collect();

    if !mapper_entry.no_builder {
      return quote! {
          #[derive( #(#derive_idents),* )]
          #[builder(default)]
          pub struct #dto {
              #(#mappings),*
          }
      };
    }

    //if no_builder=true return without a builder
    return quote! {
         #[derive( #(#derive_idents),* )]
        pub struct #dto {
            #(#mappings),*
        }
    };
  });

  dtos.collect()
}

//this is to build the implementation of Into trait for Dto and original structure
pub fn generate_impl(
  mapper_entries: &Vec<MapperEntry>,
  struct_entry: &StructEntry,
  is_dto: bool,
) -> Vec<TokenStream> {
  let impls: Vec<TokenStream> = mapper_entries
    .iter()
    .map(|mp_entry| {
      let mut init_fields = build_into_fields(&struct_entry, &mp_entry, is_dto);
      if is_dto {
        init_fields.extend(build_init_new_fields_token(&mp_entry));
      }

      let impl_stream: TokenStream;
      let struct_name = format_ident!("{}", &struct_entry.name.as_str());
      let dto = format_ident!("{}", mp_entry.dto.as_str());
      if is_dto {
        //convert struct into dto
        impl_stream = quote! {

            impl Into<#dto> for #struct_name{
                fn into(self) -> #dto {
                    #dto {
                        #(#init_fields),*
                    }
                }
            }

        };
      } else {
        //convert dto into original struct
        impl_stream = quote! {

            impl Into<#struct_name> for #dto{
                fn into(self) -> #struct_name {
                    #struct_name {
                        #(#init_fields),* ,
                        ..#struct_name::default()
                    }
                }
            }

        };
      }
      //println!("#######dto_impls = {}",impl_stream.to_string());
      impl_stream
    })
    .collect();

  impls
}

//this is a fundamental function to build the fields for Into trait traits such as field1 : field2
fn build_into_fields(
  st_entry: &StructEntry,
  mp_entry: &MapperEntry,
  is_dto: bool,
) -> Vec<TokenStream> {
  //we retrieve a hashmap of MapValue with key=source_field_name in the struct , and the the value as MapValue
  let map_fields = get_map_of_mapvalue(mp_entry);

  // Let us retrieve the ignore fields
  let ignore_fields = get_ignore_fields(mp_entry);

  // let selected_fields =
  //   get_selected_fields(&st_entry, &ignore_fields, &map_fields);
  let selected_fields =
    if mp_entry.exactly && map_fields.len() == 0 && ignore_fields.len() == 0 {
      get_all_fields(&st_entry)
    } else {
      get_selected_fields(&st_entry, &ignore_fields, &map_fields)
    };

  selected_fields
    .iter()
    .map(|field| {
      let mut name = format!("{}", field.field_name.to_string());
      //first we assume that is_dto = true  (building initialization macro for dto init field for Into trait)
      // build fields for initialization such that #left_name = self.#right_name
      //the right_name contains the struct field value
      let mut right_name = format_ident!("{}", name.as_str());
      // the left_name has the target dto field_name which will hold the value of right_name;
      let mut left_name = right_name.clone();
      //Let's check if the dto field(left_name) has been renamed
      if let Some(m_value) = map_fields.get(&name) {
        //let's rename the struct field if there is a mapping for it
        if let Some(ref new_name) = m_value.to_field {
          name = new_name.clone();
          left_name = format_ident!("{}", name.as_str());
        }

        //if build into is not for a dto but for a struct, let's swap left and right
        if !is_dto {
          mem::swap(&mut right_name, &mut left_name);
        }

        //if m_value.required = false(Option) , field.is_optional = false (straight_value)
        let is_optional = !m_value.required && !field.is_optional;

        if is_dto && is_optional {
          return quote! { #left_name: Some(self.#right_name) };
        } else if !is_dto && is_optional {
          return quote! { #left_name: self.#right_name.unwrap_or_default() };
        }
      }

      quote! { #left_name: self.#right_name}
    })
    .collect()
}

fn build_fields(
  st_entry: &StructEntry,
  mp_entry: &MapperEntry,
) -> Vec<TokenStream> {
  //we retrieve a hashmap of MapValue with key=source_field_name in the struct , and the the value as MapValue
  let map_fields = get_map_of_mapvalue(mp_entry);

  // Let us retrieve the ignore fields
  let ignore_fields = get_ignore_fields(mp_entry);

  let selected_fields =
    if mp_entry.exactly && map_fields.len() == 0 && ignore_fields.len() == 0 {
      get_all_fields(&st_entry)
    } else {
      get_selected_fields(&st_entry, &ignore_fields, &map_fields)
    };

  let tk_stream_iterator = selected_fields.iter().map(|field| {
    let mut name = format!("{}", field.field_name.to_string());
    let mut name_ident = format_ident!("{}", name.as_str());

    let ty = &field.field_type;
    if let Some(m_value) = map_fields.get(&name) {
      //let's rename the struct field if there is a mapping for it
      if let Some(ref new_name) = m_value.to_field {
        name = new_name.clone();
        name_ident = format_ident!("{}", name.as_str())
      }

      if !m_value.required && !field.is_optional {
        return quote! { pub #name_ident: Option<#ty> };
      }
    }

    quote! { pub #name_ident: #ty }
  });

  let mut struct_fields = tk_stream_iterator.collect::<Vec<TokenStream>>();
  let new_field_token = build_new_fields_token(&mp_entry);
  //println!("New Fields token = {:?}",new_field_token);
  struct_fields.extend(new_field_token);

  struct_fields
}

fn build_new_fields_token(mp_entry: &MapperEntry) -> Vec<TokenStream> {
  mp_entry
    .new_fields
    .iter()
    .map(|new_field| {
      let new_field_ident = format_ident!("{}", new_field.field_name.as_str());
      let field_type = &new_field.field_type;
      // Parse the field_type string into a syn::Type
      let type_tokens: syn::Type = parse_str(field_type)
        .unwrap_or_else(|_| panic!("Failed to parse type: {}", field_type));
      quote! { pub #new_field_ident: #type_tokens }
    })
    .collect()
}

fn build_init_new_fields_token(mp_entry: &MapperEntry) -> Vec<TokenStream> {
  mp_entry
    .new_fields
    .iter()
    .map(|new_field| {
      let name = format_ident!("{}", new_field.field_name.as_str());
      let expr = new_field.expression_value.clone();

      let token = quote! { #name: unstringify!(#expr) };
      println!(
        "name:{name} | expr: {expr} | TokenStream {}",
        token.to_string()
      );
      token
    })
    .collect()
}

fn get_ignore_fields(mp_entry: &MapperEntry) -> HashSet<String> {
  let ignore_fields: HashSet<String> = mp_entry
    .ignore
    .iter()
    .map(|elem| elem.to_string())
    .collect();
  ignore_fields
}

fn get_selected_fields(
  st_entry: &StructEntry,
  ignore_fields: &HashSet<String>,
  map_fields: &HashMap<String, MapValue>,
) -> Vec<FieldEntry> {
  let is_ignore = ignore_fields.len() > 0;
  st_entry
    .field_entries
    .iter()
    .filter(|&field| {
      (is_ignore && !ignore_fields.contains(&field.field_name.to_string()))
        || (!is_ignore
          && map_fields.contains_key(&field.field_name.to_string()))
    })
    .map(|f| f.clone())
    .collect()
}

fn get_all_fields(st_entry: &StructEntry) -> Vec<FieldEntry> {
  st_entry.field_entries.iter().map(|f| f.clone()).collect()
}

fn get_map_of_mapvalue(mp_entry: &MapperEntry) -> HashMap<String, MapValue> {
  let mut map_fields: HashMap<String, MapValue> = HashMap::new();
  mp_entry.map.iter().for_each(|mp_val| {
    map_fields.insert(mp_val.from_field.to_string(), mp_val.clone());
  });
  map_fields
}
