use std::error::Error;
extern crate proc_macro2;

use quote::ToTokens;
use syn::{self, Type, DeriveInput, Attribute, meta::ParseNestedMeta, punctuated::Punctuated, Meta, Token, Path, Expr, parse_macro_input, Data, Field};
pub fn parse_syn() -> Result<(),Box<dyn Error>>{
    let t: Type = syn::parse_str("std::collections::HashMap<String, Value>")?;
    Ok(())
}


//This is the datastructure to model the mapper attributes below
/*#[mapper(dto="AnswerDto" , map=[("username:login",true),
("email",false)])]*/
pub const MAPPER : &'static str = "mapper";
pub const DTO: &'static str = "dto";
pub const MAP: &'static str = "map";
pub const EXCEPT: &'static str = "except";
pub const ALL_FIELD: &'static str = "include_all";

#[derive(Default,Debug)]
pub struct MapperAttr {
    pub dto_name: String,
    pub field_vec: Vec<(String,bool)>,
    all_included: bool,
    except: Vec<String>,   
}



//AST = Abstract Syntax Treee
pub struct ParsedAst<'a>{
    pub attributes : Vec<&'a Attribute>,
    pub struct_fields: Vec<&'a Field>,
}


impl<'a> ParsedAst<'a> {
    pub fn new(input : &'a DeriveInput) -> syn::Result<ParsedAst<'a>>{
        let mapper_attrs =  input.attrs.iter()
           .filter(| & a| a.path().is_ident(MAPPER))
           .collect::<Vec<&Attribute>>();

        let mut data_fields:Vec<&Field> = Vec::new();
        if let Data::Struct(data) = &input.data{
            data_fields = data.fields.iter().collect::<Vec<&Field>>();
        }
        
        syn::Result::Ok(Self { 
            attributes: mapper_attrs, 
            struct_fields: data_fields})
    }
}


pub fn extract_attributes(input : &DeriveInput) -> Result<(), Box<dyn Error> >{
    println!("Attrs Length={}",input.attrs.len());
    let mut has_mapper = false;
    let mapper_attrs = input.attrs.iter()
    .filter(| & a| a.path().is_ident(MAPPER))
    .collect::<Vec<&Attribute>>();

    //let struct_data = input.datata

    //let mapper_attrs = Box::new(mapper_attrs);
    if mapper_attrs.len() > 0 {
        println!("Mapper Entry found with name={}", MAPPER);
        has_mapper = true;
    }

    for attr in mapper_attrs {
        let nested = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated).unwrap();
        //println!("Nested : {:?}", nested);
        nested.iter().map(|this_meta|{
            //println!("inside nested.iter().map()");
            match this_meta {
                Meta::Path(path) => {
                    println!("inside Meta::Path");
                    //get_meta_attribute(path)
                    Some("Path".to_string())
                }
                Meta::List(meta_list) => {
                    println!("inside Meta::List");
                    Some("MetaList".to_string()) 
                }
                Meta::NameValue(name_value) => {
                    //println!("inside Meta::NameValue");
                    //let attribute = get_mapper_attribute(& name_value.path);
                    if let Some(attribute) = get_mapper_attribute(& name_value.path){
                        analyze_namevalue_expr(attribute, &name_value.value);
                        Some(attribute.to_string())
                    }
                    else {
                        None
                    }
                }
                _=> None
            }
        })
        .filter(Option::is_some)
        .for_each(|item| println!("attribute={:?}",item))
    }

    /*if !has_mapper{
        return Err("mapper declaration has not been found".into());
    }*/

    Ok(())
}


/*fn get_mapper_attribute(meta: & ParseNestedMeta<'_> ) -> Option<String> {
    meta.path.is_ident(DTO);
    let attr_names = vec![DTO, EXCEPT, ALL_FIELD];
    attr_names.iter()
    .filter(|& attr_name| meta.path.is_ident(attr_name))
    .map(|attr| Some(attr.to_string()))
    .collect()
}*/


