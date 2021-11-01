pub mod api_gateway;

/// Run a lambda once locally by expecting a JSON event payload on the specified reader, and writing
/// the JSON event response to the specified writer.
///
/// # Examples
///
/// ```
/// use std::io::{Read, Write};
/// use serde::{Deserialize, Serialize};
///
/// use lambda_runtime::{error::HandlerError, Context};
///
/// #[derive(Deserialize, Clone)]
/// pub struct Event {
///     name: String,
/// }
///
/// #[derive(Serialize, Clone)]
/// pub struct Output {
///     message: String,
/// }
///
/// pub fn handler(e: Event, c: Context) -> Result<Output, HandlerError> {
///     if e.name.is_empty() {
///         return Err(c.new_error("Empty name name"));
///     }
///
///     Ok(Output {
///         message: format!("Hello, {}!", e.name),
///     })
/// }
///
/// fn main() -> anyhow::Result<()> {
///     let reader = "{\"name\": \"John Doe\"}".as_bytes();
///     let mut writer = Vec::<u8>::new();
///
///     legion_aws::lambda::run_lambda_once(handler, reader, &mut writer)?;
///     
///     assert_eq!(String::from_utf8_lossy(&writer), "{\"message\":\"Hello, John Doe!\"}");
///     
///     Ok(())
/// }
/// ```
pub fn run_lambda_once<H, I, O, R, W>(handler: H, reader: R, writer: &mut W) -> anyhow::Result<()>
where
    H: Fn(I, lambda_runtime::Context) -> Result<O, lambda_runtime::error::HandlerError>
        + Send
        + Sync
        + 'static,
    I: for<'de> serde::Deserialize<'de>,
    O: serde::Serialize,
    R: std::io::Read,
    W: std::io::Write,
{
    let i: I = serde_json::from_reader(reader)?;
    let context = lambda_runtime::Context::default();
    let o = handler(i, context)?;
    serde_json::to_writer(writer, &o).map_err(Into::into)
}

/// Run a lamba function locally on stdin/stdout unless `API` is set in the environment.
#[macro_export]
macro_rules! lambda {
    ($handler:ident) => {
        if std::env::var("API").is_err() {
            log::info!("API is not set, running locally and expecting event as JSON on stdin");
            legion_aws::lambda::run_lambda_once($handler, std::io::stdin(), &mut std::io::stdout())
        } else {
            lambda_runtime::lambda!($handler);

            Ok(())
        }
    };
    ($handler:ident, $runtime:expr) => {
        if std::env::var("API").is_err() {
            log::info!("API is not set, running locally and expecting event as JSON on stdin");
            legion_aws::lambda::run_lambda_once($handler, std::io::stdin(), &mut std::io::stdout())
        } else {
            lambda_runtime::lambda!($handler, $runtime);

            Ok(())
        }
    };
}

pub use lambda;
