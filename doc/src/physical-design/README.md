# Physical Design

The physical design of a project refers to how its source files get layed out on disk.

## Legion's Monorepo

Legion is a monorepo in the sense that it hosts multiple directly unrelated applications and pieces of software. Monorepos encourage everyone to hold ownership of the end goal rather than the individual pieces that comprise it. This can lead to people feeling more involved and better informed about what’s going on. Monorepos reduce duplicate actions. If you need to refactor, it’s a single Find and Replace to apply the change across the codebase. There’s less switching between projects and fewer pull requests to review. On the negative side, relying on a commit to identify changes for individual components is not possible anymore, as a single commit can contain changes unrelated changes to unrelated components.

## Top Level Folders

### lib

Libraries crates implementing reusable functionnality

> Graphics Api, App definition

### plugin

Application framework/ecs plugins

### client

For client exes, apps that have direct user interactions, like clis, editors, the game:

> Editor (client part), Game (client part), GameStreamed (client part)

### server

Server exes, apps that have api based interactions

> EditorSrv, RuntimeSrv, ResourceSrv, GameServer

### compiler

Compiler exes, special clients used to transform source data/code to a different representations

> Data Compilers, Shader Compilers

