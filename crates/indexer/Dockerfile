FROM rust-search-base AS builder

FROM debian:trixie-slim AS runtime
WORKDIR /app

COPY --from=builder /usr/local/bin/indexer ./indexer
COPY ./model_export/megaloc.onnx ./megaloc.onnx
COPY ./model_export/megaloc.onnx.data ./megaloc.onnx.data

RUN chmod +x ./indexer
ENTRYPOINT ["./indexer", "serve"]