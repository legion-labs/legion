pub mod api_gateway;

pub fn is_running_as_lambda() -> bool {
    std::env::var("AWS_LAMBDA_RUNTIME_API").is_ok()
}
