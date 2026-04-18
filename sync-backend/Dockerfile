# Stage 1: Install dependencies (includes native addon compilation)
FROM node:20 AS deps
WORKDIR /app
COPY package.json ./
RUN npm install --production

# Stage 2: Production image
FROM node:20-slim
WORKDIR /app

COPY --from=deps /app/node_modules ./node_modules
COPY package.json tsconfig.json ./
COPY src ./src

ENV DATA_DIR=/data
ENV SCHEMA_DIR=/schemas
ENV PORT=3000

RUN mkdir -p /data /schemas && chown -R node:node /data /schemas
USER node

# EXPOSE documents the default port; the actual port is controlled by the PORT env var
EXPOSE 3000

ENTRYPOINT ["sh", "-c", "mkdir -p \"$DATA_DIR\" \"$SCHEMA_DIR\" && exec node --import tsx src/index.ts"]
