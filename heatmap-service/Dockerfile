FROM rust:1.86 AS builder

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

COPY ./heatmap-service ./heatmap-service
COPY ./heatmap-api ./heatmap-api

RUN <<EOF
echo "SERVER_ADDRESS=127.0.0.1:8000" >> ./.env
echo "CACHE_TTL=3600" >> ./.env
echo "HEATMAP_GEO_JSON_PATH=/heatmap/sat_data.geojson" >> ./.env
echo "GEO_JSON_PATH=/heatmap/sat_data.geojson" >> ./.env
EOF

# --release would be preferable here but I run out of memory with the flag
RUN cargo build --release --manifest-path=./heatmap-service/Cargo.toml  

FROM gcr.io/distroless/cc

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /heatmap

COPY --from=builder /heatmap/heatmap-service/target/debug/heatmap-service ./
COPY --from=builder /heatmap/heatmap-service/sat_data.geojson ./
COPY --from=builder /heatmap/.env ./


USER heatmap:heatmap

CMD ["/heatmap/heatmap-service"]
