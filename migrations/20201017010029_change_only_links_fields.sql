-- Add migration script here

ALTER TABLE public.only_link_channel ALTER COLUMN guild_id TYPE bigint USING guild_id::bigint;

ALTER TABLE public.only_link_channel ALTER COLUMN channel_id TYPE bigint USING channel_id::bigint;

ALTER TABLE public.only_link_channel ALTER COLUMN user_id TYPE bigint USING user_id::bigint;
