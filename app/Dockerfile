FROM node:22-alpine AS deps

WORKDIR /app

COPY app/package.json ./
RUN corepack enable && pnpm install --no-frozen-lockfile

FROM node:22-alpine AS builder

WORKDIR /app

COPY --from=deps /app/node_modules ./node_modules
COPY app/package.json ./
COPY app/next.config.ts ./
COPY app/tsconfig.json ./
COPY app/next-env.d.ts ./
COPY app/eslint.config.mjs ./
COPY app/app ./app
COPY app/public ./public

RUN corepack enable && pnpm build

FROM node:22-alpine AS runner

WORKDIR /app
ENV NODE_ENV=production
ENV PORT=3000

RUN corepack enable

COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/package.json ./package.json
COPY --from=builder /app/next.config.ts ./next.config.ts
COPY --from=builder /app/public ./public
COPY --from=builder /app/.next ./.next

EXPOSE 3000

CMD ["pnpm", "exec", "next", "start", "--hostname", "0.0.0.0", "--port", "3000"]
