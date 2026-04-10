# Stage 1: Install dependencies
FROM oven/bun:1 AS deps
WORKDIR /app
COPY package.json bun.lock ./
RUN bun install --frozen-lockfile --production

# Stage 2: Production image
FROM oven/bun:1-slim
WORKDIR /app

COPY --from=deps /app/node_modules ./node_modules
COPY package.json tsconfig.json ./
COPY src ./src

ENV DATA_DIR=/data
ENV PORT=3000

RUN mkdir -p /data && chown -R bun:bun /data
USER bun

# EXPOSE documents the default port; the actual port is controlled by the PORT env var
EXPOSE 3000

ENTRYPOINT ["sh", "-c", "mkdir -p \"$DATA_DIR\" && exec bun run src/index.ts"]
