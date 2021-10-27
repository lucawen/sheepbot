-- Add migration script here
alter table tracker alter column created_at type timestamptz;
alter table tracker alter column updated_at type timestamptz;
