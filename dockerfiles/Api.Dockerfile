FROM node:lts-alpine3.18 AS frontend
WORKDIR /frontend-app
COPY ./trackscape-discord-ui /frontend-app
RUN npm install
RUN npm run build

FROM rust:1.86-bookworm AS api-builder
WORKDIR /app
COPY . /app
RUN cargo build --bin trackscape-discord-api --release
#
FROM rust:1.86-slim-bookworm AS api
WORKDIR /app
COPY --from=api-builder /app/target/release/trackscape-discord-api /usr/local/bin/trackscape-discord-api
COPY --from=frontend /frontend-app/dist /app/trackscape-discord-api/ui
EXPOSE 8000
CMD ["trackscape-discord-api"]
