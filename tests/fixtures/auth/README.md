# Authentication test vectors

These fixtures are copied unchanged between Gazel and Flowl. Each app runs them
against its own parser and local OIDC provider; tests do not require the other
repository.

## Return targets

`return-targets.json` contains a serialized `input` and per-app `target`
acceptance flags. Accepted values are preserved unchanged; rejected or absent
values fall back to `/`. Long inputs use `prefix + repeat × count + suffix`.
The target limit is 2,048 UTF-8 bytes before validation-only percent decoding,
not the size of the encoded query parameter. Flowl additionally excludes uploads.

## Callbacks

`callbacks.json` contains raw queries. The provider harness replaces `{state}`
and `{code}` with a bound login attempt and `{error_uri}` with an encoded local
sentinel URL. Target outcomes are `success` (one exchange), `error` (consume
without exchange), and `invalid` (neither consume nor exchange).

Both apps assert the `target` expectations. Callback tests also retry the
original valid response to check whether the attempt was consumed, and count
provider requests to detect unintended token exchanges or error-URI fetches.
