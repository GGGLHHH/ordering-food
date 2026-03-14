FROM node:22-bookworm-slim AS builder

RUN corepack enable && npm i -g vite-plus

WORKDIR /app

COPY package.json pnpm-lock.yaml ./
RUN vp install --frozen-lockfile

COPY . .
RUN vp build

FROM nginx:stable-alpine AS runtime

COPY --from=builder /app/dist /usr/share/nginx/html
COPY docker/nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 3000

CMD ["nginx", "-g", "daemon off;"]
