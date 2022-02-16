use clap::Parser;
use url::Url;

const DEFAULT_ISSUER_URL: &str =
    "https://cognito-idp.ca-central-1.amazonaws.com/ca-central-1_SkZKDimWz";

const DEFAULT_APPLICATION_NAME: &str = "legion-editor";

const DEFAULT_CLIENT_ID: &str = "5m58nrjfv6kr144prif9jk62di";

const DEFAULT_REDIRECT_URI: &str = "http://localhost:3000/";

const DEFAULT_PORT: &str = "5000";

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct Config {
    /// The application name
    #[clap(long, default_value = DEFAULT_APPLICATION_NAME)]
    pub application_name: String,

    /// The issuer (i.e. oauth provider) URL
    #[clap(long, default_value = DEFAULT_ISSUER_URL)]
    pub issuer_url: Url,

    /// The client id as registered in the issuer
    #[clap(long, default_value = DEFAULT_CLIENT_ID)]
    pub client_id: String,

    /// The redirect uri (must be registered in the issuer)
    #[clap(long, default_value = DEFAULT_REDIRECT_URI)]
    pub redirect_uri: Url,

    /// The port used by the temporary server to retrieve the authentication code
    #[clap(long, default_value = DEFAULT_PORT)]
    pub port: u16,
}

impl Config {
    pub fn new_from_environment() -> anyhow::Result<Self> {
        Self::try_parse().map_err(std::convert::Into::into)
    }
}
