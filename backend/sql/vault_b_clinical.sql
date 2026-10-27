-- Vault B (Encrypted Clinical EHR & Immutable Audit Logs) schema.
-- Synthetic development data only. Never store patient names, addresses, or raw NFC UIDs here.

CREATE TABLE IF NOT EXISTS immutable_audit_logs (
    audit_id BIGSERIAL PRIMARY KEY,
    blind_patient_id TEXT NOT NULL CHECK (blind_patient_id LIKE 'bp_v%'),
    doctor_actor_id TEXT NOT NULL CHECK (length(btrim(doctor_actor_id)) > 0),
    hospital_id TEXT NOT NULL CHECK (length(btrim(hospital_id)) > 0),
    session_reference TEXT NOT NULL CHECK (length(btrim(session_reference)) > 0),
    action_type TEXT NOT NULL CHECK (length(btrim(action_type)) > 0),
    recorded_at_unix BIGINT NOT NULL
);

CREATE OR REPLACE FUNCTION prevent_immutable_audit_mutation()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    RAISE EXCEPTION 'immutable_audit_logs is append-only; % is forbidden', TG_OP;
END;
$$;

DROP TRIGGER IF EXISTS trg_immutable_audit_no_update_delete ON immutable_audit_logs;
CREATE TRIGGER trg_immutable_audit_no_update_delete
BEFORE UPDATE OR DELETE OR TRUNCATE ON immutable_audit_logs
FOR EACH STATEMENT
EXECUTE FUNCTION prevent_immutable_audit_mutation();

CREATE TABLE IF NOT EXISTS clinical_encrypted_records (
    record_id TEXT PRIMARY KEY CHECK (length(btrim(record_id)) > 0),
    blind_patient_id TEXT NOT NULL CHECK (blind_patient_id LIKE 'bp_v%'),
    hospital_id TEXT NOT NULL CHECK (length(btrim(hospital_id)) > 0),
    resource_purpose TEXT NOT NULL CHECK (
        resource_purpose IN (
            'encounter_note',
            'diagnosis_condition',
            'allergy_record',
            'medication_order',
            'observation_value',
            'audit_context'
        )
    ),
    schema_version INT NOT NULL CHECK (schema_version > 0),
    envelope_version INT NOT NULL CHECK (envelope_version = 1),
    key_id TEXT NOT NULL CHECK (length(btrim(key_id)) > 0),
    nonce BYTEA NOT NULL CHECK (octet_length(nonce) = 12),
    ciphertext_with_tag BYTEA NOT NULL CHECK (octet_length(ciphertext_with_tag) > 16),
    resource_version BIGINT NOT NULL DEFAULT 1 CHECK (resource_version >= 1),
    updated_at_unix BIGINT NOT NULL
);

REVOKE ALL ON TABLE immutable_audit_logs, clinical_encrypted_records FROM PUBLIC;
REVOKE ALL ON SEQUENCE immutable_audit_logs_audit_id_seq FROM PUBLIC;

-- Runtime account can only SELECT and INSERT audit logs; UPDATE, DELETE, and TRUNCATE remain revoked.
GRANT SELECT, INSERT ON TABLE immutable_audit_logs TO clinical_app;
GRANT USAGE, SELECT ON SEQUENCE immutable_audit_logs_audit_id_seq TO clinical_app;

-- Runtime account can SELECT, INSERT, and UPDATE versioned encrypted clinical records; DELETE and TRUNCATE remain revoked.
GRANT SELECT, INSERT, UPDATE ON TABLE clinical_encrypted_records TO clinical_app;
