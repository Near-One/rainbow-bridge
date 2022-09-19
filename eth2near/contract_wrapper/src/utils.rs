pub fn trim_quotes(s: String) -> String {
    let mut res_str = s;
    if (res_str.starts_with('"') && res_str.ends_with('"'))
        || (res_str.starts_with('\'') && res_str.ends_with('\''))
    {
        res_str.pop();
        res_str.remove(0);
    }

    res_str
}
