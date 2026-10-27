-- Vault A (Identity & Durable NFC Replay Authority) schema.
-- Synthetic development data only. Never store Vault B clinical records here.

CREATE TABLE IF NOT EXISTS nfc_bands (
    tag_uid_hex TEXT PRIMARY KEY CHECK (tag_uid_hex ~ '^[0-9a-f]{14}$'),
    blind_patient_id TEXT NOT NULL CHECK (blind_patient_id LIKE 'bp_v%'),
    band_status TEXT NOT NULL CHECK (
        band_status IN (
            'inventory',
            'provisioning',
            'verification_pending',
            'active',
            'frozen',
            'replacement_pending',
            'retired',
            'failed'
        )
    ),
    high_water_counter BIGINT NOT NULL DEFAULT 0 CHECK (high_water_counter >= 0 AND high_water_counter <= 16777215),
    resource_version BIGINT NOT NULL DEFAULT 1 CHECK (resource_version >= 1),
    updated_at_unix BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS accepted_tap_contexts (
    context_id TEXT PRIMARY KEY CHECK (length(btrim(context_id)) > 0),
    blind_patient_id TEXT NOT NULL CHECK (blind_patient_id LIKE 'bp_v%'),
    tag_uid_hex TEXT NOT NULL REFERENCES nfc_bands(tag_uid_hex),
    accepted_counter BIGINT NOT NULL CHECK (accepted_counter > 0 AND accepted_counter <= 16777215),
    hospital_id TEXT NOT NULL CHECK (length(btrim(hospital_id)) > 0),
    doctor_actor_id TEXT NOT NULL CHECK (length(btrim(doctor_actor_id)) > 0),
    state TEXT NOT NULL CHECK (
        state IN (
            'proof_accepted',
            'patient_pending',
            'approved',
            'declined',
            'expired',
            'consumed',
            'revoked'
        )
    ),
    failed_pin_attempts SMALLINT NOT NULL DEFAULT 0 CHECK (failed_pin_attempts >= 0 AND failed_pin_attempts <= 5),
    issued_at_unix BIGINT NOT NULL,
    expires_at_unix BIGINT NOT NULL CHECK (expires_at_unix > issued_at_unix),
    UNIQUE (tag_uid_hex, accepted_counter)
);

CREATE TABLE IF NOT EXISTS emergency_triage_projections (
    blind_patient_id TEXT PRIMARY KEY CHECK (blind_patient_id LIKE 'bp_v%'),
    blood_type_json JSONB NOT NULL,
    critical_allergies_json JSONB NOT NULL,
    ice_contact_json JSONB NOT NULL,
    offline_summary_updated_at_unix BIGINT NOT NULL,
    resource_version BIGINT NOT NULL DEFAULT 1 CHECK (resource_version >= 1)
);

CREATE TABLE IF NOT EXISTS scoped_idempotency_keys (
    actor_id TEXT NOT NULL CHECK (length(btrim(actor_id)) > 0),
    hospital_id TEXT NOT NULL CHECK (length(btrim(hospital_id)) > 0),
    operation TEXT NOT NULL CHECK (length(btrim(operation)) > 0),
    idempotency_key TEXT NOT NULL CHECK (length(btrim(idempotency_key)) > 0),
    payload_sha256 BYTEA NOT NULL CHECK (octet_length(payload_sha256) = 32),
    created_at_unix BIGINT NOT NULL,
    PRIMARY KEY (actor_id, hospital_id, operation, idempotency_key)
);

CREATE TABLE IF NOT EXISTS vault_a_outbox_events (
    event_id TEXT PRIMARY KEY CHECK (length(btrim(event_id)) > 0),
    event_type TEXT NOT NULL CHECK (length(btrim(event_type)) > 0),
    payload_json JSONB NOT NULL,
    delivery_state TEXT NOT NULL CHECK (
        delivery_state IN ('pending', 'leased', 'delivered', 'retryable_failure', 'dead_lettered')
    ),
    retry_count INT NOT NULL DEFAULT 0 CHECK (retry_count >= 0),
    created_at_unix BIGINT NOT NULL
);

REVOKE ALL ON TABLE
    nfc_bands,
    accepted_tap_contexts,
    emergency_triage_projections,
    scoped_idempotency_keys,
    vault_a_outbox_events
FROM PUBLIC;

GRANT SELECT, INSERT, UPDATE ON TABLE
    nfc_bands,
    accepted_tap_contexts,
    emergency_triage_projections,
    vault_a_outbox_events
TO identity_app;

GRANT SELECT, INSERT ON TABLE scoped_idempotency_keys TO identity_app;
