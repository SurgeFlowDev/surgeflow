
CREATE TABLE
    workflows (
        "id" INTEGER PRIMARY KEY AUTOINCREMENT,
        "name" TEXT NOT NULL UNIQUE
    );

CREATE TABLE
    workflow_instances (
        "id" INTEGER PRIMARY KEY AUTOINCREMENT,
        "external_id" TEXT NOT NULL UNIQUE,
        "workflow_id" INTEGER NOT NULL REFERENCES workflows ("id"),
        "created_at" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
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
        "id" INTEGER PRIMARY KEY AUTOINCREMENT,
        "external_id" TEXT NOT NULL UNIQUE,
        "step" TEXT NOT NULL,
        "workflow_instance_id" INTEGER NOT NULL REFERENCES workflow_instances ("id")
    );

CREATE TABLE
    workflow_step_versions (
        "id" INTEGER PRIMARY KEY AUTOINCREMENT,
        "workflow_step_id" INTEGER NOT NULL REFERENCES workflow_steps_base ("id"),
        "status" INTEGER NOT NULL REFERENCES lu_workflow_step_status ("id"),
        "created_at" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        "version" INTEGER NOT NULL DEFAULT 1
    );

CREATE TABLE
    workflow_step_outputs (
        "id" INTEGER PRIMARY KEY AUTOINCREMENT,
        "workflow_step_id" INTEGER NOT NULL UNIQUE REFERENCES workflow_steps_base ("id"),
        "output" TEXT
    );


CREATE VIEW workflow_steps AS
    SELECT 
        wsb.id AS "id",
        wsb.external_id AS "external_id",
        wsb.step AS "step",
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

CREATE TRIGGER insert_latest_workflow_step_trigger 
INSTEAD OF INSERT ON workflow_steps
FOR EACH ROW
BEGIN
    INSERT INTO workflow_steps_base ("workflow_instance_id", "external_id", "step")
    VALUES ((SELECT id FROM workflow_instances WHERE external_id = NEW."workflow_instance_external_id"), NEW."external_id", NEW."step");
    
    INSERT INTO workflow_step_versions ("workflow_step_id", "status", "version")
    VALUES (last_insert_rowid(), 1, 1);
END;

CREATE TRIGGER update_latest_workflow_step_trigger
INSTEAD OF UPDATE ON workflow_steps
FOR EACH ROW
BEGIN
    INSERT INTO workflow_step_versions (workflow_step_id, status, version)
    VALUES (OLD.id, NEW.status, (SELECT MAX(version) + 1 FROM workflow_step_versions WHERE workflow_step_id = OLD.id));
END;

INSERT INTO
    workflows ("name")
VALUES
    ('workflow_1'),
    ('workflow_2');
