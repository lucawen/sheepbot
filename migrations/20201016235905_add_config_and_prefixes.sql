-- Add migration script here

CREATE TABLE config (
    guild_id bigint NOT NULL PRIMARY KEY,
    host_role_id bigint
);

CREATE TABLE prefixes (
    guild_id bigint NOT NULL PRIMARY KEY,
    prefix text
);