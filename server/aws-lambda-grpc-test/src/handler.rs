use lambda_http::{IntoResponse, Request};
use lambda_runtime::{Context, Error};
use log::info;

pub async fn endpoint(request: Request, _context: Context) -> Result<impl IntoResponse, Error> {
    info!("request: {:?}", request);
    Ok(format!("{:?}", request).into_response())
}
