use clap::Arg;

pub struct Config {
    pub server_addr: String,
}

impl Config {
    pub fn new(args: &clap::ArgMatches<'_>) -> anyhow::Result<Self> {
        Ok(Self {
            server_addr: args
                .value_of("server-addr")
                .unwrap_or("http://[::1]:50051")
                .parse()?,
        })
    }

    pub fn new_from_environment() -> anyhow::Result<Self> {
        let args = clap::App::new("Legion Labs editor")
            .author(clap::crate_authors!())
            .version(clap::crate_version!())
            .about("Legion Labs editor.")
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
