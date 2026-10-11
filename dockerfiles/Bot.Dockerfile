FROM rust:1.86-bookworm AS bot-builder
WORKDIR /app
COPY . /app
RUN cargo build --bin trackscape-discord-bot --release
#
FROM rust:1.86-slim-bookworm AS bot
COPY --from=bot-builder /app/target/release/trackscape-discord-bot /usr/local/bin/trackscape-discord-bot
CMD ["trackscape-discord-bot"]
