use syn::{
    punctuated::Punctuated, spanned::Spanned, Attribute, Expr, ExprArray, ExprLit, ExprTuple, Lit,
    Meta, Token,
};

use crate::utils;
use crate::utils::isblank;

pub type MapTuple = (String, bool, Vec<String>);
#[derive(Debug, Default)]
pub struct MapperEntry {
    pub dto: String,
    pub map: Vec<MapValue>,
    pub ignore: Vec<String>,
    pub derive: Vec<String>,
    pub no_builder: bool,
    pub new_fields: Vec<NewField>,
    pub exactly: bool,
    pub macro_attr: Vec<String>,
}

//DataStructure for the type of mapper values found in each entry
#[derive(Debug, Default, Clone)]
pub struct MapValue {
    //Literal value are consited of properties with key=value
    // dto="MyDto" , ignore="true"
    pub from_field: String,
    pub to_field: Option<String>,
    pub macro_attr: Vec<String>,
    pub required: bool,
}

#[derive(Debug, Clone)]
pub struct NewField {
    pub field_name: String,
    pub field_type: String,
    //init_value is used compute this field value in the DTO during conversion with into()
    pub expression_value: String,
    pub attributes: Vec<String>,
}

impl NewField {
    pub fn new(name: &str, r#type: &str, init_expression: &str, attr: Option<Vec<String>>) -> Self {
        Self {
            field_name: name.to_string(),
            field_type: r#type.to_string(),
            expression_value: init_expression.to_string(),
            attributes: attr.unwrap_or(vec![]),
        }
    }
}

impl MapValue {
    fn new(map_tuple: &MapTuple) -> Self {
        let fields = map_tuple.0.as_str().split(":");
        let fields: Vec<&str> = fields.collect();

        let from_field = fields[0].to_string();
        let mut to_field: Option<String> = None;
        if fields.len() > 1 && !fields[1].trim().is_empty() {
            to_field = Some(fields[1].trim().to_string());
        }
        let macro_attr: Vec<String> = map_tuple.2.clone();

        let required = map_tuple.1;

        Self {
            from_field,
            to_field,
            required,
            macro_attr,
        }
    }
}

const DTO: &'static str = "dto";
const MAP: &'static str = "map";
const IGNORE: &'static str = "ignore";
//const ALL_FIELD: &'static str = "include_all";
const DERIVE: &'static str = "derive";
const WITHOUT_BUILDER: &'static str = "no_builder";
const NEW_FIELDS: &'static str = "new_fields";
const EXACTLY: &'static str = "exactly";
const MACRO_ATTR: &'static str = "macro_attr";

