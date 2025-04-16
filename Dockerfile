# syntax=docker/dockerfile:1
FROM rust:1.86.0-bullseye AS builder

ADD . .

ENV RUSTFLAGS="-C target-cpu=haswell -C opt-level=3"

RUN cargo build --release

FROM rust:1.86.0-slim-bullseye AS runner

COPY --from=builder --chown=65534 /target/release/portfolio-studio /usr/local/bin
ADD --chown=65534 /templates /templates
ADD --chown=65534 /static /static

ENV RUST_BACKTRACE=full

ENV PORT=${PORT:-8080}

EXPOSE $PORT

USER 65534

CMD ["/usr/local/bin/portfolio-studio"]