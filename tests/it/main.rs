mod common;
mod test_auto;
mod test_off;
mod test_on;
mod test_toggle;

#[cfg(not(feature = "mock-time"))]
compile_error!("Integration tests require the 'mock-time' feature to be enabled (try running with `cargo x test`)");
