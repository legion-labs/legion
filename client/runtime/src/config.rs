use clap::Arg;
use url::Url;

pub struct Config {
    pub authorization_url: Url,
}

impl Config {
    pub fn new(authorization_url: Url) -> Self {
        Self { authorization_url }
    }

    pub fn new_from_environment() -> anyhow::Result<Self> {
        let args = clap::App::new("Legion Labs runtime")
            .author(clap::crate_authors!())
            .version(clap::crate_version!())
            .about("Legion Labs runtime.")
            .arg(
                Arg::with_name("authorization-url")
                    .long("authorization-url")
                    .takes_value(true)
                    .help("The authorization URL"),
            )
            .get_matches();

        let authorization_url = args
                .value_of("authorization-url")
                .unwrap_or("https://legionlabs-playground.auth.ca-central-1.amazoncognito.com/oauth2/authorize?client_id=5m58nrjfv6kr144prif9jk62di&response_type=code&scope=aws.cognito.signin.user.admin+email+https://legionlabs.com/editor/allocate+openid+profile&redirect_uri=http://localhost:3000/&identity_provider=Azure")
                .parse()?;

        Ok(Self::new(authorization_url))
    }
}
