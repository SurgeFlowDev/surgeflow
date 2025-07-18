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

CREATE TABLE
    workflows (
        "id" INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
        "name" TEXT NOT NULL UNIQUE
    );

CREATE TABLE
    workflow_instances (
        "id" INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
        "workflow_id" INTEGER NOT NULL REFERENCES workflows ("id"),
        "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

CREATE TABLE
    lu_workflow_step_status (
        "id" INTEGER PRIMARY KEY,
        "label" TEXT NOT NULL UNIQUE
    );

INSERT INTO
    lu_workflow_step_status ("id", "label")
VALUES
    (1, 'pending'),
    (2, 'running'),
    (3, 'completed'),
    (4, 'failed');

CREATE TABLE
    workflow_steps (
        "id" INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
        "workflow_instance_id" INTEGER NOT NULL REFERENCES workflow_instances ("id")
    );

CREATE TABLE
    workflow_step_versions (
        "id" INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
        "workflow_step_id" INTEGER NOT NULL REFERENCES workflow_steps ("id"),
        "status" INTEGER NOT NULL REFERENCES lu_workflow_step_status ("id"),
        "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
        "version" INTEGER NOT NULL DEFAULT 1
    );

CREATE VIEW
    latest_workflow_steps AS
SELECT
    ws.id AS step_id,
    ws.workflow_instance_id,
    wi.workflow_id,
    wsv.id AS step_version_id,
    wsv.version,
    wsv.status,
    status.label AS status_label,
    wsv.created_at AS version_created_at,
    wi.created_at AS instance_created_at
FROM
    workflow_steps ws
    INNER JOIN workflow_instances wi ON ws.workflow_instance_id = wi.id
    INNER JOIN (
        SELECT
            workflow_step_id,
            MAX(version) AS latest_version
        FROM
            workflow_step_versions
        GROUP BY
            workflow_step_id
    ) latest ON ws.id = latest.workflow_step_id
    INNER JOIN workflow_step_versions wsv ON ws.id = wsv.workflow_step_id
    AND wsv.version = latest.latest_version
    INNER JOIN lu_workflow_step_status status ON wsv.status = status.id;

CREATE OR REPLACE FUNCTION insert_latest_workflow_step()
RETURNS TRIGGER AS $$
DECLARE
    new_step_id INTEGER;
BEGIN    
    INSERT INTO workflow_steps (workflow_instance_id)
    VALUES (NEW.workflow_instance_id)
    RETURNING id INTO new_step_id;
    
    INSERT INTO workflow_step_versions (workflow_step_id, status, version)
    -- ignore NEW.version and NEW.status, these values are set by the trigger
    VALUES (new_step_id, 1, 1);
    RETURN NEW;
END;
$$
LANGUAGE plpgsql
SECURITY DEFINER;

CREATE TRIGGER insert_latest_workflow_step_trigger INSTEAD OF INSERT ON latest_workflow_steps FOR EACH ROW EXECUTE FUNCTION insert_latest_workflow_step ();

CREATE OR REPLACE FUNCTION update_latest_workflow_step()
RETURNS TRIGGER AS $$
DECLARE
    next_version INTEGER;
BEGIN
    IF NEW.version IS NOT NULL THEN
        RAISE EXCEPTION 'Version must be NULL when updating a workflow step, it is automatically set.';
    END IF;

    -- Calculate next version number
    SELECT MAX(version) + 1 INTO next_version
    FROM workflow_step_versions
    WHERE workflow_step_id = OLD.step_id;
    
    -- Insert a new version with the new status
    INSERT INTO workflow_step_versions (
        workflow_step_id, 
        status, 
        version
    )
    VALUES (
        OLD.step_id,
        NEW.status,
        next_version
    );
    
    RETURN NEW;
END;
$$
LANGUAGE plpgsql
SECURITY DEFINER;

CREATE TRIGGER update_latest_workflow_step_trigger 
INSTEAD OF UPDATE ON latest_workflow_steps 
FOR EACH ROW 
EXECUTE FUNCTION update_latest_workflow_step();


-------------------------------
-------------------------------
-------------------------------

REVOKE INSERT, UPDATE ON workflow_steps FROM app_user;
REVOKE INSERT, UPDATE ON workflow_step_versions FROM app_user;

-------------------------------
-------------------------------
-------------------------------

INSERT INTO
    workflows ("name")
VALUES
    ('workflow_0'),
    ('workflow_1');
