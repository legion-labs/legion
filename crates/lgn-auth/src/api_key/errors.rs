use thiserror::Error;

use super::ApiKey;

#[derive(Error, Debug)]
pub enum Error {
    #[error("The API key is invalid")]
    InvalidApiKey(ApiKey),
}

pub type Result<T> = std::result::Result<T, Error>;

#[allow(clippy::fallible_impl_from)]
impl<ResBody> From<Error> for http::Response<ResBody>
where
    ResBody: Default,
{
    fn from(err: Error) -> Self {
        match err {
            Error::InvalidApiKey(_) => http::Response::builder()
                .status(http::StatusCode::UNAUTHORIZED)
                .body(Default::default())
                .unwrap(),
        }
    }
}
