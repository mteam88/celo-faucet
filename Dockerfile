# Stage 1: Build the frontend
FROM node:20-slim AS frontend-builder
WORKDIR /app/web
COPY web/package.json web/package-lock.json ./
RUN npm install
COPY web/ ./
RUN npm run build

# Stage 2: Build the backend
FROM rust:1-slim AS backend-builder
WORKDIR /app
RUN apt-get update && apt-get install -y build-essential pkg-config libssl-dev
COPY . .
RUN cargo build --release --locked

# Stage 3: Create the final image
FROM debian:12-slim
WORKDIR /app
COPY --from=backend-builder /app/target/release/celo-faucet /usr/local/bin/
COPY --from=frontend-builder /app/web/dist ./web/dist
COPY --from=frontend-builder /app/web/assets ./web/assets
COPY --from=frontend-builder /app/web/index.html ./web/index.html

EXPOSE 8080
ENV RUST_LOG=info
CMD ["celo-faucet"]
