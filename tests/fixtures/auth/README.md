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

## Characterization during consolidation

Return-target tests in both apps assert the final `target` expectations.
Callback `baseline` fields still describe existing Gazel behavior; Flowl's
callback differences are explicit in its test harnesses.
Tests retry the original callback to distinguish rejection from consumption and
count provider requests to detect unintended token or error-URI requests.

As each implementation is aligned, replace its baseline assertions with `target`
expectations. Do not change target expectations just to make a baseline test pass.
