FROM node:24-alpine AS frontend
WORKDIR /frontend-app
COPY ./trackscape-discord-ui /frontend-app
# Use the pnpm version pinned by the UI's packageManager field so the image
# builds the exact dependency tree recorded in pnpm-lock.yaml.
RUN corepack enable && pnpm install --frozen-lockfile
RUN pnpm run build

FROM rust:1.99-bookworm AS api-builder
ENV SQLX_OFFLINE=true
WORKDIR /app
COPY . /app
RUN cargo build --bin trackscape-discord-api --release --locked
#
FROM rust:1.99-slim-bookworm AS api
WORKDIR /app
COPY --from=api-builder /app/target/release/trackscape-discord-api /usr/local/bin/trackscape-discord-api
COPY --from=frontend /frontend-app/dist /app/trackscape-discord-api/ui
EXPOSE 8000
CMD ["trackscape-discord-api"]
