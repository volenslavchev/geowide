FROM rust-search-base AS builder

FROM debian:trixie-slim AS runtime
WORKDIR /app

# RUN apt-get update && apt-get install -y --no-install-recommends \
#     libssl3 \
#     ca-certificates \
#  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/bin/indexer ./indexer

RUN chmod +x ./indexer
ENTRYPOINT ["./indexer"]