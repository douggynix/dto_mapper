use syn::{Attribute, LitStr, meta::ParseNestedMeta, parse_quote, Meta, punctuated::Punctuated, Token, Expr, Lit};


pub type MapTuple = (String,bool);
#[derive(Debug,Default)]
pub struct MapperEntry{
    pub dto: String,
    pub map : Vec<MapValue>,
    pub except:Vec<String>,
    pub include_all: bool,
    pub derive: Vec<String>,
}


//DataStructure for the type of mapper values found in each entry
#[derive(Debug)]
pub struct MapValue{
    //Literal value are consited of properties with key=value
    // dto="MyDto" , except="true"
    pub from_field: String,
    pub to_field: Option<String>,
    pub required: bool,
}

impl Default for MapValue{
    fn default() -> Self {
        Self { 
            from_field: "".into(), 
            to_field:None, 
            required: true 
        }
    }
}

impl MapValue {
    fn new(map_tuple: &MapTuple) -> Self{
        let fields = map_tuple.0.as_str().split(":");
        let fields: Vec<&str> = fields.collect();

        let from_field = fields[0].to_string();
        let mut to_field : Option<String> = None;
        if fields.len()>1 && ! fields[1].trim().is_empty(){
            to_field = Some(fields[1].trim().to_string());
        }

        let required = map_tuple.1;

        Self { from_field, to_field, required }
    }
}
const DTO: &'static str = "dto";
const MAP: &'static str = "map";
const EXCEPT: &'static str = "except";
const ALL_FIELD: &'static str = "include_all";
const DERIVE: &'static str = "derive";

impl MapperEntry{
    pub fn build(attr: &Attribute) -> syn::Result<Self>{
        
        let nested = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        println!("nested count={:?}",nested.iter().count());

        let mut mapper_entry = MapperEntry::default();

        nested.iter().for_each(|meta| {
            if let Meta::NameValue(metaname) = meta {
                let ident = metaname.path.get_ident().unwrap();
                let keyname = remove_white_space(& ident.to_string());
                //println!("keyname={}",keyname);

                //if we got literal values such as keyname="value" or keyname=true
                if let Expr::Lit(expr) =  & metaname.value{
                    if keyname.eq_ignore_ascii_case(DTO){
                        //we should read the string value
                        if let Lit::Str(lit_str) = & expr.lit {
                            //println!("{}={}",keyname,lit_str.value())
                            mapper_entry.dto = lit_str.value();
                        }
                    }

                    if keyname.eq_ignore_ascii_case(ALL_FIELD){
                        //we should read the string value
                        if let Lit::Bool(bool_lit) = & expr.lit {
                            //println!("{}={}",keyname,bool_lit.value())
                            mapper_entry.include_all = bool_lit.value;
                        }
                    }
                }

                if let Expr::Array(expr_arr) = & metaname.value{
                    //println!("{} array has {} elements",keyname,expr_arr.elems.iter().clone().count());

                    if keyname.eq_ignore_ascii_case(MAP){
                        //map is a vec of tuples such as map=[("f1",true),("f2",false)]
                        let map_tuples = Self::parse_array_of_tuple(expr_arr);
                        //println!("{}={:?}",keyname,map_tuple);
                        mapper_entry.map = map_tuples.iter()
                                            .map(MapValue::new).collect();
                    }
                    else if keyname.eq_ignore_ascii_case(EXCEPT){
                        //except is a vec of string such as except=["val1","val2"]
                        let except_arr = Self::parse_array_of_string(expr_arr);
                        //println!("{}={:?}",keyname, except_arr);
                        mapper_entry.except = except_arr;
                    }
                }


                if let Expr::Tuple(tuple_expr) = & metaname.value {
                    //println!("keyname {} is Tuple of literal value",keyname);
                    let derive_items = tuple_expr.elems
                        .iter()
                        .map(|elem_expr| {
                            if let Expr::Path(path_exp)= & elem_expr{
                                let ident = path_exp.path.get_ident().unwrap();
                                let derive_obj : String = ident.to_string();
                                derive_obj
                            }
                            else{
                                "".to_string()
                            }
                        }).collect::<Vec<String>>();

                mapper_entry.derive = derive_items;    
                }
            }
        });

        syn::Result::Ok(mapper_entry)
    }


    fn parse_array_of_tuple(expr_arr: &syn::ExprArray) -> Vec<MapTuple> {
        let mut vec_tuple: Vec<(String,bool)> = Vec::new();
    
        for elem in expr_arr.elems.iter(){
            if let Expr::Tuple(el_exp) = elem{
                //println!("{} content  is a Tuple",keyname);
                let mut str_val : Option<String> = None;
                let mut flag: Option<bool> = None;
                for content_expr in el_exp.elems.iter(){
                    if let Expr::Lit(content_lit) = content_expr{
                        if let Lit::Str(content) = & content_lit.lit{
                            //print!("valueStr={}",content.value());
                            str_val = remove_white_space(&content.value()).into();
                        }
    
                        if let Lit::Bool(content) = & content_lit.lit{
                            //print!("  valueBool={}",content.value());
                            flag = content.value.into();
                        }
                    }
                }
    
                if(str_val.is_some() && flag.is_some()){
                    let tuple : MapTuple = ( str_val.unwrap() , flag.unwrap() );
                    vec_tuple.push( tuple);
                }   
                //println!("");
            }
        }
    
        return vec_tuple
    }
    
    
    fn parse_array_of_string(expr_arr: &syn::ExprArray) -> Vec<String> {
        let mut vec_str : Vec<String>  = Vec::new();
        for elem in expr_arr.elems.iter(){
            if let Expr::Lit(lit_expr) = elem{
                //println!("{} content  is a String",keyname);
                if let Lit::Str(content) = & lit_expr.lit{
                    //print!("valueStr={}, ",content.value());
                    vec_str.push(remove_white_space(& content.value()));
                }
    
            }
        }
       return vec_str
    }

}


fn remove_white_space(str: &String) -> String{
    str.as_str().chars()
    .filter(|c| !c.is_whitespace())
    .collect()
}