-- Revoke privileges
REVOKE USAGE ON SCHEMA public FROM app_user;
REVOKE CONNECT ON DATABASE neondb FROM app_user;

-- Revoke default privileges that were granted in the up migration
ALTER DEFAULT PRIVILEGES 
  FOR ROLE neondb_owner
  IN SCHEMA public
  REVOKE SELECT, INSERT, UPDATE, DELETE
  ON TABLES
  FROM app_user;

ALTER DEFAULT PRIVILEGES
  FOR ROLE neondb_owner
  IN SCHEMA public
  REVOKE USAGE
  ON SEQUENCES
  FROM app_user;

-- Drop role
DROP ROLE app_user;