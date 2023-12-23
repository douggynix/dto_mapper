use proc_macro2::Span;

pub fn remove_white_space(str: &String) -> String {
    str.as_str()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

pub fn isblank(str: &String) -> bool {
    remove_white_space(str).is_empty()
}

pub fn create_ident(name: &str) -> syn::Ident {
    syn::Ident::new(name, Span::call_site())
}
