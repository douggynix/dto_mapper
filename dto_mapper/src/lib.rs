extern crate proc_macro;
extern crate quote;

mod parser;

mod mapper_entry;

use mapper_entry::MapperEntry;

use proc_macro::TokenStream;
use syn::{DeriveInput, Data, Attribute, parse_macro_input};
use quote::quote;

//https://developerlife.com/2022/03/30/rust-proc-macro/
//https://astexplorer.net/
//https://towardsdatascience.com/nine-rules-for-creating-procedural-macros-in-rust-595aa476a7ff

#[proc_macro_derive(DtoMapper, attributes(mapper))]
pub fn dto_mapper_proc_macro(input: TokenStream) -> TokenStream{ 
    let input = parse_macro_input!(input as DeriveInput);
    let input = Box::new(input);
    //process_struct_data(input.clone());
    let _ = get_mapper_entries(input.clone());

    let expanded = quote! {
        fn check_mapper(){
            println!("Hello")
        }
    };

    expanded.into()
}

fn process_struct_data(input: Box<DeriveInput>) {
    if let Data::Struct(data) = input.clone().data{
        data.fields.iter().for_each(|field|{
            let ident_name = field.clone().ident.unwrap().to_string();
            println!("=====struct field={}",ident_name);
        });
    }
}

const MAPPER : &'static str = "mapper";

fn get_mapper_entries(input: Box<DeriveInput>) -> syn::Result<Vec<MapperEntry>>{
    let mut has_mapper = false;
    let mapper_attrs = input.attrs.iter()
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



