-- Add migration script here
alter table tracker alter column created_at type timestamp with time zone;
alter table tracker alter column updated_at type timestamp with time zone;
