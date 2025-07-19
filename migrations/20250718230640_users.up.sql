-- Add up migration script here
CREATE ROLE app_user
  WITH LOGIN
       PASSWORD ''
       NOSUPERUSER
       NOCREATEDB
       NOCREATEROLE
       INHERIT;

GRANT CONNECT ON DATABASE neondb TO app_user;
GRANT USAGE ON SCHEMA public TO app_user;

ALTER DEFAULT PRIVILEGES 
  FOR ROLE neondb_owner
  IN SCHEMA public
  GRANT SELECT, INSERT, UPDATE, DELETE
  ON TABLES
  TO app_user;

ALTER DEFAULT PRIVILEGES
  FOR ROLE neondb_owner
  IN SCHEMA public
  GRANT USAGE
  ON SEQUENCES
  TO app_user;
