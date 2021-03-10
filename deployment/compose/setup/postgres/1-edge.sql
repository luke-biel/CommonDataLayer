CREATE TABLE IF NOT EXISTS cdl.edge (
    left_object_id UUID NOT NULL,
    left_schema_id UUID NOT NULL,
    right_object_id UUID NOT NULL,
    right_schema_id UUID NOT NULL
);
