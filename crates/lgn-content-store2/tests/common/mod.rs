mod asserts;
mod providers;

pub(crate) use asserts::*;
pub use providers::*;

pub fn get_random_localhost_addr() -> String {
    match std::net::TcpListener::bind("127.0.0.1:0") {
        Ok(stream) => format!("127.0.0.1:{}", stream.local_addr().unwrap().port()),
        Err(_) => "127.0.0.1:50051".to_string(),
    }
}
