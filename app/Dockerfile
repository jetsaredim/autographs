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

WORKDIR /workspace
ENV NODE_ENV=production
ENV PORT=3000

RUN corepack enable && corepack prepare pnpm@11.1.2 --activate

COPY --from=builder /workspace/node_modules ./node_modules
COPY --from=builder /workspace/app/node_modules ./app/node_modules
COPY --from=builder /workspace/package.json ./package.json
COPY --from=builder /workspace/pnpm-workspace.yaml ./pnpm-workspace.yaml
COPY --from=builder /workspace/pnpm-lock.yaml ./pnpm-lock.yaml
COPY --from=builder /workspace/app/package.json ./app/package.json
COPY --from=builder /workspace/app/next.config.ts ./app/next.config.ts
COPY --from=builder /workspace/app/tsconfig.json ./app/tsconfig.json
COPY --from=builder /workspace/app/public ./app/public
COPY --from=builder /workspace/app/src ./app/src
COPY --from=builder /workspace/app/db ./app/db
COPY --from=builder /workspace/app/scripts ./app/scripts
COPY --from=builder /workspace/app/.next ./app/.next

EXPOSE 3000

CMD ["pnpm", "--filter", "app", "exec", "next", "start", "--hostname", "0.0.0.0", "--port", "3000"]
