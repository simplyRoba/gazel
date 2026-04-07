# Base image with common devcontainer tools
FROM mcr.microsoft.com/devcontainers/base:ubuntu

# System dependencies
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        curl \
        git \
        ca-certificates \
        unzip \
        gnupg \
        lsb-release \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Chromium / Playwright system library dependencies.
# These are the shared libraries Playwright's Chromium binary requires at
# runtime.  Installing them as root during image build avoids sudo/permission
# issues at postCreateCommand time and works on both amd64 and arm64 (Apple
# Silicon devcontainers).
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        # Core libs required by Chromium
        libnss3 \
        libnspr4 \
        libatk1.0-0 \
        libatk-bridge2.0-0 \
        libcups2 \
        libdrm2 \
        libxkbcommon0 \
        libatspi2.0-0 \
        libxcomposite1 \
        libxdamage1 \
        libxfixes3 \
        libxrandr2 \
        libgbm1 \
        libpango-1.0-0 \
        libcairo2 \
        libasound2t64 \
        libwayland-client0 \
        # Fonts so rendered pages are not blank boxes
        fonts-liberation \
        fonts-noto-color-emoji \
        # Misc runtime deps
        libx11-xcb1 \
        libxcb1 \
        libx11-6 \
        libxext6 \
        libdbus-1-3 \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Starship prompt installation
RUN curl -sS https://starship.rs/install.sh | sh -s -- -y
