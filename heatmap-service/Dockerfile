FROM rust:1.69 AS builder

RUN update-ca-certificates

ENV USER=heatmap
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

WORKDIR /heatmap

COPY ./ .

RUN cargo update # required to fix issue with poorly specified dependencies for tokio-postgres
RUN cargo build --release

FROM gcr.io/distroless/cc

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /heatmap

COPY --from=builder /heatmap/target/release/heatmap-service ./

USER heatmap:heatmap

CMD ["/heatmap/heatmap-service"]
