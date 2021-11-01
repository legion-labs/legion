mod handler;

use handler::handler;
use lambda_runtime::lambda;
use log::info;
use std::env;

fn main() -> anyhow::Result<()> {
    simple_logger::init_with_level(log::Level::Info)?;

    lambda_or_local!(handler)
}

#[macro_export]
macro_rules! lambda_or_local {
    ($handler:ident) => {
        if env::var("API").is_err() {
            info!("API is not set, running locally and expecting JSON on stdin");
            run_lambda_once($handler)
        } else {
            lambda!($handler);

            Ok(())
        }
    };
    ($handler:ident, $runtime:expr) => {
        if env::var("API").is_err() {
            info!("API is not set, running locally and expecting JSON on stdin");
            run_lambda_once($handler)
        } else {
            lambda!($handler, $runtime);

            Ok(())
        }
    };
}

fn run_lambda_once<H, I, O>(handler: H) -> anyhow::Result<()>
where
    H: Fn(I, lambda_runtime::Context) -> Result<O, lambda_runtime::error::HandlerError>
        + Send
        + Sync
        + 'static,
    I: for<'de> serde::Deserialize<'de>,
    O: serde::Serialize,
{
    let i: I = serde_json::from_reader(std::io::stdin())?;
    let context = lambda_runtime::Context::default();
    let o = handler(i, context)?;
    serde_json::to_writer(std::io::stdout(), &o).map_err(Into::into)
}
