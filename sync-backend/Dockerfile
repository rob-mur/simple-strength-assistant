# Stage 1: Build TypeScript and install production dependencies
FROM node:20 AS build
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY tsconfig.json tsconfig.build.json ./
COPY src ./src
RUN npx tsc -p tsconfig.build.json

# Stage 2: Production image (only production deps + compiled JS)
FROM node:20-slim
WORKDIR /app

COPY package.json package-lock.json ./
RUN npm ci --production

COPY --from=build /app/dist ./dist

ENV DATA_DIR=/data
ENV SCHEMA_DIR=/schemas
ENV PORT=3000

RUN mkdir -p /data /schemas && chown -R node:node /data /schemas
USER node

# EXPOSE documents the default port; the actual port is controlled by the PORT env var
EXPOSE 3000

ENTRYPOINT ["sh", "-c", "mkdir -p \"$DATA_DIR\" \"$SCHEMA_DIR\" && exec node dist/index.js"]
