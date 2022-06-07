use crate::{declare_built_in_permissions, declare_built_in_roles};

declare_built_in_permissions!(
    ROOT: "The root permission, which encompasses all permissions.",
    SPACE_ADMIN(ROOT): "The permission to administer a space.",
    SPACE_WRITE(SPACE_ADMIN): "The permission to write to a space.",
    SPACE_READ(SPACE_WRITE): "The permission to read from a space.",
);

declare_built_in_roles!(
    SUPERADMIN: "The superadmin role, which has full access to the system." => ROOT,
);