impl MapperEntry {
    pub fn build(attr: &Attribute) -> syn::Result<Self> {
        let nested = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        //println!("nested count={:?}",nested.iter().count());

        let mut mapper_entry = MapperEntry::default();
        //Mapper will always set a Default derive
        mapper_entry.derive.push("Default".to_string());

        //dto property is required
        let mut dto_prop: Option<String> = None;
        nested.iter().for_each(|meta| {
            if let Meta::NameValue(metaname) = meta {
                let ident = metaname.path.get_ident().unwrap();
                let keyname = utils::remove_white_space(&ident.to_string());
                //println!("keyname={}",keyname);

                //if we got literal values such as keyname="value" or keyname=true
                if let Expr::Lit(expr) = &metaname.value {
                    if keyname.eq_ignore_ascii_case(DTO) {
                        //we should read the string value
                        Self::parse_dto_attribute(&mut mapper_entry, &expr);
                        dto_prop = Some(mapper_entry.dto.to_string());
                    }

                    //
                    if keyname.eq_ignore_ascii_case(WITHOUT_BUILDER) {
                        Self::parse_no_builder_attribute(&mut mapper_entry, &expr);
                    }
                    if keyname.eq_ignore_ascii_case(EXACTLY) {
                        Self::parse_exactly_attribute(&mut mapper_entry, &expr);
                    }
                }

                if let Expr::Array(expr_arr) = &metaname.value {
                    //println!("{} array has {} elements",keyname,expr_arr.elems.iter().clone().count());
                    if keyname.eq_ignore_ascii_case(MAP) {
                        //map is a vec of tuples such as map=[("f1",true),("f2",false)]
                        Self::parse_map_attribute(&mut mapper_entry, expr_arr);
                    }

                    if keyname.eq_ignore_ascii_case(NEW_FIELDS) {
                        Self::parse_new_fields_attribute(&mut mapper_entry, expr_arr);
                    }
                    if keyname.eq_ignore_ascii_case(&MACRO_ATTR) {
                        Self::parse_macro_attr_attribute(&mut mapper_entry, expr_arr);
                    }

                    if keyname.eq_ignore_ascii_case(IGNORE) {
                        //ignore is a vec of string such as ignore=["val1","val2"]
                        Self::parse_ignore_attribute(&mut mapper_entry, expr_arr);
                    }
                }

                if let Expr::Tuple(tuple_expr) = &metaname.value {
                    //println!("keyname {} is Tuple of literal value",keyname);
                    if keyname.eq_ignore_ascii_case(DERIVE) {
                        Self::parse_derive_attribute(&mut mapper_entry, tuple_expr);
                    }
                }
            }
        });

        //dto property is required and must be checked
        match dto_prop {
            Some(val) if isblank(&val) => Err(syn::Error::new(
                attr.span(),
                "`dto` property is blank. It must not have whitespace",
            )),
            None => Err(syn::Error::new(
                attr.span(),
                "`dto` property is missing.It is required for mapper",
            )),
            _ => Ok(mapper_entry),
        }
    }

    fn parse_no_builder_attribute(mapper_entry: &mut MapperEntry, expr: &&ExprLit) {
        if let Lit::Bool(lit_bool) = &expr.lit {
            mapper_entry.no_builder = lit_bool.value();
        }
    }

    fn parse_exactly_attribute(mapper_entry: &mut MapperEntry, expr: &&ExprLit) {
        if let Lit::Bool(lit_bool) = &expr.lit {
            mapper_entry.exactly = lit_bool.value();
        }
    }

    fn parse_derive_attribute(mapper_entry: &mut MapperEntry, tuple_expr: &ExprTuple) {
        let derive_items = tuple_expr
            .elems
            .iter()
            .map(|elem_expr| {
                if let Expr::Path(path_exp) = &elem_expr {
                    let ident = path_exp.path.get_ident().unwrap();
                    let derive_obj: String = ident.to_string();
                    derive_obj
                } else {
                    "".to_string()
                }
            })
            .collect::<Vec<String>>();

        //Adding a builder by default if property isn't explicitly set to true
        if !mapper_entry.no_builder {
            mapper_entry.derive.push("Builder".into());
        }

        derive_items
            .iter()
            .filter(|&val| !val.eq("Default"))
            .map(|val| val.clone())
            .for_each(|val| mapper_entry.derive.push(val));
    }

    fn parse_dto_attribute(mapper_entry: &mut MapperEntry, expr: &ExprLit) {
        if let Lit::Str(lit_str) = &expr.lit {
            //println!("{}={}",keyname,lit_str.value())
            mapper_entry.dto = lit_str.value();
        }
    }

    fn parse_ignore_attribute(mapper_entry: &mut MapperEntry, expr_arr: &ExprArray) {
        let ignore_arr = Self::parse_array_of_string(expr_arr);
        //println!("{}={:?}",keyname, ignore_arr);
        //check if matt attribute is blank
        if ignore_arr.iter().filter(|&text| isblank(text)).count() > 0 {
            panic!("`{}` attribute must not be blank", IGNORE);
        };
        mapper_entry.ignore = ignore_arr;
    }

