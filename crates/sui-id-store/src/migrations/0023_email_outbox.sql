-- Migration 0023 — email_outbox (RFC 001)
--
-- Persistent outbox for reliable email delivery with retry.
-- Outgoing mail is enqueued here by the request thread and drained by the
-- background OutboxWorker with exponential backoff.
--
-- recipient_enc and payload_enc are AAD-bound XChaCha20-Poly1305 ciphertext,
-- sealed under the master key. They are added to the key-rotation reseal list.

CREATE TABLE email_outbox (
    id              TEXT    PRIMARY KEY,
    state           TEXT    NOT NULL
                    CHECK (state IN ('queued', 'sending', 'sent', 'failed')),
    template        TEXT    NOT NULL,       -- stable id: 'forgot_password', etc.
    recipient_enc   BLOB    NOT NULL,       -- encrypted recipient address
    payload_enc     BLOB    NOT NULL,       -- encrypted serialised template params
    attempt_count   INTEGER NOT NULL DEFAULT 0,
    next_attempt_at TEXT    NOT NULL,       -- ISO-8601 UTC; eligible when <= now()
    last_error      TEXT,                  -- last SMTP error (no credentials)
    created_at      TEXT    NOT NULL,
    updated_at      TEXT    NOT NULL
);

-- Partial index: only queued rows need the scheduler to look them up.
CREATE INDEX idx_email_outbox_eligible
    ON email_outbox (next_attempt_at)
    WHERE state = 'queued';

-- On startup, rows stuck in 'sending' (process crash mid-send) are reset
-- by requeue_stuck_sending(). No index needed for that rare sweep.
