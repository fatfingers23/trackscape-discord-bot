FROM node:lts-alpine3.18 AS frontend
WORKDIR /frontend-app
COPY ./trackscape-discord-ui /frontend-app
RUN npm install
RUN npm run build

FROM rust:1.78-bookworm AS backend
WORKDIR /app
COPY . /app
COPY --from=frontend /frontend-app/dist /app/trackscape-discord-api/ui
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall cargo-shuttle -y --version 0.48.1
RUN cargo fetch
# compile app
RUN cargo build --release
EXPOSE 8000
EXPOSE 8001
