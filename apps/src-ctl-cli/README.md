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

    attach-branch            Merges the lock domains of the two branches

    commit                   Records local changes in the repository as a single transaction

    config                   Prints the path to the configuration file and its content

    create-branch            Creates a new branch based on the state of the workspace

    delete                   Deletes the local file and records the pending change

    detach-branch            Move the current branch and its descendance to a new lock domain

    diff                     Prints difference between local file and specified commit

    edit                     Makes file writable and adds it to the set of pending changes

    help                     Prints this message or the help of the given subcommand(s)

    import-git-repo          Replicates branches and commits from a git repo

    init-local-repository    Initializes a repository stored on a local filesystem

    init-workspace           Initializes a workspace and populates it with the latest version of the main branch

    list-branches            Prints a list of all branches

    list-locks               Prints all the locks in the current lock domain

    local-changes            Lists changes in workspace lsc knows about

    lock                     Prevent others from modifying the specified file. Locks apply throught all related
                             branches

    log                      Lists commits of the current branch

    merge-branch             Merge the specified branch into the current one

    resolve                  Reconciles local modifications with colliding changes from other workspaces

    resolves-pending         Lists the files that are scheduled to be merged following a sync with colliding changes

    revert                   Abandon the local changes made to a file. Overwrites the content of the file based on
                             the current commit.

    switch-branch            Syncs workspace to specified branch

    sync                     Updates the workspace with the latest version of the files

    unlock                   Releases a lock, allowing others to modify or lock the file

    

```