    fn parse_new_fields_attribute(mapper_entry: &mut MapperEntry, expr_arr: &ExprArray) {
        mapper_entry.new_fields = Self::parse_array_of_new_fields(expr_arr);
        //println!("{:?}",mapper_entry.new_fields);
        if mapper_entry.new_fields.len() == 0 {
            panic!(
                "`{}` attribute must not be empty or have odd number of elements",
                NEW_FIELDS
            );
        };
    }

    fn parse_macro_attr_attribute(mapper_entry: &mut MapperEntry, expr_arr: &ExprArray) {
        if mapper_entry
            .macro_attr
            .iter()
            .find(|attr| isblank(attr))
            .is_some()
        {
            panic!(
                "`{}` attribute must not be empty. Remove it if it's not needed. {:?}",
                MACRO_ATTR, mapper_entry.macro_attr
            );
        }
        mapper_entry.macro_attr = Self::parse_array_of_macro_attr(expr_arr);
        //println!("{:?}",mapper_entry.new_fields);
    }

    fn parse_map_attribute(mapper_entry: &mut MapperEntry, expr_arr: &ExprArray) {
        let map_tuples = Self::parse_array_of_tuple(expr_arr);
        //println!("{}={:?}",keyname,map_tuple);
        mapper_entry.map = map_tuples.iter().map(MapValue::new).collect();
        if mapper_entry
            .map
            .iter()
            .filter(|&m_val| isblank(&m_val.from_field))
            .count()
            > 0
        {
            panic!("`{}` attribute must not be blank", MAP);
        };
    }

    fn parse_array_of_macro_attr(expr_arr: &ExprArray) -> Vec<String> {
        let mut vec_tuple: Vec<String> = Vec::new();

        for elem in expr_arr.elems.iter() {
            Self::process_macro_attr(&mut vec_tuple, elem);
        }

        vec_tuple
    }
    fn process_macro_attr(vec_array: &mut Vec<String>, elem: &Expr) {
        if let Expr::Lit(content_lit) = elem {
            // eprintln!("content_lit={:?}", content_lit);
            if let Lit::Str(content) = &content_lit.lit {
                vec_array.push(content.value().trim().to_string());
            }
        }
        // eprintln!("================>");
        // eprintln!("vec_arr={:#?}", vec_array);
        // eprintln!("elem={:#?}", elem);
    }

    fn parse_array_of_tuple(expr_arr: &ExprArray) -> Vec<MapTuple> {
        let mut vec_tuple: Vec<(String, bool, Vec<String>)> = Vec::new();

        for elem in expr_arr.elems.iter() {
            if let Expr::Tuple(el_exp) = elem {
                //println!("{} content  is a Tuple",keyname);
                let mut str_val: Option<String> = None;
                let mut flag: Option<bool> = None;
                let mut attrs: Vec<String> = Vec::new();
                for content_expr in el_exp.elems.iter() {
                    if let Expr::Lit(content_lit) = content_expr {
                        if let Lit::Str(content) = &content_lit.lit {
                            //print!("valueStr={}",content.value());
                            str_val = utils::remove_white_space(&content.value()).into();
                        }

                        //There can only exist one boolean
                        if let Lit::Bool(content) = &content_lit.lit {
                            //print!("  valueBool={}",content.value());
                            flag = content.value.into();
                        }
                    }

                    //this will parse macro attributes for mapped fields
                    if let Expr::Array(content_arr) = content_expr {
                        attrs = Self::parse_array_of_macro_attr(&content_arr);
                    }
                }

                if str_val.is_some() && flag.is_some() {
                    let tuple: MapTuple = (str_val.unwrap(), flag.unwrap(), attrs);
                    vec_tuple.push(tuple);
                }
                //println!("");
            }
        }

        vec_tuple
    }

    fn parse_array_of_new_fields(expr_arr: &ExprArray) -> Vec<NewField> {
        let mut vec_tuple: Vec<NewField> = Vec::new();

        for elem in expr_arr.elems.iter() {
            Self::process_new_fields(&mut vec_tuple, elem);
        }

        vec_tuple
    }

