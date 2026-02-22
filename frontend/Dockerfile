FROM docker.io/library/node:alpine AS builder

WORKDIR /app

ENV NEXT_TELEMETRY_DISABLED=1
RUN --mount=type=bind,target=/app,rw \
    --mount=type=cache,target=/root/.npm \
    npm ci --no-audit --no-fund && \
    npm run build && \
    cp -r /app/.next/standalone /output && \
    cp -r /app/public /output/public && \
    cp -r /app/.next/static /output/.next/static

FROM docker.io/library/node:alpine

ENV NODE_ENV=production
ENV PORT=3000
ENV HOSTNAME="0.0.0.0"
ENV NEXT_TELEMETRY_DISABLED=1

COPY --from=builder --chown=node:node /output /app

USER node
WORKDIR /app
EXPOSE 3000
CMD ["node", "server.js"]
