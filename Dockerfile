FROM rust as builder
WORKDIR /usr/src/monithor
COPY . .

RUN cargo install --path .

FROM debian:bookworm-slim
#RUN apt-get update && apt-get install -y libssl-dev build-essential curl glibc-source && rm -rf /var/lib/apt/lists/*
# RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/monithor_web /usr/local/bin/monithor_web
CMD ["monithor_web"]
EXPOSE 8899 
EXPOSE 7878
