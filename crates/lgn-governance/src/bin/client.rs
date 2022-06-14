//! The Governance server executable.

use anyhow::anyhow;
use clap::{Parser, Subcommand};
use lgn_governance::{
    formatter::Format,
    types::{SpaceId, SpaceUpdate, UserId},
    Config,
};
use lgn_telemetry_sink::TelemetryGuardBuilder;
use lgn_tracing::{async_span_scope, LevelFilter};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands,

    #[clap(short, long)]
    debug: bool,

    #[clap(arg_enum, short = 'f', long, global = true, default_value_t)]
    format: Format,

    #[clap(long, global = true, env)]
    space_id: Option<SpaceId>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[clap(name = "init-stack", about = "Initialize the stack", alias = "init")]
    InitStack {
        #[clap(
            help = "The initialization key, as specified on the server's command line",
            env
        )]
        init_key: String,
    },
    #[clap(subcommand, about = "Manage users", alias = "user")]
    Users(UsersCommands),
    #[clap(subcommand, about = "Manage permissions", alias = "permission")]
    Permissions(PermissionsCommands),
    #[clap(subcommand, about = "Manage roles", alias = "role")]
    Roles(RolesCommands),
    #[clap(subcommand, about = "Manage spaces", alias = "space")]
    Spaces(SpacesCommands),
    #[clap(subcommand, about = "Manage workspaces", aliases = &["workspace", "ws"])]
    Workspaces(WorkspacesCommands),
}

#[derive(Subcommand, Debug)]
enum UsersCommands {
    #[clap(name = "get", about = "Get the user's information")]
    Get {
        #[clap(help = "The user's ID")]
        user_id: UserId,
    },
    #[clap(name = "resolve", about = "Resolve a user id from its email address")]
    Resolve {
        #[clap(help = "The user's email")]
        email: String,
    },
}

#[derive(Subcommand, Debug)]
enum PermissionsCommands {
    #[clap(name = "list", about = "List all the permissions known to the system")]
    List,
}

#[derive(Subcommand, Debug)]
enum RolesCommands {
    #[clap(name = "list", about = "List all the roles known to the system")]
    List,
}

#[derive(Subcommand, Debug)]
enum SpacesCommands {
    #[clap(name = "list", about = "List all the spaces the user has access to")]
    List,
    #[clap(name = "get", about = "Get a specific space")]
    Get {
        #[clap(help = "The space id")]
        id: SpaceId,
    },
    #[clap(name = "create", about = "Create a new space")]
    Create {
        #[clap(help = "The space id")]
        id: SpaceId,

        #[clap(long, default_value_t, help = "The space description")]
        description: String,
    },
    #[clap(name = "update", about = "Update a space")]
    Update {
        #[clap(help = "The space id")]
        id: SpaceId,

        #[clap(long, help = "The space description")]
        description: Option<String>,
    },
    #[clap(name = "delete", about = "Delete a space")]
    Delete {
        #[clap(help = "The space id")]
        id: SpaceId,
    },
    #[clap(name = "cordon", about = "Cordon a space")]
    Cordon {
        #[clap(help = "The space id")]
        id: SpaceId,
    },
    #[clap(name = "uncordon", about = "Uncordon a space")]
    Uncordon {
        #[clap(help = "The space id")]
        id: SpaceId,
    },
}

#[derive(Subcommand, Debug)]
enum WorkspacesCommands {
    #[clap(name = "list", about = "List all the workspaces in the current space")]
    List,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Args = Args::parse();

    let _telemetry_guard = TelemetryGuardBuilder::default()
        .with_local_sink_max_level(if args.debug {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
        .build();

    async_span_scope!("lgc::main");

    let config = Config::load()?;
    let client = config.instantiate_client().await?;

    match args.command {
        Commands::InitStack { init_key } => client.init_stack(&init_key).await?,
        Commands::Users(command) => match command {
            UsersCommands::Get { user_id } => {
                let user_info = client.get_user_info(&user_id).await?;

                args.format.format_one(&user_info);
            }
            UsersCommands::Resolve { email } => {
                let user_id = client.resolve_user_id(&email).await?;

                args.format.format_unit(&user_id);
            }
        },
        Commands::Permissions(command) => match command {
            PermissionsCommands::List => {
                let permissions = client.list_permissions().await?;

                args.format.format_many(permissions);
            }
        },
        Commands::Roles(command) => match command {
            RolesCommands::List => {
                let roles = client.list_roles().await?;

                args.format.format_many(roles);
            }
        },
        Commands::Spaces(command) => match command {
            SpacesCommands::List => {
                let spaces = client.list_spaces().await?;

                args.format.format_many(spaces);
            }
            SpacesCommands::Create { id, description } => {
                let space = client.create_space(id, &description).await?;

                args.format.format_one(&space);
            }
            SpacesCommands::Get { id } => {
                let space = client.get_space(id).await?;

                args.format.format_one(&space);
            }
            SpacesCommands::Update { id, description } => {
                let update = SpaceUpdate { description };
                let space = client.update_space(id, update).await?;

                args.format.format_one(&space);
            }
            SpacesCommands::Delete { id } => {
                let space = client.delete_space(id).await?;

                args.format.format_one(&space);
            }
            SpacesCommands::Cordon { id } => {
                let space = client.cordon_space(id).await?;

                args.format.format_one(&space);
            }
            SpacesCommands::Uncordon { id } => {
                let space = client.uncordon_space(id).await?;

                args.format.format_one(&space);
            }
        },
        Commands::Workspaces(command) => {
            let space_id = args.space_id.ok_or_else(|| {
                anyhow!("no space id was specified. Have you forgotten to set `SPACE_ID`?")
            })?;

            match command {
                WorkspacesCommands::List => {
                    let workspaces = client.list_workspaces(&space_id).await?;

                    args.format.format_many(workspaces);
                }
            }
        }
    }

    Ok(())
}
