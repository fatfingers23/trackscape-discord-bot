FROM rust:1.72-bullseye
WORKDIR /app
COPY . /app
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall cargo-shuttle -y
RUN cargo fetch
# compile app
RUN cargo build --release
EXPOSE 8000
EXPOSE 8001