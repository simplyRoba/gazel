<p align="center">
  <!-- TODO: add logo -->
  <img src="docs/assets/icon-512.png" alt="gazel logo" width="128" />
</p>

<h1 align="center">gazel</h1>

<p align="center">
  <strong>gazel</strong> — short for <strong>ga</strong>s ga<strong>zel</strong>le — /ɡəˈzɛl/
</p>

<p align="center">
  <a href="https://conventionalcommits.org"><img src="https://img.shields.io/badge/Conventional%20Commits-1.0.0-yellow.svg" alt="Conventional Commits" /></a>
  <img src="https://img.shields.io/github/license/simplyRoba/gazel?link=https%3A%2F%2Fgithub.com%2FsimplyRoba%2Fgazel%2Fblob%2Fmain%2FLICENSE" alt="GitHub License" />
  <img src="https://img.shields.io/github/actions/workflow/status/simplyRoba/gazel/ci.yml?link=https%3A%2F%2Fgithub.com%2FsimplyRoba%2Fgazel%2Factions%2Fworkflows%2Fci.yml%3Fquery%3Dbranch%253Amain" alt="GitHub Workflow Status" />
  <a href="https://github.com/simplyRoba/gazel/releases"><img src="https://img.shields.io/github/v/release/simplyRoba/gazel?link=https%3A%2F%2Fgithub.com%2FsimplyRoba%2Fgazel%2Freleases" alt="GitHub release" /></a>
  <a href="https://github.com/simplyRoba/gazel/issues"><img src="https://img.shields.io/github/issues/simplyRoba/gazel?link=https%3A%2F%2Fgithub.com%2FsimplyRoba%2Fgazel%2Fissues" alt="GitHub issues" /></a>
  <img src="https://img.shields.io/github/stars/simplyRoba/gazel" alt="GitHub Repo stars" />
</p>

A lightweight, self-hosted fuel expense and mileage tracker. Log fill-ups, track fuel efficiency and costs across your vehicles, and spot trends over time — all from a single binary with no external dependencies.

The gazel remembers every drop so you don't have to.

## Features

<!-- TODO: add screenshots once UI is implemented -->

- **Multi-vehicle tracking** — manage all your cars, motorcycles, and trucks in one place
- **Fill-up logging** — record date, odometer, fuel amount, cost, and station
- **Fuel efficiency** — automatic MPG / L/100km calculation between fill-ups
- **Cost tracking** — cost per mile/km, monthly and yearly spend breakdowns
- **Dashboard** — at-a-glance overview with summary stats and recent activity
- **Charts** — visualize efficiency, cost, and fuel price trends over time
- **Flexible units** — switch between imperial and metric, choose your currency
- **Data portability** — export and import your data as JSON
- **Light & dark theme** — follows your system preference, with manual override
- **Single binary** — self-contained Rust service with embedded UI, just run it or use Docker

## Quick start

### Docker run

```bash
docker run -p 4110:4110 -v gazel-data:/data \
  ghcr.io/simplyroba/gazel:latest
```

Open `http://localhost:4110`. Data is persisted in the `gazel-data` volume.

### Docker Compose

A `docker-compose.yml` is included in the repository.

```bash
docker compose up -d
```

## Configuration

| Variable | Default | Description |
| --- | --- | --- |
| `GAZEL_PORT` | `4110` | HTTP server listen port. |
| `GAZEL_DB_PATH` | `/data/gazel.db` | Filesystem path to the SQLite database. |
| `GAZEL_LOG_LEVEL` | `info` | `tracing` level filter for logs. |

## Security

gazel has no built-in authentication. It is designed to run on a trusted home network or behind a reverse proxy that handles auth (e.g., Authelia, Authentik, Caddy with basic auth). Do not expose it directly to the internet.

---

**This project is developed spec-driven with AI assistance, reviewed by a critical human.**
