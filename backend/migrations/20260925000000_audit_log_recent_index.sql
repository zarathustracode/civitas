-- The operator dashboard pages through the most recent audit events across
-- every entity, keyed on (created_at, id). The existing audit_log indexes
-- all lead with entity or action, so that scan had nothing to use.

create index idx_audit_log_created_at
    on audit_log (created_at desc, id desc);
