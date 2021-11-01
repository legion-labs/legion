use clap::Arg;

use log::LevelFilter;
use tonic::codegen::http::Uri;
use url::Url;

pub struct Config {
    pub authorization_url: Url,
    pub server_addr: Uri,
    pub log_level: LevelFilter,
}

impl Config {
    pub fn new(args: &clap::ArgMatches<'_>) -> anyhow::Result<Self> {
        Ok(Self {
            authorization_url: args
                .value_of("authorization-url")
                .unwrap_or("https://legionlabs-playground.auth.ca-central-1.amazoncognito.com/oauth2/authorize?client_id=5m58nrjfv6kr144prif9jk62di&response_type=code&scope=aws.cognito.signin.user.admin+email+https://legionlabs.com/editor/allocate+openid+profile&redirect_uri=http://localhost:5001/&identity_provider=Azure")
                .parse()?,
            server_addr: args
                .value_of("server-addr")
                .unwrap_or("http://[::1]:50051")
                .parse()?,
            log_level: LevelFilter::Debug,
        })
    }

    pub fn new_from_environment() -> anyhow::Result<Self> {
        let args = clap::App::new("Legion Labs editor")
            .author(clap::crate_authors!())
            .version(clap::crate_version!())
            .about("Legion Labs editor.")
            .arg(
                Arg::with_name("authorization-url")
                    .long("authorization-url")
                    .takes_value(true)
                    .help("The authorization URL"),
            )
            .arg(
                Arg::with_name("server-addr")
                    .long("server-addr")
                    .takes_value(true)
                    .help("The address of the editor server to connect to"),
            )
            .get_matches();

        Self::new(&args)
    }
}
