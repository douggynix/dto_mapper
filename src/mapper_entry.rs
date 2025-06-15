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
    pub from_field: String,
    pub to_field: Option<String>,
    pub macro_attr: Vec<String>,
    pub required: bool,
}

#[derive(Debug, Clone)]
pub struct NewField {
    pub field_name: String,
    pub field_type: String,
    pub expression_value: String,
    pub attributes: Vec<String>,
}

impl NewField {
    pub fn new(name: &str, r#type: &str, init_expression: &str, attr: Option<Vec<String>>) -> Self {
        Self {
            field_name: name.to_string(),
            field_type: r#type.to_string(),
            expression_value: init_expression.to_string(),
            attributes: attr.unwrap_or_default(),
        }
    }
}

impl MapValue {
    fn new(map_tuple: &MapTuple) -> Self {
        let fields: Vec<&str> = map_tuple.0.split(':').collect();

        let from_field = fields[0].to_string();
        let to_field = if fields.len() > 1 && !fields[1].trim().is_empty() {
            Some(fields[1].trim().to_string())
        } else {
            None
        };

        Self {
            from_field,
            to_field,
            required: map_tuple.1,
            macro_attr: map_tuple.2.clone(),
        }
    }
}

const DTO: &str = "dto";
const MAP: &str = "map";
const IGNORE: &str = "ignore";
const DERIVE: &str = "derive";
const WITHOUT_BUILDER: &str = "no_builder";
const NEW_FIELDS: &str = "new_fields";
const EXACTLY: &str = "exactly";
const MACRO_ATTR: &str = "macro_attr";

impl MapperEntry {
    pub fn build(attr: &Attribute) -> syn::Result<Self> {
        let nested = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;

        let mut mapper_entry = MapperEntry::default();
        // Mapper will always set a Default derive
        mapper_entry.derive.push("Default".to_string());

        // dto property is required
        let mut dto_prop: Option<String> = None;

        for meta in nested.iter() {
            if let Meta::NameValue(metaname) = meta {
                if let Some(ident) = metaname.path.get_ident() {
                    let keyname = utils::remove_white_space(&ident.to_string());

                    match &metaname.value {
                        Expr::Lit(expr) if keyname.eq_ignore_ascii_case(DTO) => {
                            Self::parse_dto_attribute(&mut mapper_entry, expr);
                            dto_prop = Some(mapper_entry.dto.clone());
                        }
                        Expr::Lit(expr) if keyname.eq_ignore_ascii_case(WITHOUT_BUILDER) => {
                            Self::parse_no_builder_attribute(&mut mapper_entry, expr);
                        }
                        Expr::Lit(expr) if keyname.eq_ignore_ascii_case(EXACTLY) => {
                            Self::parse_exactly_attribute(&mut mapper_entry, expr);
                        }
                        Expr::Array(expr_arr) if keyname.eq_ignore_ascii_case(MAP) => {
                            Self::parse_map_attribute(&mut mapper_entry, expr_arr);
                        }
                        Expr::Array(expr_arr) if keyname.eq_ignore_ascii_case(NEW_FIELDS) => {
                            Self::parse_new_fields_attribute(&mut mapper_entry, expr_arr);
                        }
                        Expr::Array(expr_arr) if keyname.eq_ignore_ascii_case(MACRO_ATTR) => {
                            Self::parse_macro_attr_attribute(&mut mapper_entry, expr_arr);
                        }
                        Expr::Array(expr_arr) if keyname.eq_ignore_ascii_case(IGNORE) => {
                            Self::parse_ignore_attribute(&mut mapper_entry, expr_arr);
                        }
                        Expr::Tuple(tuple_expr) if keyname.eq_ignore_ascii_case(DERIVE) => {
                            Self::parse_derive_attribute(&mut mapper_entry, tuple_expr);
                        }
                        _ => {}
                    }
                }
            }
        }

        // Validate dto property
        match dto_prop {
            Some(val) if isblank(&val) => Err(syn::Error::new(
                attr.span(),
                "`dto` property is blank. It must not have whitespace",
            )),
            None => Err(syn::Error::new(
                attr.span(),
                "`dto` property is missing. It is required for mapper",
            )),
            _ => Ok(mapper_entry),
        }
    }

