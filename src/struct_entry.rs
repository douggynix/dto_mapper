use syn::Type;
use syn::{Data, DataStruct, DeriveInput, Fields};

//A StructEntry will hold the structure name and a list(vector) of FieldEntry
#[derive(Default)]
pub struct StructEntry {
    pub name: String,
    pub field_entries: Vec<FieldEntry>,
}

//FieldEntry will hold data about a field from a struct such that its name, and its type
#[derive(Clone)]
pub struct FieldEntry {
    pub field_name: String,
    pub field_type: Type,
    pub is_optional: bool,
}

impl StructEntry {
    pub fn build_struct_entry(input: Box<DeriveInput>) -> syn::Result<Self> {
        let struct_name = format!("{}", input.ident);
        let fields = if let Data::Struct(DataStruct {
            fields: Fields::Named(syn::FieldsNamed { ref named, .. }),
            ..
        }) = input.data
        {
            named
        } else {
            unimplemented!("Implemented only for structure not other type")
        };

        let struct_entries: Vec<FieldEntry> = fields
            .iter()
            .map(|field| {
                //let name = format!("{:?}",field.ident);
                let name = field.clone().ident.unwrap().to_string();

                let has_option = is_type_option(&field.ty);
                //println!("field={} is_optional={}",name,has_option);

                FieldEntry {
                    field_name: name,
                    field_type: field.ty.clone(),
                    is_optional: has_option,
                }
            })
            .collect();
        //let struct_entry = StructEntry::default();
        syn::Result::Ok(Self {
            name: struct_name,
            field_entries: struct_entries,
        })
    }
}

//https://github.com/jonhoo/proc-macro-workshop/blob/master/builder/src/lib.rs
//this code snippet is inspired from the builder workshop for syn library
//This is to figure out if type is an option
fn is_type_option(a_type: &Type) -> bool {
    if let Type::Path(ref p) = a_type {
        if p.path.segments.len() > 0 && p.path.segments[0].ident == "Option" {
            return true;
        }

        return false;
    }
    //return false by default
    false
}

fn _ty_inner_type<'a>(wrapper: &str, ty: &'a syn::Type) -> Option<&'a syn::Type> {
    if let syn::Type::Path(ref p) = ty {
        //println!("segment len: {}",p.path.segments.len());
        /*if p.path.segments.len()>0 {
            println!("Segment Ident={}", p.path.segments[0].ident);
        }*/
        if p.path.segments.len() != 1 || p.path.segments[0].ident != wrapper {
            return None;
        }

        if let syn::PathArguments::AngleBracketed(ref inner_ty) = p.path.segments[0].arguments {
            if inner_ty.args.len() != 1 {
                return None;
            }

            let inner_ty = inner_ty.args.first().unwrap();

            if let syn::GenericArgument::Type(ref t) = inner_ty {
                return Some(t);
            }
        }
    }
    None
}
