FROM rust:1.84 AS builder

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

COPY ./ ./

# Service config values
RUN <<EOF
echo "SERVER_ADDRESS=0.0.0.0:8000" >> ./.env
echo "CACHE_TTL=3600" >> ./.env
echo "HEATMAP_GEO_JSON_PATH=/heatmap/sat_data.geojson" >> ./.env
echo "GEO_JSON_PATH=/heatmap/sat_data.geojson" >> ./.env
EOF

RUN cargo build --release --package heatmap-service

FROM gcr.io/distroless/cc

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /heatmap

COPY --from=builder /heatmap/target/release/heatmap-service ./
COPY --from=builder /heatmap/heatmap-service/sat_data.geojson ./
COPY --from=builder /heatmap/.env ./

USER heatmap:heatmap

EXPOSE 8000

CMD [ "/heatmap/heatmap-service" ]
