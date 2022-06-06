use crate::declare_built_in_permissions;

// Don't forget to extend the list of built-in permissions whenever required, to
// make them available to other services.
declare_built_in_permissions!(
    ROOT => "The root permission, which encompasses all permissions.",
    SPACE_ADMIN(ROOT) => "The permission to administer a space.",
    SPACE_WRITE(SPACE_ADMIN) => "The permission to write to a space.",
    SPACE_READ(SPACE_WRITE) => "The permission to read from a space.",
);
