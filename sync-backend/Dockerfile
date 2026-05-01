# Stage 1: Install deps and build TypeScript
FROM node:20 AS build
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY tsconfig.json tsconfig.build.json ./
COPY src ./src
RUN npx tsc -p tsconfig.build.json
# Prune devDependencies so only production deps remain for the runtime image
RUN npm prune --omit=dev

# Stage 2: Production image (only production deps + compiled JS)
FROM node:20-slim
WORKDIR /app

COPY --from=build /app/node_modules ./node_modules
COPY --from=build /app/dist ./dist
COPY package.json ./
COPY schemas ./schemas

ENV DATA_DIR=/data
ENV SCHEMA_DIR=/app/schemas
ENV PORT=3000

RUN mkdir -p /data /schemas && chown -R node:node /data /schemas
USER node

# EXPOSE documents the default port; the actual port is controlled by the PORT env var
EXPOSE 3000

ENTRYPOINT ["sh", "-c", "mkdir -p \"$DATA_DIR\" \"$SCHEMA_DIR\" && exec node dist/index.js"]

