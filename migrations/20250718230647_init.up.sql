
CREATE TABLE
    workflows (
        "id" INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
        "name" TEXT NOT NULL UNIQUE
    );

CREATE TABLE
    workflow_instances (
        "id" INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
        "external_id" UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
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
    (1, 'next'),
    (2, 'awaiting_event'),
    (3, 'active'),
    (4, 'completed'),
    (5, 'failed');

    

CREATE TABLE
    workflow_steps_base (
        "id" INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
        "external_id" UUID NOT NULL UNIQUE,
        "workflow_instance_id" INTEGER NOT NULL REFERENCES workflow_instances ("id")
    );

CREATE TABLE
    workflow_step_versions (
        "id" INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
        "workflow_step_id" INTEGER NOT NULL REFERENCES workflow_steps_base ("id"),
        "status" INTEGER NOT NULL REFERENCES lu_workflow_step_status ("id"),
        "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
        "version" INTEGER NOT NULL DEFAULT 1
    );

CREATE VIEW workflow_steps AS
    SELECT 
        wsb.id AS "id",
        wsb.external_id AS "external_id",
        wi.external_id AS "workflow_instance_external_id",
        wsv.status AS "status",
        wsv.version AS "version",
        wsv.created_at AS "created_at"
    FROM 
        workflow_steps_base AS wsb
    INNER JOIN workflow_instances AS wi ON wi.id = wsb.workflow_instance_id
    JOIN (
        SELECT 
            workflow_step_id,
            MAX(version) AS max_version
        FROM 
            workflow_step_versions
        GROUP BY 
            workflow_step_id
    ) AS latest ON latest.workflow_step_id = wsb.id
    JOIN 
        workflow_step_versions AS wsv ON wsv.workflow_step_id = wsb.id AND wsv.version = latest.max_version;

CREATE FUNCTION insert_latest_workflow_step()
RETURNS TRIGGER AS $$
DECLARE
    new_step_id INTEGER;
    instance_id INTEGER;
BEGIN
    -- Get workflow_instance_id from external_id
    SELECT id INTO instance_id
    FROM workflow_instances
    WHERE external_id = NEW."workflow_instance_external_id";
    
    INSERT INTO workflow_steps_base ("workflow_instance_id", "external_id")
    VALUES (instance_id, NEW."external_id")
    RETURNING id INTO new_step_id;
    
    INSERT INTO workflow_step_versions ("workflow_step_id", "status", "version")
    -- ignore NEW.version and NEW.status, these values are set by the trigger
    VALUES (new_step_id, 1, 1);
    RETURN NEW;
END;
$$
LANGUAGE plpgsql
SECURITY DEFINER;

CREATE TRIGGER insert_latest_workflow_step_trigger INSTEAD OF INSERT ON workflow_steps FOR EACH ROW EXECUTE FUNCTION insert_latest_workflow_step ();

CREATE FUNCTION update_latest_workflow_step()
RETURNS TRIGGER AS $$
DECLARE
    next_version INTEGER;
BEGIN
    -- Calculate next version number
    SELECT MAX(version) + 1 INTO next_version
    FROM workflow_step_versions
    WHERE workflow_step_id = OLD.id;
    
    -- Insert a new version with the new status
    INSERT INTO workflow_step_versions (
        workflow_step_id, 
        status, 
        version
    )
    VALUES (
        OLD.id,
        NEW.status,
        next_version
    );
    
    RETURN NEW;
END;
$$
LANGUAGE plpgsql
SECURITY DEFINER;

CREATE TRIGGER update_latest_workflow_step_trigger 
INSTEAD OF UPDATE ON workflow_steps 
FOR EACH ROW 
EXECUTE FUNCTION update_latest_workflow_step();


-------------------------------
-------------------------------
-------------------------------

REVOKE INSERT, UPDATE ON workflow_steps_base FROM app_user;
REVOKE INSERT, UPDATE ON workflow_step_versions FROM app_user;

-------------------------------
-------------------------------
-------------------------------

INSERT INTO
    workflows ("name")
VALUES
    ('workflow_0'),
    ('workflow_1');
