FROM backend-template-base AS builder


FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 \
    ca-certificates \
 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/indexer ./indexer

RUN chmod +x ./indexer
ENTRYPOINT ["./indexer"]