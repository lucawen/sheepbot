-- Add migration script here
CREATE TABLE public.guild_info
(
    guild_id bigint NOT NULL,
    prefix text COLLATE pg_catalog."default",
    CONSTRAINT guild_info_pkey PRIMARY KEY (guild_id)
)

TABLESPACE pg_default;

ALTER TABLE public.guild_info
    OWNER to postgres;
