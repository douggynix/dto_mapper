extern crate proc_macro;
extern crate quote;

mod mapper_entry;
mod struct_entry;
mod entry_validator;
mod utils;


use entry_validator::validate_entry_data;
use mapper_entry::MapperEntry;

use proc_macro::TokenStream;
use struct_entry::StructEntry;
use syn::{DeriveInput, Attribute, parse_macro_input};
use quote::quote;

//https://developerlife.com/2022/03/30/rust-proc-macro/
//https://astexplorer.net/
//https://towardsdatascience.com/nine-rules-for-creating-procedural-macros-in-rust-595aa476a7ff

#[proc_macro_derive(DtoMapper, attributes(mapper))]
pub fn dto_mapper_proc_macro(input: TokenStream) -> TokenStream{ 
    let input = parse_macro_input!(input as DeriveInput);
    let input = Box::new(input);

    let struct_entry = match process_struct_data(input.clone()){
        Ok(st_entry) => st_entry,
        Err(error) => panic!("Failed parsing structure entry with error: {} ",error)
    };
    
    let mapper_entries = match get_mapper_entries( input.clone() ){
        Ok(map_entries) => map_entries,
        Err(error) => panic!("Error parsing mapper entries : {}",error)
    };

    
    if let Err(error) = validate_entry_data(&struct_entry, 
        mapper_entries){
            panic!("Failed Validating mapper entries with error : {:?}",error);
    }

    let expanded = quote! {
        fn check_mapper(){
            println!("Hello")
        }
    };

    expanded.into()
}


fn process_struct_data(input: Box<DeriveInput>) -> syn::Result<StructEntry> {
    return StructEntry::build_struct_entry(input)
}

const MAPPER : &'static str = "mapper";

fn get_mapper_entries(input: Box<DeriveInput>) -> syn::Result<Vec<MapperEntry>>{
    let mut has_mapper = false;
    let mapper_attrs: Vec<&Attribute> = input.attrs.iter()
    .filter(| & a| a.path().is_ident(MAPPER))
    .collect::<Vec<&Attribute>>();
    
    let mut mapper_entries: Vec<MapperEntry> = Vec::new();

    for attr in mapper_attrs{
        println!("=======MapperEntry===============");
        let build_result  = MapperEntry::build(attr);
        if let Ok(mapper_entry) = build_result{
            println!("{:?}",mapper_entry);
            mapper_entries.push(mapper_entry);
        }
        else if let Err(error) = build_result { 
            panic!("Error parsing mapper entry: {:?}", error)
        }
    }
    syn::Result::Ok(mapper_entries)
}



