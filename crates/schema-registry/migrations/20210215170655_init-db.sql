-- add migration script here

DROP TABLE IF EXISTS schemas;
DROP TABLE IF EXISTS definitions;
DROP TABLE IF EXISTS views;

CREATE TYPE schema_type AS ENUM ('document_storage', 'timeseries');

CREATE TABLE IF NOT EXISTS schemas (
    id         uuid primary key not null,
    name       varchar not null,
    type       schema_type not null,
    queue      varchar not null,
    query_addr varchar not null
);

CREATE TABLE IF NOT EXISTS definitions (
    version varchar not null,
    definition json not null,
    schema uuid not null,

    PRIMARY KEY(schema, version),
    CONSTRAINT fk_schema_1
        FOREIGN KEY(schema)
        REFERENCES schemas(id) 
        ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS views (
    id       uuid primary key not null,
    name     varchar not null,
    jmespath varchar not null,
    schema   uuid not null,

    CONSTRAINT fk_schema_2
        FOREIGN KEY(schema)
        REFERENCES schemas(id) 
        ON DELETE CASCADE
);
