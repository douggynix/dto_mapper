pub fn remove_white_space(str: &String) -> String {
    str.as_str()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

pub fn isblank(str: &String) -> bool {
    remove_white_space(str).is_empty()
}
