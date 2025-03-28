FROM rust:1.85 as builder
WORKDIR /app
COPY . .
RUN rustup default nightly-2025-03-28
RUN cargo build --release

FROM rust:1.85-slim
WORKDIR /app
COPY --from=builder /app/datasets /app/datasets
COPY --from=builder /app/static /app/static
COPY --from=builder /app/target /app/target
CMD ["./target/release/spidershield"]
