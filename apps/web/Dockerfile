FROM ubuntu:22.04 AS base

RUN apt-get update && apt-get install -y curl ca-certificates

RUN mkdir /usr/local/nvm
ENV NVM_DIR /usr/local/nvm
ENV NODE_VERSION 20.6.1

RUN curl https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.1/install.sh | bash \
    && . $NVM_DIR/nvm.sh \
    && nvm install $NODE_VERSION \
    && nvm alias default $NODE_VERSION \
    && nvm use default

ENV NODE_PATH $NVM_DIR/v$NODE_VERSION/lib/node_modules
ENV PATH $NVM_DIR/versions/node/v$NODE_VERSION/bin:$PATH

FROM base as builder
ENV PNPM_HOME="/root/.local/share/pnpm"
ENV PATH="${PATH}:${PNPM_HOME}"
# Set working directory
WORKDIR /app
RUN npm install --global pnpm@8.7.5
RUN pnpm add -g turbo
COPY . .
RUN turbo prune dashboard --docker

# Add lockfile and package.json's of isolated subworkspace
FROM base AS installer

ENV PNPM_HOME="/root/.local/share/pnpm"
ENV PATH="${PATH}:${PNPM_HOME}"

RUN npm install --global pnpm@8.7.5

WORKDIR /app

# First install dependencies (as they change less often)
COPY .gitignore .gitignore
COPY --from=builder /app/out/json/ .
COPY --from=builder /app/out/pnpm-lock.yaml ./pnpm-lock.yaml

COPY --from=builder /app/out/full/ .
COPY turbo.json turbo.json

RUN pnpm i

RUN pnpm turbo run build --filter=dashboard...

FROM base AS runner

ENV PNPM_HOME="/root/.local/share/pnpm"
ENV PATH="${PATH}:${PNPM_HOME}"

RUN npm install --global pnpm@8.7.5

WORKDIR /app

# Don't run production as root
RUN groupadd --system --gid 1001 nextjs
RUN useradd --system --uid 1001 nodejs
USER nodejs
COPY --from=installer /app .

CMD cd apps/dashboard/ && pnpm start
