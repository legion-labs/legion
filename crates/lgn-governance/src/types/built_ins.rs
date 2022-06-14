use crate::{declare_built_in_permissions, declare_built_in_roles};

declare_built_in_permissions!(
    ROOT: "The root permission, which encompasses all permissions.",
    USER_ADMIN(ROOT): "The permission to manage users.",
    USER_WRITE(USER_ADMIN): "The permission to edit users.",
    USER_READ(USER_WRITE): "The permission to read users.",
    SPACE_ADMIN(ROOT): "The permission to administer a space.",
    SPACE_WRITE(SPACE_ADMIN): "The permission to write to a space.",
    SPACE_READ(SPACE_WRITE): "The permission to read from a space.",
);

declare_built_in_roles!(
    SUPERADMIN: "The superadmin role, which has full access to the system." => ROOT,
);
