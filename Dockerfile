FROM rust:1.72
WORKDIR /usr/src/app
COPY . .
CMD ["carg"
CMD ["cargo", "build", "--release"]