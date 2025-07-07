CREATE TABLE
    workflows (
        "id" INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
        "name" TEXT NOT NULL UNIQUE
    );

CREATE TABLE
    workflow_instances (
        "id" INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
        "workflow_id" INTEGER NOT NULL REFERENCES workflows ("id")
    );

INSERT INTO
    workflows ("name")
VALUES
    ('workflow_0'),
    ('workflow_1');