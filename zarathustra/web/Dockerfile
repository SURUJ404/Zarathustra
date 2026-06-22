FROM rust:1.81-slim-bookworm AS rust-builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY zarathustra_common/ ./zarathustra_common/
COPY zarathustra_field/ ./zarathustra_field/
COPY zarathustra_ast/ ./zarathustra_ast/
COPY zarathustra_analysis/ ./zarathustra_analysis/
COPY zarathustra_codegen/ ./zarathustra_codegen/
COPY zarathustra_core/ ./zarathustra_core/
COPY zarathustra_interpreter/ ./zarathustra_interpreter/
COPY zarathustra_embed/ ./zarathustra_embed/
COPY zarathustra_abi/ ./zarathustra_abi/
COPY zarathustra_fs_resolver/ ./zarathustra_fs_resolver/
COPY zarathustra_proof_systems/ ./zarathustra_proof_systems/
COPY zarathustra_bellman/ ./zarathustra_bellman/
COPY zarathustra_bellperson/ ./zarathustra_bellperson/
COPY zarathustra_ark/ ./zarathustra_ark/
COPY zarathustra_circom/ ./zarathustra_circom/
COPY zarathustra_profiler/ ./zarathustra_profiler/
COPY zarathustra_stdlib/ ./zarathustra_stdlib/
COPY zarathustra_cli/ ./zarathustra_cli/
COPY zarathustra_js/ ./zarathustra_js/
COPY zarathustra_pest_ast/ ./zarathustra_pest_ast/
COPY zarathustra_test_derive/ ./zarathustra_test_derive/
COPY zarathustra.rs ./

RUN cargo build --release --locked --package zarathustra_cli && \
    cp target/release/zarathustra /usr/local/bin/zarathustra && \
    strip /usr/local/bin/zarathustra

FROM node:20-slim-bookworm AS node-builder

WORKDIR /app
COPY zarathustra/web/package*.json ./
RUN npm ci --omit=dev

FROM node:20-slim-bookworm AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -r zarathustra && useradd -r -g zarathustra -m -d /app zarathustra

COPY --from=rust-builder /usr/local/bin/zarathustra /usr/local/bin/zarathustra
COPY --from=node-builder /app/node_modules /app/node_modules

WORKDIR /app
COPY zarathustra/web/package.json zarathustra/web/server.js ./
COPY zarathustra/web/public/ ./public/
COPY zarathustra/stdlib/ ./stdlib/
COPY zarathustra/web/ecosystem.config.js ./

RUN mkdir -p /app/work && \
    chown -R zarathustra:zarathustra /app

USER zarathustra
EXPOSE 4000

ENV PORT=4000
ENV HOST=0.0.0.0
ENV NODE_ENV=production
ENV WORK_DIR=/app/work
ENV ZARATHUSTRA_BIN=/usr/local/bin
ENV ZARATHUSTRA_STDLIB=/app/stdlib

HEALTHCHECK --interval=30s --timeout=5s --start-period=30s --retries=3 \
    CMD node -e "require('http').get('http://localhost:4000/api/health', r => {process.exit(r.statusCode === 200 ? 0 : 1)}).on('error', () => process.exit(1))"

CMD ["node", "server.js"]
