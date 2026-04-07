<h1 align="center">gazel</h1>

<p align="center">
  <strong>gazel</strong> — short for <strong>ga</strong>s <strong>zel</strong> (gazelle) — /ɡəˈzɛl/
</p>

<p align="center">
  <a href="https://conventionalcommits.org"><img src="https://img.shields.io/badge/Conventional%20Commits-1.0.0-yellow.svg" alt="Conventional Commits" /></a>
  <img src="https://img.shields.io/github/license/simplyRoba/gazel?link=https%3A%2F%2Fgithub.com%2FsimplyRoba%2Fgazel%2Fblob%2Fmain%2FLICENSE" alt="GitHub License" />
  <img src="https://img.shields.io/github/actions/workflow/status/simplyRoba/gazel/ci.yml?link=https%3A%2F%2Fgithub.com%2FsimplyRoba%2Fgazel%2Factions%2Fworkflows%2Fci.yml%3Fquery%3Dbranch%253Amain" alt="GitHub Workflow Status" />
  <a href="https://github.com/simplyRoba/gazel/releases"><img src="https://img.shields.io/github/v/release/simplyRoba/gazel?link=https%3A%2F%2Fgithub.com%2FsimplyRoba%2Fgazel%2Freleases" alt="GitHub release" /></a>
  <a href="https://github.com/simplyRoba/gazel/issues"><img src="https://img.shields.io/github/issues/simplyRoba/gazel?link=https%3A%2F%2Fgithub.com%2FsimplyRoba%2Fgazel%2Fissues" alt="GitHub issues" /></a>
  <img src="https://img.shields.io/github/stars/simplyRoba/gazel" alt="GitHub Repo stars" />
</p>

Fuel expense and mileage tracker — the gazel remembers every drop so you don't have to.

## Features

<!-- TODO: define features as they are implemented -->

## Quick start

### Docker run

```bash
docker run -p 4100:4100 -v gazel-data:/data \
  ghcr.io/simplyroba/gazel:latest
```

Open `http://localhost:4100`. Data is persisted in the `gazel-data` volume.

### Docker Compose

A `docker-compose.yml` is included in the repository.

```bash
docker compose up -d
```

## Configuration

<!-- TODO: define configuration environment variables -->

## Security

gazel has no built-in authentication. It is designed to run on a trusted home network or behind a reverse proxy that handles auth (e.g., Authelia, Authentik, Caddy with basic auth). Do not expose it directly to the internet.

---

**This project is developed spec-driven with AI assistance, reviewed by a critical human.**
