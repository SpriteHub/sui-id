// sui-id step-up WebAuthn helper.
//
// Attaches to the form #step-up-passkey-form on /me/security/step-up.
// Flow:
//
//   POST /me/security/step-up/webauthn/start
//     → { challenge_json: "..." }   (the server stamps a pending-id cookie)
//   navigator.credentials.get(...)
//   POST /me/security/step-up/webauthn/finish
//     → 303 to return_to             (or 400 on failure)
//
// The challenge JSON shape and base64url encoding rules are identical
// to webauthn.js's authentication flow; we duplicate the helpers
// rather than refactor because the two scripts' concerns are
// independent and the duplication is shallow.

(function () {
  "use strict";

  function b64urlToBytes(s) {
    s = s.replace(/-/g, "+").replace(/_/g, "/");
    while (s.length % 4) s += "=";
    var raw = atob(s);
    var arr = new Uint8Array(raw.length);
    for (var i = 0; i < raw.length; i++) arr[i] = raw.charCodeAt(i);
    return arr.buffer;
  }

  function bytesToB64url(buf) {
    var bytes = new Uint8Array(buf);
    var s = "";
    for (var i = 0; i < bytes.length; i++) s += String.fromCharCode(bytes[i]);
    return btoa(s).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
  }

  function decodeRequestOptions(opts) {
    opts.publicKey.challenge = b64urlToBytes(opts.publicKey.challenge);
    if (Array.isArray(opts.publicKey.allowCredentials)) {
      opts.publicKey.allowCredentials = opts.publicKey.allowCredentials.map(function (c) {
        return Object.assign({}, c, { id: b64urlToBytes(c.id) });
      });
    }
    return opts.publicKey;
  }

  function encodeAuthenticationCredential(cred) {
    return {
      id: cred.id,
      rawId: bytesToB64url(cred.rawId),
      type: cred.type,
      response: {
        authenticatorData: bytesToB64url(cred.response.authenticatorData),
        clientDataJSON: bytesToB64url(cred.response.clientDataJSON),
        signature: bytesToB64url(cred.response.signature),
        userHandle: cred.response.userHandle
          ? bytesToB64url(cred.response.userHandle)
          : null,
      },
      extensions: cred.getClientExtensionResults
        ? cred.getClientExtensionResults()
        : {},
    };
  }

  function postForm(url, body) {
    return fetch(url, {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      credentials: "same-origin",
      body: body,
      redirect: "manual",
    });
  }

  var form = document.getElementById("step-up-passkey-form");
  if (!form) return;

  form.addEventListener("submit", function (e) {
    e.preventDefault();
    var csrf = (form.querySelector('input[name="_csrf"]') || {}).value || "";
    var returnTo = (form.querySelector('input[name="return_to"]') || {}).value || "/me/security";
    var startBody =
      "_csrf=" + encodeURIComponent(csrf) +
      "&return_to=" + encodeURIComponent(returnTo);
    postForm("/me/security/step-up/webauthn/start", startBody)
      .then(function (r) {
        if (!r.ok) throw new Error("server rejected start");
        return r.json();
      })
      .then(function (body) {
        var opts = JSON.parse(body.challenge_json);
        return navigator.credentials.get({
          publicKey: decodeRequestOptions(opts),
        });
      })
      .then(function (cred) {
        var enc = encodeAuthenticationCredential(cred);
        var finishBody =
          "_csrf=" + encodeURIComponent(csrf) +
          "&credential=" + encodeURIComponent(JSON.stringify(enc)) +
          "&return_to=" + encodeURIComponent(returnTo);
        return postForm("/me/security/step-up/webauthn/finish", finishBody);
      })
      .then(function (r) {
        // fetch redirect:'manual' surfaces a 303 as opaqueredirect;
        // we just navigate manually to return_to on success.
        if (r.type === "opaqueredirect" || (r.status >= 300 && r.status < 400) || r.ok) {
          window.location.href = returnTo;
        } else {
          alert("Step-up authentication failed. Please try again.");
        }
      })
      .catch(function (err) {
        console.error("step-up passkey failed", err);
        alert("Step-up authentication failed: " + (err && err.message ? err.message : err));
      });
  });
})();
