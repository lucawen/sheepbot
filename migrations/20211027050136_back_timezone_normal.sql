-- Add migration script here
alter table tracker alter column created_at type timestamp;
alter table tracker alter column updated_at type timestamp;