fn get_mapper_attribute(path: &Path ) -> Option<&str> {
    let attr_names = vec![DTO, EXCEPT, ALL_FIELD, MAP];
    let attribute = attr_names.into_iter()
    .find(|& attr_name| path.is_ident(attr_name));
    attribute
}

fn analyze_namevalue_expr(attr_name: &str, expr : &Expr){
    match expr {
        Expr::Array(arr_expr) => {
            println!("Expression for {} is Array",attr_name)
        },
        Expr::Lit(lit_expr) => {  
            println!("Expression for {} is Lit", attr_name);
        },
        _ => println!("Expression for {} is neither Array nor Lit",attr_name),
    }
}



pub fn expr_namevalue_debugger(attr_name: &str, expr : &Expr){
    match expr {
        Expr::Array(_) => println!("Expression for {} is Array",attr_name),
        Expr::Assign(_) => println!("Expression for {} is Assign",attr_name),
        Expr::Async(_) => println!("Expression for {} is Async",attr_name),
        Expr::Await(_) => println!("Expression for {} is Await",attr_name),
        Expr::Binary(_) => println!("Expression for {} is Binary",attr_name),
        Expr::Block(_) => println!("Expression for {} is Block",attr_name),
        Expr::Break(_) => println!("Expression for {} is Break",attr_name),
        Expr::Call(_) => println!("Expression for {} is Call",attr_name),
        Expr::Cast(_) => println!("Expression for {} is Cast",attr_name),
        Expr::Closure(_) => println!("Expression for {} is Closure",attr_name),
        Expr::Const(_) => println!("Expression for {} is Const",attr_name),
        Expr::Continue(_) => println!("Expression for {} is Continue",attr_name),
        Expr::Field(_) => println!("Expression for {} is Field",attr_name),
        Expr::ForLoop(_) => println!("Expression for {} is ForLoop",attr_name),
        Expr::Group(_) => println!("Expression for {} is Group",attr_name),
        Expr::If(_) => println!("Expression for {} is If",attr_name),
        Expr::Index(_) => println!("Expression for {} is Index",attr_name),
        Expr::Infer(_) => println!("Expression for {} is Infer",attr_name),
        Expr::Let(_) => println!("Expression for {} is Let",attr_name),
        Expr::Lit(_) => println!("Expression for {} is Lit",attr_name),
        Expr::Loop(_) => println!("Expression for {} is Loop",attr_name),
        Expr::Macro(_) => println!("Expression for {} is Macro",attr_name),
        Expr::Match(_) => println!("Expression for {} is Match",attr_name),
        Expr::MethodCall(_) => println!("Expression for {} is MethodCall",attr_name),
        Expr::Paren(_) => println!("Expression for {} is Paren",attr_name),
        Expr::Path(_) => println!("Expression for {} is Path",attr_name),
        Expr::Range(_) => println!("Expression for {} is Range",attr_name),
        Expr::Reference(_) => println!("Expression for {} is Reference",attr_name),
        Expr::Repeat(_) => println!("Expression for {} is Repeat",attr_name),
        Expr::Return(_) => println!("Expression for {} is Return",attr_name),
        Expr::Struct(_) => println!("Expression for {} is Struct",attr_name),
        Expr::Try(_) => println!("Expression for {} is Try",attr_name),
        Expr::TryBlock(_) => println!("Expression for {} is TryBlock",attr_name),
        Expr::Tuple(_) => println!("Expression for {} is Tuple",attr_name),
        Expr::Unary(_) => println!("Expression for {} is Unary",attr_name),
        Expr::Unsafe(_) => println!("Expression for {} is Unsafe",attr_name),
        Expr::Verbatim(_) => println!("Expression for {} is Verbatim",attr_name),
        Expr::While(_) => println!("Expression for {} is While",attr_name),
        Expr::Yield(_) => println!("Expression for {} is Yield",attr_name),
        _ => println!("Expression for {} is None",attr_name),
        /*Expr::Tuple(ExprTuple) =>{
            println!("Expression for {} is Field",attr_name);
        }
        _ => println!("Expression is not field for {}",attr_name)*/
    }
}

fn extract_struct_fields(input: &DeriveInput){

}