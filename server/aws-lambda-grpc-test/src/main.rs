use lambda_http::handler;
use legion_aws::lambda::run_lambda;

mod handler;

use handler::endpoint;

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    simple_logger::init_with_level(log::Level::Info)?;

    let handler = handler(endpoint);

    run_lambda(handler).await
}
