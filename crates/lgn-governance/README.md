Session service
===============

The Legion Labs Governance service provides endpoints to manage spaces, user
profiles, permissions, sessions, and workspaces.

It lives at the heart of the Legion Engine pipeline and is used by users as well
as other services and server instances.

## Definitions

In order to properly explain the behavior and responsibilities of the session
service, let's define a few terms.

### Space

A space is a virtual data space that provides a strong separation between
different customers or different logical entities within a customer. 

As spaces are completely isolated from one another, they cannot share anything
and have possibly different user-sets and permissions.

Space can be cordoned, meaning that no write operation can succeed on it,
including the creation of new workspaces, sessions or commits. Cordoned spaces
are usually about to be destroyed, which requires the space to be absolutely
empty of any existing session before anything can happen.

### Workspace

A workspace is a reference to a source-control repository and one of its
branches. It also references a world that is being worked on. A workspace lives
in a space.

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

Profiles are space-agnostic and can exist in any number of spaces.