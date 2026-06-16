# Security-Sensitive Copy Review (RFC-MI-080 · v0.57.0)

Reviews copy on security-sensitive screens for:
- Anti-enumeration compliance (auth screens)
- Protocol-safe wording (OIDC consent)
- Scope accuracy (OIDC consent)
- No unintentional information disclosure

## Authentication screens

### Login

- **Success path**: Redirects without wording confirming account existence. ✅
- **Failure path**: Generic message from `t.login_fail_generic` — does not
  distinguish "wrong password" from "no such account". ✅
- **Forgot-password**: Sends confirmation email; response page uses generic
  "if an account exists…" wording (`t.forgot_password_sent_body`). ✅

### MFA challenge

- **Failure path**: Generic TOTP rejection — does not reveal remaining attempts
  to prevent timing-based enumeration. ✅
- **Recovery codes**: Count is shown only to the authenticated user on the
  MFA tab (behind session). ✅

### Password reset

- **Token validation**: Expired/invalid tokens produce a generic error page
  (`render_reset_password_invalid`) without stating whether the account exists. ✅
- **Reset success**: Generic confirmation; does not mention whether sessions
  were revoked in the message body. ✅

### Step-up

- **Purpose copy**: `t.step_up_body` explains that the action requires
  re-authentication — does not reveal what specific action is pending
  (step-up is invoked before any destructive or sensitive operation). ✅

## Dangerous confirmations

Each confirmation page renders the subject (username, client ID, key ID)
to the authenticated admin — no additional disclosure.

- Delete user: shows username and user ID. ✅
- Disable user: shows username. ✅
- Reset MFA: shows username. ✅
- Delete client: shows client ID and client name. ✅
- Delete signing key: shows key ID and creation date. ✅

All confirmation pages show the `.reversibility-badge--permanent` label
("Permanent") for delete actions to give the operator a clear non-colour
indication. ✅

## OIDC consent

### Scope wording accuracy (reviewed v0.57.0)

| Scope | Label (`t.consent_scope_*`) | Description (`t.consent_scope_*_desc`) | Accurate? |
|---|---|---|---|
| `openid` | "Verify your identity" | "Confirms your sign-in and provides a unique identifier." | ✅ |
| `profile` | "Your profile (name, language)" | "Name, preferred language, and timezone." | ✅ |
| `email` | "Your email address" | "Email address and whether it has been verified." | ✅ |
| `offline_access` | "Stay signed in (refresh tokens)" | "Keeps the app signed in on your behalf when you are not present." | ✅ — clearly communicates persistent access |

### Client identity

- Client display name is shown as `<strong>{client_name}</strong>`. ✅
- No raw `client_id` is shown to the user in the consent screen
  (it is visible in the browser's URL bar via the authorization request). ✅
- No logo/icon-only identification — text name is always present. ✅

### Approve / Deny symmetry

- Approve: primary `<button type="submit">` in a `<form>`. ✅
- Deny: secondary `<button type="submit" class="secondary">` in a `<form>`. ✅
- Both are keyboard-reachable and POST with CSRF. ✅
- Deny is not a small text link. ✅ (Non-goal explicitly addressed in RFC-MI-070)

## Sign-out

- Sign-out uses POST + server-rendered CSRF (since v0.51.0). ✅
- No JavaScript required for sign-out. ✅
- Sign-out redirects to login page without indicating the previous user. ✅

## Open items

None. All security-sensitive copy was reviewed and found to be compliant.

---

*Generated: RFC-MI-080 v0.57.0. This review should be repeated
whenever authentication, consent, or dangerous-action copy is changed.*
