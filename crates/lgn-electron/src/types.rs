use std::{fmt::Display, path::PathBuf, str::FromStr};

use clap::{ArgEnum, Parser, Subcommand};
use http::Uri;
use serde::Serialize;

use crate::error::{Error, Result};

const DEFAULT_HEIGHT: u16 = 900;

const DEFAULT_WIDTH: u16 = 1200;

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum Source {
    #[serde(with = "http_serde::uri")]
    Remote(Uri),
    Local(PathBuf),
}

#[derive(Clone, Debug, Serialize, ArgEnum)]
#[serde(rename_all = "camelCase")]
pub enum Decoration {
    /// Displays the top bar and the menu
    Full,
    /// Displays only the menu
    TopbarOnly,
    /// No decoration
    None,
}

#[derive(Debug, Serialize)]
pub struct Dimension((u16, u16));

impl Dimension {
    fn width(&self) -> u16 {
        self.0 .0
    }

    fn height(&self) -> u16 {
        self.0 .1
    }
}

impl Default for Dimension {
    fn default() -> Self {
        Self((DEFAULT_WIDTH, DEFAULT_HEIGHT))
    }
}

impl FromStr for Dimension {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let parts = s.split('x').collect::<Vec<&str>>();

        if parts.len() != 2 {
            return Err("Invalid dimension".into());
        }

        let width = parts[0]
            .parse()
            .map_err(|_error| "Width is not a valid u16".to_string())?;

        let height = parts[1]
            .parse()
            .map_err(|_error| "Height is not a valid u16".to_string())?;

        Ok(Self((width, height)))
    }
}

impl Display for Dimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width(), self.height())
    }
}

#[derive(Debug, Serialize)]
pub struct ElectronRuntimeConfiguration {
    pub source: Source,
    pub decoration: Decoration,
    pub fullscreen: bool,
    pub dimension: Dimension,
    pub verbose: bool,
}

impl ElectronRuntimeConfiguration {
    pub fn serialize(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|_error| Error::InvalidElectronRuntimeConfiguration)
    }
}

impl TryFrom<Args> for ElectronRuntimeConfiguration {
    type Error = Error;

    fn try_from(args: Args) -> Result<Self> {
        let source = match args.command {
            Command::Remote { uri } => Source::Remote(uri),
            Command::Build { path } | Command::Start { path } => Source::Local(
                dunce::canonicalize(&path).map_err(|_error| Error::InvalidPath(path))?,
            ),
        };

        Ok(Self {
            decoration: args.decoration,
            fullscreen: args.fullscreen,
            dimension: args.dimension,
            verbose: args.verbose,
            source,
        })
    }
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Run in remote mode, will listen to the provided endpoint
    Remote {
        /// Set the running application Uri
        #[clap(short, long)]
        uri: Uri,
    },
    /// Start a built application, using the provided path
    Start {
        /// The application build folder path
        #[clap(short, long)]
        path: PathBuf,
    },
    /// Build an Electron application
    Build {
        /// The application src folder path
        #[clap(short, long)]
        path: PathBuf,
    },
}

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Command,

    /// Type of decoration used for the main window
    #[clap(arg_enum,  long, default_value_t = Decoration::Full)]
    decoration: Decoration,

    /// If this flag is used, the dimension will be ignored
    #[clap(long)]
    fullscreen: bool,

    /// Path to a custom Electron node module. Will be used as the "CWD" when running Electron.
    /// A "local" node module with simple defaults will be used by default
    #[clap(long)]
    pub package_path: Option<PathBuf>,

    /// Path to a custom `main.js`/`main.ts` file used to bootstrap the application.
    /// This path is relative to the `package-path `and will be ignored if said `package-path` is not set.
    #[clap(long, default_value = "src/main.js")]
    pub main_path: PathBuf,

    /// Path to a custom tsconfig file (`tsconfig.json` by default)
    /// This path is relative to the `package-path `and will be ignored if said `package-path` is not set.
    /// Also ignored if the project doesn't use typescript.
    #[clap(long, default_value = "tsconfig.json")]
    pub tsconfig_path: PathBuf,

    // TODO: Not used for now
    #[clap(short, long)]
    verbose: bool,

    /// Forces typescript mode, by default typescript is activated only if the provided
    /// `--package-path` and `--main-path` pair point to a file which extension is `.ts`
    #[clap(long)]
    pub typescript: bool,

    /// Dimension must have format WIDTHxHEIGHT (notice the 'x' in between the height and the width) (ignored if the `--fullscreen` is set)
    #[clap(long, default_value_t = Dimension::default())]
    dimension: Dimension,
}
