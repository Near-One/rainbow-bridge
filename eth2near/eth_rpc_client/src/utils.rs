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

pub fn read_json_file_from_data_dir(file_name: &str) -> std::string::String {
    std::fs::read_to_string(format!("data/{}", file_name)).expect("Unable to read file")
}
