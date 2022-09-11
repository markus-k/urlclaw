# syntax=docker/dockerfile:1.3
FROM rust:1.63 as builder

WORKDIR /src
COPY ./ /src/

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo install --path urlclaw-server/

FROM debian:bullseye-slim

ENV PORT=80
ENV BIND_ADDR=0.0.0.0

COPY --from=builder /usr/local/cargo/bin/urlclaw-server /usr/local/bin/

EXPOSE $PORT
USER "www-data"

CMD ["urlclaw-server"]