    fn process_new_fields(mut vec_tuple: &mut Vec<NewField>, elem: &Expr) {
        if let Expr::Tuple(el_exp) = elem {
            let mut field_data: [Option<String>; 2] = [None, None];
            let mut attributes: Vec<String> = Vec::new();
            let total_passed_args = el_exp.elems.len();

            // eprintln!("===========>");
            // eprintln!("elem= {:#?}", el_exp.elems);
            // eprintln!("attrs= {:#?}", el_exp.attrs);
            // eprintln!("vec_tuple= {:#?}", vec_tuple);

            for (position, content_expr) in el_exp.elems.iter().enumerate() {
                // eprintln!("content_expr={:#?}", content_expr);
                // eprintln!("position={}", position);
                match position % 4 {
                    0 | 1 => {
                        // Field name or value
                        if let Expr::Lit(content_lit) = content_expr {
                            if let Lit::Str(content) = &content_lit.lit {
                                if let Some(str_val) =
                                    utils::remove_white_space(&content.value()).into()
                                {
                                    // eprintln!("str_val={}", str_val);
                                    field_data[position % 2] = Some(str_val);
                                }
                            }
                        }
                    }
                    2 => {
                        // Attributes array
                        attributes = extract_attributes(content_expr);
                        // eprintln!("attributes={:#?}", attributes);
                    }
                    _ => unreachable!(),
                }

                // Process the field when we have all provided arguments
                if total_passed_args - 1 == position {
                    if let (Some(field_decl), Some(field_value)) = (&field_data[0], &field_data[1])
                    {
                        if let Some(colon_position) = field_decl.find(":") {
                            // eprintln!("field_decl={}", field_decl);
                            // eprintln!("colon={}", colon_position);
                            let new_field_attr = match attributes.is_empty() {
                                true => None,
                                false => Some(attributes.clone()),
                            };

                            Self::insert_next_field_value(
                                &mut vec_tuple,
                                field_value.clone(),
                                field_decl,
                                &colon_position,
                                new_field_attr,
                            );
                        } else {
                            panic!("Missing `:` character for field declaration");
                        }
                    }
                    // Reset for next field
                    field_data = [None, None];
                    attributes.clear();
                }
            }

            // eprintln!("===========>");
        }
    }

    fn insert_next_field_value(
        vec_tuple: &mut Vec<NewField>,
        str_val: String,
        field_decl: &String,
        colon_position: &usize,
        attributes: Option<Vec<String>>,
    ) {
        if *colon_position == 0 {
            panic!("`:` cannot be the first character. Need to specify new fieldname for struct");
        }
        if *colon_position == field_decl.len() - 1 {
            panic!("Need to specify a type for the fieldname after `:`");
        }

        let field_name = &field_decl.as_str()[..*colon_position];
        let field_type = &field_decl.as_str()[*colon_position + 1..];

        vec_tuple.push(NewField::new(
            field_name,
            field_type,
            str_val.as_str(),
            attributes,
        ));
    }

    fn parse_array_of_string(expr_arr: &ExprArray) -> Vec<String> {
        let mut vec_str: Vec<String> = Vec::new();
        for elem in expr_arr.elems.iter() {
            if let Expr::Lit(lit_expr) = elem {
                //println!("{} content  is a String",keyname);
                if let Lit::Str(content) = &lit_expr.lit {
                    //fprint!("valueStr={}, ",content.value());
                    vec_str.push(utils::remove_white_space(&content.value()));
                }
            }
        }
        vec_str
    }
}

// Helper to extract extra attrs
fn extract_attributes(expr: &Expr) -> Vec<String> {
    if let Expr::Array(array_expr) = expr {
        array_expr
            .elems
            .iter()
            .filter_map(|elem| {
                if let Expr::Lit(lit_expr) = elem {
                    if let Lit::Str(str_lit) = &lit_expr.lit {
                        Some(str_lit.value().trim().to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    } else {
        Vec::new()
    }
}
