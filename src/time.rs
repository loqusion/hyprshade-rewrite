#[cfg(not(feature = "mock-time"))]
pub fn now() -> chrono::DateTime<chrono::Local> {
    chrono::Local::now()
}

#[cfg(feature = "mock-time")]
pub fn now() -> chrono::DateTime<chrono::Local> {
    mock_now()
}

#[cfg(feature = "mock-time")]
fn mock_now() -> chrono::DateTime<chrono::Local> {
    let time: chrono::NaiveTime = std::env::var("__HYPRSHADE_MOCK_TIME")
        .unwrap()
        .parse()
        .unwrap();
    chrono::Local::now().with_time(time).unwrap()
}
