-- Migration 0024 — email_outbox.locale (RFC 002 § C)
--
-- Stores the resolved locale for each outbox row so the worker thread
-- renders the mail template in the correct language even when the
-- preferred_lang of the recipient differs from the request locale.
-- NULL means "use server default".

ALTER TABLE email_outbox ADD COLUMN locale TEXT;
