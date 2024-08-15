#[cfg(not(feature = "_mock_time"))]
pub fn now() -> chrono::DateTime<chrono::Local> {
    chrono::Local::now()
}

#[cfg(feature = "_mock_time")]
pub fn now() -> chrono::DateTime<chrono::Local> {
    mock_now()
}

#[cfg(feature = "_mock_time")]
fn mock_now() -> chrono::DateTime<chrono::Local> {
    let mock_time_str = std::env::var("__HYPRSHADE_MOCK_TIME")
        .unwrap_or_else(|err| panic!("reading __HYPRSHADE_MOCK_TIME: {err}"));
    let time: chrono::NaiveTime = mock_time_str
        .parse()
        .unwrap_or_else(|err| panic!("parsing '{mock_time_str}': {err}"));
    chrono::Local::now().with_time(time).unwrap()
}
