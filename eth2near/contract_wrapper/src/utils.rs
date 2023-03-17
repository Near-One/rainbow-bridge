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
    let success = match status {
        FinalExecutionStatus::SuccessValue(value) => Some(value),
        _ => None,
    };
    success.and_then(|value| near_sdk::base64::decode(&value).ok())
}
