
DROP TRIGGER update_latest_workflow_step_trigger ON latest_workflow_steps;
DROP TRIGGER insert_latest_workflow_step_trigger ON latest_workflow_steps;

-- Drop functions
DROP FUNCTION update_latest_workflow_step();
DROP FUNCTION insert_latest_workflow_step();

-- Drop view
DROP VIEW latest_workflow_steps;

-- Drop tables (in reverse order to handle dependencies)
DROP TABLE workflow_step_versions;
DROP TABLE workflow_steps;
DROP TABLE lu_workflow_step_status;
DROP TABLE workflow_instances;

-- Delete sample data
DELETE FROM workflows WHERE name IN ('workflow_0', 'workflow_1');

DROP TABLE workflows;

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