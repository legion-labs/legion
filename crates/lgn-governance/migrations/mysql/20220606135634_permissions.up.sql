CREATE TABLE `permissions` (
	`id` VARCHAR(64) NOT NULL COMMENT 'The permission identifier.',
	`description` VARCHAR(256) NOT NULL COMMENT 'A description for the permission.',
	`parent_id` VARCHAR(64) COMMENT 'The parent permission.',
	`created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'The date at which the permission was created.',
	PRIMARY KEY (`id`),
    FOREIGN KEY (`parent_id`) REFERENCES `permissions`(`id`)
);

-- Force the identifiers to be lowercase.
CREATE TRIGGER permissions_insert_lcase BEFORE INSERT ON `permissions` FOR EACH ROW
SET NEW.id = LOWER(NEW.id);

CREATE TRIGGER permissions_update_lcase BEFORE UPDATE ON `permissions` FOR EACH ROW
SET NEW.id = LOWER(NEW.id);

CREATE TABLE `roles` (
	`id` VARCHAR(64) NOT NULL COMMENT 'The role identifier.',
	`description` VARCHAR(256) NOT NULL COMMENT 'A description for the role.',
	`created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'The date at which the role was created.',
	PRIMARY KEY (`id`)
);

-- Force the identifiers to be lowercase.
CREATE TRIGGER roles_insert_lcase BEFORE INSERT ON `roles` FOR EACH ROW
SET NEW.id = LOWER(NEW.id);

CREATE TRIGGER roles_update_lcase BEFORE UPDATE ON `roles` FOR EACH ROW
SET NEW.id = LOWER(NEW.id);

CREATE TABLE `roles_to_permissions` (
	`role_id` VARCHAR(64) NOT NULL COMMENT 'The role identifier.',
	`permission_id` VARCHAR(64) NOT NULL COMMENT 'The permission identifier.',
    `built_in` BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether the permission is built-in.',
	PRIMARY KEY (`role_id`, `permission_id`),
    FOREIGN KEY (`role_id`) REFERENCES `roles`(`id`),
    FOREIGN KEY (`permission_id`) REFERENCES `permissions`(`id`)
);

CREATE TABLE `users_to_roles` (
	`user_id` VARCHAR(64) NOT NULL COMMENT 'The user identifier.',
    `space_id` VARCHAR(64) DEFAULT NULL COMMENT 'The space identifier where this role is bound.',
	`role_id` VARCHAR(64) NOT NULL COMMENT 'The role identifier.',
	CONSTRAINT UC_UsersToRoles UNIQUE (`user_id`, `space_id`, `role_id`),
    FOREIGN KEY (`space_id`) REFERENCES `spaces`(`id`),
    FOREIGN KEY (`role_id`) REFERENCES `roles`(`id`)
);