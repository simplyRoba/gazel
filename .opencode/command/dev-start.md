---
description: Start the hot-reload dev servers (Vite UI + Rust backend) as background tasks for development.
---

Start both dev servers as background tasks per CONTRIBUTING.md:

1. **UI dev server** (Vite with hot module reload, exposed on network for mobile testing):
   ```bash
   npm run dev --prefix ui -- --host
   ```
   Run this as a background task.

2. **Rust backend** (cargo watch with auto-restart):
   ```bash
   GAZEL_DB_PATH=/tmp/gazel.db SKIP_UI_BUILD=1 cargo watch -x run
   ```
   Run this as a background task.

After starting both, read the Vite output (non-blocking) to find the network URLs, then tell the user:
- The local URL (http://localhost:5173)
- The network URL(s) for phone testing
- That Vite proxies `/api` and `/health` to the Rust backend on port 4110
