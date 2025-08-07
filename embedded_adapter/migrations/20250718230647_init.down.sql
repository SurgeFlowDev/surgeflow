-- Add down migration script here

DROP TRIGGER update_latest_workflow_step_trigger ON workflow_steps;
DROP TRIGGER insert_latest_workflow_step_trigger ON workflow_steps;

-- Drop functions
DROP FUNCTION update_latest_workflow_step();
DROP FUNCTION insert_latest_workflow_step();

-- Drop view
DROP VIEW workflow_steps;

-- Drop tables (in reverse order to handle dependencies)
DROP TABLE workflow_step_outputs;
DROP TABLE workflow_step_versions;
DROP TABLE workflow_steps_base;
DROP TABLE lu_workflow_step_status;
DROP TABLE workflow_instances;

-- Delete sample data
DELETE FROM workflows WHERE name IN ('workflow_0', 'workflow_1');

DROP TABLE workflows;