    fn parse_no_builder_attribute(mapper_entry: &mut MapperEntry, expr: &ExprLit) {
        if let Lit::Bool(lit_bool) = &expr.lit {
            mapper_entry.no_builder = lit_bool.value();
        }
    }

    fn parse_exactly_attribute(mapper_entry: &mut MapperEntry, expr: &ExprLit) {
        if let Lit::Bool(lit_bool) = &expr.lit {
            mapper_entry.exactly = lit_bool.value();
        }
    }

    fn parse_derive_attribute(mapper_entry: &mut MapperEntry, tuple_expr: &ExprTuple) {
        let derive_items: Vec<String> = tuple_expr
            .elems
            .iter()
            .filter_map(|elem_expr| {
                if let Expr::Path(path_exp) = elem_expr {
                    path_exp.path.get_ident().map(|ident| ident.to_string())
                } else {
                    None
                }
            })
            .collect();

        // Add Builder by default if no_builder is false
        if !mapper_entry.no_builder {
            mapper_entry.derive.push("Builder".into());
        }

        // Add all derive items except Default (which is already added)
        derive_items
            .iter()
            .filter(|&val| val != "Default")
            .for_each(|val| mapper_entry.derive.push(val.clone()));
    }

    fn parse_dto_attribute(mapper_entry: &mut MapperEntry, expr: &ExprLit) {
        if let Lit::Str(lit_str) = &expr.lit {
            mapper_entry.dto = lit_str.value();
        }
    }

    fn parse_ignore_attribute(mapper_entry: &mut MapperEntry, expr_arr: &ExprArray) {
        let ignore_arr = Self::parse_array_of_string(expr_arr);

        if ignore_arr.iter().any(|text| isblank(text)) {
            panic!("`{}` attribute must not be blank", IGNORE);
        }

        mapper_entry.ignore = ignore_arr;
    }

    fn parse_new_fields_attribute(mapper_entry: &mut MapperEntry, expr_arr: &ExprArray) {
        mapper_entry.new_fields = Self::parse_array_of_new_fields(expr_arr);

        if mapper_entry.new_fields.is_empty() {
            panic!(
                "`{}` attribute must not be empty or have odd number of elements",
                NEW_FIELDS
            );
        }
    }

    fn parse_macro_attr_attribute(mapper_entry: &mut MapperEntry, expr_arr: &ExprArray) {
        mapper_entry.macro_attr = Self::parse_array_of_macro_attr(expr_arr);

        if mapper_entry.macro_attr.iter().any(|attr| isblank(attr)) {
            panic!(
                "`{}` attribute must not be empty. Remove it if it's not needed. {:?}",
                MACRO_ATTR, mapper_entry.macro_attr
            );
        }
    }

    fn parse_map_attribute(mapper_entry: &mut MapperEntry, expr_arr: &ExprArray) {
        let map_tuples = Self::parse_array_of_tuple(expr_arr);
        mapper_entry.map = map_tuples.iter().map(MapValue::new).collect();

        if mapper_entry
            .map
            .iter()
            .any(|m_val| isblank(&m_val.from_field))
        {
            panic!("`{}` attribute must not be blank", MAP);
        }
    }

    fn parse_array_of_macro_attr(expr_arr: &ExprArray) -> Vec<String> {
        extract_string_literals(expr_arr, true)
    }

    fn parse_array_of_tuple(expr_arr: &ExprArray) -> Vec<MapTuple> {
        expr_arr
            .elems
            .iter()
            .filter_map(|elem| match elem {
                Expr::Tuple(el_exp) => {
                    let mut str_val = None;
                    let mut flag = None;
                    let mut attrs = Vec::new();

                    for content_expr in &el_exp.elems {
                        match content_expr {
                            Expr::Lit(content_lit) => match &content_lit.lit {
                                Lit::Str(content) => {
                                    str_val = Some(utils::remove_white_space(&content.value()));
                                }
                                Lit::Bool(content) => {
                                    flag = Some(content.value);
                                }
                                _ => {}
                            },
                            Expr::Array(content_arr) => {
                                attrs = Self::parse_array_of_macro_attr(content_arr);
                            }
                            _ => {}
                        }
                    }

                    match (str_val, flag) {
                        (Some(s), Some(f)) => Some((s, f, attrs)),
                        _ => None,
                    }
                }
                _ => None,
            })
            .collect()
    }

