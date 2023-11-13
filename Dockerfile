FROM node:21-alpine3.14 as Frontend
WORKDIR /frontend-aoo
COPY ./trackscape-discord-ui /frontend-app


FROM rust:1.72-bookworm as Backend
WORKDIR /app
COPY . /app
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall cargo-shuttle -y --version 0.27.0
RUN cargo fetch
# compile app
RUN cargo build --release
EXPOSE 8000
EXPOSE 8001