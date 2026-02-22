# This dockerfile builds the API and runs it on a minimal container
ARG STORAGE_ADAPTOR

FROM docker.io/library/alpine:latest as builder

RUN --mount=type=cache,target=/etc/apk/cache \
    apk add curl rust cargo pkgconfig openssl-dev

WORKDIR /usr/src/app
# Will build and cache the binary and dependent crates in release mode
RUN --mount=type=bind,target=/usr/src/app \
    --mount=type=cache,target=target \
    --mount=type=cache,target=/root/.cargo \
    cargo build --release --locked --features ${STORAGE_ADAPTOR:-sql}-adaptor && mv ./target/release/crabfit-api /api

# Runtime image
FROM docker.io/library/alpine:latest

RUN apk --no-cache add libgcc

# Run as "app" user
RUN adduser -D app

USER app
WORKDIR /app

# Get compiled binaries from builder's cargo install directory
COPY --from=builder /api /app/api

# Run the app
EXPOSE 3000
CMD ["/app/api"]
