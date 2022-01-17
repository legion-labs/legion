use clap::Parser;
use url::Url;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct Config {
    /// The authorization URL
    #[clap(
        long,
        default_value = "https://legionlabs-playground.auth.ca-central-1.amazoncognito.com/oauth2/authorize?client_id=5m58nrjfv6kr144prif9jk62di&response_type=code&scope=aws.cognito.signin.user.admin+email+https://legionlabs.com/editor/allocate+openid+profile&redirect_uri=http://localhost:3000/&identity_provider=Azure"
    )]
    pub authorization_url: Url,
}

impl Config {
    pub fn new_from_environment() -> anyhow::Result<Self> {
        Self::try_parse().map_err(std::convert::Into::into)
    }
}
