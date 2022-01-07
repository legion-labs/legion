use lgn_telemetry::{error, info};

pub mod api_gateway;

/// Run a lambda once locally by expecting a JSON event payload on the specified
/// reader, and writing the JSON event response to the specified writer.
///
/// # Examples
///
/// ```
/// use std::io::{Read, Write};
/// use serde::{Deserialize, Serialize};
///
/// use lambda_runtime::{Context, handler_fn};
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
/// pub async fn handler(e: Event, _: Context) -> anyhow::Result<Output> {
///     if e.name.is_empty() {
///         return Err(anyhow::anyhow!("name is empty"));
///     }
///
///     Ok(Output {
///         message: format!("Hello, {}!", e.name),
///     })
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), lambda_runtime::Error> {
///     let reader = "{\"name\": \"John Doe\"}".as_bytes();
///     let mut writer = Vec::<u8>::new();
///
///     lgn_online::aws::lambda::run_lambda_once(handler_fn(handler), reader, &mut writer).await?;
///     
///     assert_eq!(String::from_utf8_lossy(&writer), "{\"message\":\"Hello, John Doe!\"}");
///     
///     Ok(())
/// }
/// ```
pub async fn run_lambda_once<H, I, O, R, W>(
    handler: H,
    reader: R,
    writer: &mut W,
) -> Result<(), lambda_runtime::Error>
where
    H: lambda_runtime::Handler<I, O>,
    <H as lambda_runtime::Handler<I, O>>::Error: Into<lambda_runtime::Error>,
    I: for<'de> serde::Deserialize<'de>,
    O: serde::Serialize,
    R: std::io::Read,
    W: std::io::Write,
{
    let i: I = serde_json::from_reader(reader)?;
    let context = lambda_runtime::Context::default();
    let o = handler.call(i, context).await.map_err(Into::into)?;

    serde_json::to_writer(writer, &o).map_err(Into::into)
}

/// Run a lamba handler locally unless the `API` environment variable is set.
pub async fn run_lambda<H, I, O>(handler: H) -> Result<(), lambda_runtime::Error>
where
    H: lambda_runtime::Handler<I, O>,
    <H as lambda_runtime::Handler<I, O>>::Error: Into<lambda_runtime::Error> + std::fmt::Display,
    I: for<'de> serde::Deserialize<'de>,
    O: serde::Serialize,
{
    if !is_running_as_lambda() {
        info!("`AWS_LAMBDA_RUNTIME_API` is not set, running locally and expecting event as JSON on stdin");
        run_lambda_once(handler, std::io::stdin(), &mut std::io::stdout())
            .await
            .map_err(|err| {
                error!("Execution failed with: {}", err);
                err
            })
    } else {
        lambda_runtime::run(handler).await
    }
}

pub fn is_running_as_lambda() -> bool {
    std::env::var("AWS_LAMBDA_RUNTIME_API").is_ok()
}
