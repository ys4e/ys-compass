-- Drop existing tables.
DROP TABLE IF EXISTS `versions`;
DROP TABLE IF EXISTS `tools`;
DROP TABLE IF EXISTS `mods`;
DROP TABLE IF EXISTS `profiles`;

CREATE TABLE `versions` (
    `version` VARCHAR(24) NOT NULL PRIMARY KEY UNIQUE,
    `path` TEXT NOT NULL
);

--- This table includes:
--- injectable tools (3DMigoto, ReShade, etc.)
--- executables that run alongside the game
CREATE TABLE `tools` (
    `id` TEXT NOT NULL PRIMARY KEY UNIQUE,
    `name` TEXT NOT NULL,
    `icon` TEXT NOT NULL,
    `path` TEXT NOT NULL
);

--- This table includes:
--- visual mods
--- server plugins
CREATE TABLE `mods` (
    `id` TEXT NOT NULL PRIMARY KEY UNIQUE,
    `name` TEXT NOT NULL,
    `icon` TEXT NOT NULL,
    `path` TEXT NOT NULL,
    `version` TEXT NOT NULL,
    `tool` TEXT NOT NULL REFERENCES tools(id) ON UPDATE CASCADE
);

CREATE TABLE `profiles` (
    `id` TEXT NOT NULL PRIMARY KEY UNIQUE,
    `name` TEXT NOT NULL,
    `icon` TEXT NOT NULL,
    `version` VARCHAR(24) REFERENCES versions(version) ON UPDATE CASCADE,
    `tools` TEXT, -- This is a comma-separated list of tool IDs.
    `mods` TEXT -- This is a comma-separated list of mod IDs.
);
