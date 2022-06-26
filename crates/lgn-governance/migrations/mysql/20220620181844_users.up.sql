CREATE TABLE `users_aliases` (
	`alias` VARCHAR(64) NOT NULL COMMENT 'The user alias.',
	`user_id` VARCHAR(256) NOT NULL COMMENT 'The user id.',
	PRIMARY KEY (`alias`)
);