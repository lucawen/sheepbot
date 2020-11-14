-- 
-- Editor SQL for DB table only_link_channel
-- Created by http://editor.datatables.net/generator
-- 

CREATE TABLE IF NOT EXISTS "only_link_channel" (
	"id" serial,
	"guild_id" numeric(9,2),
	"user_id" numeric(9,2),
	"url" text,
	"channel_id" numeric(9,2),
	PRIMARY KEY( id )
);