use near_primitives::views::FinalExecutionStatus;

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

pub fn status_as_success_decoded(status: FinalExecutionStatus) -> Option<Vec<u8>> {
    match status {
        FinalExecutionStatus::SuccessValue(value) => Some(value),
        _ => None,
    }
}

pub fn new_near_rpc_client(timeout: Option<std::time::Duration>) -> reqwest::Client {
    let mut headers = reqwest::header::HeaderMap::with_capacity(2);
    headers.insert(
        reqwest::header::CONTENT_TYPE,
        reqwest::header::HeaderValue::from_static("application/json"),
    );

    let mut builder = reqwest::Client::builder().default_headers(headers);
    if let Some(timeout) = timeout {
        builder = builder.timeout(timeout);
    }
    builder.build().unwrap()
}
