//! The Governance server executable.

use anyhow::anyhow;
use clap::{Parser, Subcommand};
use lgn_governance::{
    formatter::Format,
    types::{ExtendedUserId, RoleAssignationPatch, RoleId, SpaceId, SpaceUpdate, UserAlias},
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
        user_id: ExtendedUserId,
    },
    #[clap(name = "resolve", about = "Resolve a user id from its email address")]
    Resolve {
        #[clap(help = "The user's ID")]
        user_id: ExtendedUserId,
    },
    #[clap(name = "list-spaces", about = "List the spaces a user has access to")]
    ListSpaces {
        #[clap(help = "The user's ID")]
        user_id: ExtendedUserId,
    },
    #[clap(name = "list-roles", about = "List the roles assigned to a user")]
    ListRoles {
        #[clap(help = "The user's ID")]
        user_id: ExtendedUserId,
    },
    #[clap(name = "assign-role", about = "Assign a role to the user")]
    AssignRole {
        #[clap(help = "The user's ID")]
        user_id: ExtendedUserId,
        #[clap(help = "The role")]
        role_id: RoleId,
        #[clap(help = "The space into which the role is assigned")]
        space_id: Option<SpaceId>,
    },
    #[clap(name = "unassign-role", about = "Unassign a role from a user")]
    UnassignRole {
        #[clap(help = "The user's ID")]
        user_id: ExtendedUserId,
        #[clap(help = "The role")]
        role_id: RoleId,
        #[clap(help = "The space into which the role is unassigned")]
        space_id: Option<SpaceId>,
    },
    #[clap(name = "list-aliases", about = "List the user aliases")]
    ListAliases,
    #[clap(name = "register-alias", about = "Register a new alias for a user")]
    RegisterAlias {
        #[clap(help = "The user alias")]
        user_alias: UserAlias,
        #[clap(help = "The user's ID")]
        user_id: ExtendedUserId,
    },
    #[clap(name = "unregister-alias", about = "Unregister an alias for a user")]
    UnregisterAlias {
        #[clap(help = "The user alias")]
        user_alias: UserAlias,
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
            UsersCommands::Resolve { user_id } => {
                let user_id = client.resolve_user_id(&user_id).await?;

                args.format.format_unit(&user_id);
            }
            UsersCommands::ListSpaces { user_id } => {
                let spaces = client.list_user_spaces(&user_id).await?;

                args.format.format_many(&spaces);
            }
            UsersCommands::ListRoles { user_id } => {
                let roles = client.list_user_roles(&user_id).await?;

                args.format.format_many(&roles);
            }
            UsersCommands::AssignRole {
                user_id,
                role_id,
                space_id,
            } => {
                let roles = client
                    .patch_user_roles(
                        &user_id,
                        RoleAssignationPatch::single_addition(role_id, space_id),
                    )
                    .await?;

                args.format.format_many(&roles);
            }
            UsersCommands::UnassignRole {
                user_id,
                role_id,
                space_id,
            } => {
                let roles = client
                    .patch_user_roles(
                        &user_id,
                        RoleAssignationPatch::single_removal(role_id, space_id),
                    )
                    .await?;

                args.format.format_many(&roles);
            }
            UsersCommands::ListAliases => {
                let user_aliases = client.list_users_aliases().await?;

                args.format.format_many(&user_aliases);
            }
            UsersCommands::RegisterAlias {
                user_alias,
                user_id,
            } => {
                let user_aliases = client.register_user_alias(&user_alias, &user_id).await?;

                args.format.format_many(&user_aliases);
            }
            UsersCommands::UnregisterAlias { user_alias } => {
                let user_aliases = client.unregister_user_alias(&user_alias).await?;

                args.format.format_many(&user_aliases);
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
