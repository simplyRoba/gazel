---
description: Stop the running hot-reload dev servers (Vite UI + Rust backend).
---

Stop the dev servers that were started by `/dev-start`.

## Steps

1. **Check OS processes**: Run `ps aux | grep -E "cargo watch|vite|npm run dev" | grep -v grep` to find any running dev processes. Kill them with `kill <pid>`.

2. **Verify**: Run the same `ps` command again to confirm everything is stopped.

3. **Report**: Tell the user what was stopped. If nothing was found, say so.
