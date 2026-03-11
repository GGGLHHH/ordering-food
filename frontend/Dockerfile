FROM node:22-bookworm-slim AS builder

ENV PNPM_HOME=/pnpm
ENV PATH=$PNPM_HOME:$PATH
RUN corepack enable

WORKDIR /app

COPY package.json pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile

COPY . .
RUN pnpm build

FROM node:22-bookworm-slim AS runtime

WORKDIR /app

COPY --from=builder /app/.output /app/.output
COPY --from=builder /app/public /app/public

WORKDIR /app/.output/server
RUN npm install --omit=dev --ignore-scripts

WORKDIR /app
ENV NODE_ENV=production \
    NITRO_HOST=0.0.0.0 \
    NITRO_PORT=3000

EXPOSE 3000

USER node

CMD ["node", ".output/server/index.mjs"]
