use log::error;
use serde::{Deserialize, Serialize};

use lambda_runtime::{error::HandlerError, Context};

#[derive(Deserialize, Clone)]
pub struct CustomEvent {
    #[serde(rename = "firstName")]
    first_name: String,
}

#[derive(Serialize, Clone)]
pub struct CustomOutput {
    message: String,
}

pub fn handler(e: CustomEvent, c: Context) -> Result<CustomOutput, HandlerError> {
    if e.first_name.is_empty() {
        error!("Empty first name in request {}", c.aws_request_id);
        return Err(c.new_error("Empty first name"));
    }

    Ok(CustomOutput {
        message: format!("Hello, {}!", e.first_name),
    })
}
