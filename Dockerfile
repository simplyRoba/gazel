FROM debian:trixie-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

ARG TARGETARCH
COPY release-artifacts/linux-${TARGETARCH}/gazel /usr/local/bin/gazel
RUN chmod +x /usr/local/bin/gazel

RUN mkdir -p /data && chown 1000:1000 /data
VOLUME /data

EXPOSE 4110

HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
  CMD curl -fsS http://localhost:${GAZEL_PORT:-4110}/health || exit 1

USER 1000:1000

CMD ["/usr/local/bin/gazel"]
