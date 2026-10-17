# Authentication source parity

From either checkout, compare the shared authentication files with an explicit peer:

```sh
node scripts/check-auth-parity.mjs --peer ../other-checkout
node --test scripts/check-auth-parity.test.mjs
```

Replace `../other-checkout` with the Gazel or Flowl peer path. The checker reports both branches, HEAD revisions, and working-tree status, then compares bytes. Missing peers/files, different contents, and self-comparison fail; dirty working trees are allowed and reported.

`auth-parity.json` is the explicit file list, including this tooling and the common test vectors. Update both copies together when shared files change; add new common files to the manifest. App policy in `src/auth/app.rs`, server integrations, branding, navigation, and offline/cache effects are intentionally excluded.

This is a review check, not a build dependency. Ordinary Cargo, frontend, and Docker builds never need the peer checkout.
