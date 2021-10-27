-- Add migration script here
CREATE TABLE tracker (
    id uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    user_id bigint,
    created_at timestamp,
	updated_at timestamp,
	code text,
	status text
);
