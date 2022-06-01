-- Add up migration script here
CREATE TABLE `spaces` (
	`id` VARCHAR(64) NOT NULL COMMENT 'The space identifier.',
	`description` VARCHAR(256) NOT NULL COMMENT 'A description for the space.',
	`cordoned` BOOLEAN NOT NULL DEFAULT false COMMENT 'Whether the space is currently cordoned.',
	`created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'The date at which the space was created.',
	PRIMARY KEY (`id`)
);