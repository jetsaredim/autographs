FROM node:22-alpine AS deps

WORKDIR /workspace

COPY package.json pnpm-workspace.yaml pnpm-lock.yaml ./
COPY app/package.json app/package.json
RUN corepack enable && corepack prepare pnpm@11.1.2 --activate && pnpm install --frozen-lockfile --filter app...

FROM node:22-alpine AS builder

WORKDIR /workspace

COPY --from=deps /workspace/node_modules ./node_modules
COPY --from=deps /workspace/app/node_modules ./app/node_modules
COPY package.json pnpm-workspace.yaml pnpm-lock.yaml ./
COPY app/package.json app/package.json
COPY app/next.config.ts app/next.config.ts
COPY app/tsconfig.json app/tsconfig.json
COPY app/next-env.d.ts app/next-env.d.ts
COPY app/eslint.config.mjs app/eslint.config.mjs
COPY app/app app/app
COPY app/src app/src
COPY app/db app/db
COPY app/scripts app/scripts
COPY app/public app/public

RUN corepack enable && corepack prepare pnpm@11.1.2 --activate && pnpm --filter app build

FROM node:22-alpine AS runner

WORKDIR /workspace/app
ENV NODE_ENV=production
ENV PORT=3000

COPY --from=builder /workspace/app/.next/standalone ./
COPY --from=builder /workspace/app/.next/static ./app/.next/static
COPY --from=builder /workspace/app/public ./app/public

EXPOSE 3000

CMD ["node", "app/server.js"]

FROM builder as tools

WORKDIR /workspace

ENV NODE_ENV=production
ENV NODE_OPTIONS=--max-old-space-size=384

CMD ["pnpm", "--filter", "app", "data:smoke"]
