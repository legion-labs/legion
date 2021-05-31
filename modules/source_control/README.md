# Legion Source Control
## Objectives

 - Centralized (to allow locking)
 - Scalable in size (based on cloud native tech)
 - Scalable in collaborators (smart branches with conflict-free merge guarantees)

## Command-line interface
```
Legion Source Control 0.1.0

USAGE:

lsc-cli.exe [SUBCOMMAND]


FLAGS:

-h, --help       Prints help information

SUBCOMMANDS:

    add                      Adds local file to the set of pending changes

    commit                   Records local changes in the repository as a single transaction

    delete                   Deletes the local file and records the pending change

    diff                     Prints difference between local file and specified commit

    edit                     Makes file writable and adds it to the set of pending changes

    help                     Prints this message or the help of the given subcommand(s)

    init-local-repository    Initializes a repository stored on a local filesystem

    init-workspace           Initializes a workspace and populates it with the latest version of the main branch

    local-changes            Lists changes in workspace lsc knows about

    log                      Lists commits of the current branch

    merge                    Reconciles local modifications with colliding changes from other workspaces

    merges-pending           Lists the files that are scheduled to be merged following a sync with colliding changes

    revert                   Abandon the local changes made to a file. Overwrites the content of the file based on
                             the current commit.

    sync                     Updates the workspace with the latest version of the files

```

