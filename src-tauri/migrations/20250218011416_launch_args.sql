-- Adds customizable launch arguments to profiles.
ALTER TABLE `profiles` ADD `launch_args` TEXT NOT NULL DEFAULT '';
