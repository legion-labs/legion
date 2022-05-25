Session service
===============

The Legion Labs session service provides endpoints to manage sessions,
workspaces and user profiles.

## Definitions

In order to properly explain the behavior and responsibilities of the session
service, let's define a few terms.

### Workspace

A workspace is a reference to a source-control repository and one of its
branches. It also references a world that is being worked on.

Workspaces have a unique identifier and optional friendly name and description.

### Session

A session is the association of a workspace and a user. A given session ceases
to exist when its attached user or workspace are deleted.

A session contains user-centric information related to a specific workspace,
such as the current camera, selection or editor layout.

### Profile

A profile is the global information associated with a user. It contains for
instance the user's profile picture, their editor preferences (default layout,
language, etc.).