    fn parse_array_of_new_fields(expr_arr: &ExprArray) -> Vec<NewField> {
        let mut new_fields = Vec::new();

        for elem in &expr_arr.elems {
            Self::process_new_fields(&mut new_fields, elem);
        }

        new_fields
    }

    fn process_new_fields(vec_tuple: &mut Vec<NewField>, elem: &Expr) {
        if let Expr::Tuple(el_exp) = elem {
            let mut field_data: [Option<String>; 2] = [None, None];
            let mut attributes: Vec<String> = Vec::new();
            let total_passed_args = el_exp.elems.len();

            for (position, content_expr) in el_exp.elems.iter().enumerate() {
                match position % 4 {
                    0 | 1 => {
                        // Field name or value
                        if let Expr::Lit(content_lit) = content_expr {
                            if let Lit::Str(content) = &content_lit.lit {
                                field_data[position % 2] =
                                    Some(utils::remove_white_space(&content.value()));
                            }
                        }
                    }
                    2 => {
                        // Attributes array
                        attributes = extract_attributes(content_expr);
                    }
                    _ => unreachable!(),
                }

                // Process the field when we have all provided arguments
                if total_passed_args - 1 == position {
                    Self::process_field_data(vec_tuple, &field_data, attributes);

                    // Reset for next field
                    field_data = [None, None];
                    attributes = Vec::new();
                }
            }
        }
    }

    fn process_field_data(
        vec_tuple: &mut Vec<NewField>,
        field_data: &[Option<String>; 2],
        attributes: Vec<String>,
    ) {
        if let (Some(field_decl), Some(field_value)) = (&field_data[0], &field_data[1]) {
            if let Some(colon_position) = field_decl.find(':') {
                let new_field_attr = if attributes.is_empty() {
                    None
                } else {
                    Some(attributes)
                };

                Self::insert_next_field_value(
                    vec_tuple,
                    field_value.clone(),
                    field_decl,
                    colon_position,
                    new_field_attr,
                );
            } else {
                panic!("Missing `:` character for field declaration");
            }
        }
    }

    fn insert_next_field_value(
        vec_tuple: &mut Vec<NewField>,
        value: String,
        field_decl: &str,
        colon_position: usize,
        attributes: Option<Vec<String>>,
    ) {
        if colon_position == 0 {
            panic!("`:` cannot be the first character. Need to specify new fieldname for struct");
        }
        if colon_position == field_decl.len() - 1 {
            panic!("Need to specify a type for the fieldname after `:`");
        }

        let field_name = &field_decl[..colon_position];
        let field_type = &field_decl[colon_position + 1..];

        vec_tuple.push(NewField::new(field_name, field_type, &value, attributes));
    }

    fn parse_array_of_string(expr_arr: &ExprArray) -> Vec<String> {
        extract_string_literals(expr_arr, false)
    }
}

// Helper to extract extra attrs
fn extract_attributes(expr: &Expr) -> Vec<String> {
    match expr {
        Expr::Array(array_expr) => array_expr
            .elems
            .iter()
            .filter_map(|elem| match elem {
                Expr::Lit(lit_expr) => match &lit_expr.lit {
                    Lit::Str(str_lit) => Some(str_lit.value().trim().to_string()),
                    _ => None,
                },
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn extract_string_literals(expr_arr: &ExprArray, trim_whitespace: bool) -> Vec<String> {
    expr_arr
        .elems
        .iter()
        .filter_map(|elem| match elem {
            Expr::Lit(lit_expr) => match &lit_expr.lit {
                Lit::Str(content) => {
                    let value = content.value();
                    Some(if trim_whitespace {
                        value.trim().to_string()
                    } else {
                        utils::remove_white_space(&value)
                    })
                }
                _ => None,
            },
            _ => None,
        })
        .collect()
}
