# MultiChain NFT Indexer

A Rust-based event indexer + Next.js analytics dashboard for ERC-721 contracts.
Watches an EVM chain via WebSocket logs (alloy + tokio), normalises Transfer /
Minted events into Postgres, and exposes them through a typed REST + WebSocket
surface authenticated with **Sign-In With Ethereum** (SIWE → JWT).

Pairs with [`Multi-Chain-NFT-DApp`](https://github.com/NaviteLogger/Multi-Chain-NFT-DApp)
— this indexer can be pointed at the IndexedNFT contract here, or at the
MultiChainNFT contract in that repo.

## Stack

| Layer        | Tooling                                                                        |
| ------------ | ------------------------------------------------------------------------------ |
| EVM contract | Foundry · Solidity 0.8.24 · OpenZeppelin (ERC-721 URIStorage + Ownable)        |
| Indexer      | Rust 1.82 · tokio · axum · sqlx (Postgres) · alloy (`provider-ws` + `pubsub`)  |
| Auth         | `siwe` crate · HS256 JWT (`jsonwebtoken`) · single-use nonces in Postgres      |
| Realtime     | tokio broadcast channel → `axum::ws` `/ws/events` fanout                       |
| API spec     | Hand-written OpenAPI 3 (`indexer/openapi.yaml`)                                |
| Database     | Postgres 16 + pgvector (schema reserved for embeddings)                        |
| Dashboard    | Next.js 14 · wagmi v2 · RainbowKit · `siwe` (browser) · Tailwind               |
| Deploy       | docker-compose (Postgres + indexer + dashboard)                                |

## Repo layout

```
contracts-evm/    Foundry — IndexedNFT.sol with Transfer + Minted events the indexer consumes.
indexer/          Rust crate — chain watcher, REST API, SIWE auth, WebSocket fanout, pgvector similarity.
  src/main.rs     Bootstrap: load config, start chain task, serve axum.
  src/chain/evm.rs Subscribes to logs, decodes Transfer/Minted, persists, embeds, broadcasts.
  src/embed.rs    Hash-trick text → vector(384); swap for fastembed / OpenAI in one place.
  src/api/        axum routes: stats, events, nfts, nfts/similar, holders, /api/me, /auth/{nonce,verify}.
  src/auth.rs     JWT issue/verify (HS256, 12h).
  src/ws.rs       /ws/events broadcast subscriber.
  migrations/     SQL schema + pgvector extension + ivfflat index on nfts.embedding.
  openapi.yaml    Hand-written API spec.
dashboard/        Next.js 14 — stats, SIWE login, live event feed.
subgraph/         The Graph alternative for the same events (source-only comparison).
docker-compose.yml  Postgres+pgvector + indexer + dashboard local stack.
```

## What's verified

| Suite                                 | Status                                         |
| ------------------------------------- | ---------------------------------------------- |
| `forge test` — `contracts-evm/`       | **6/6 passing** (incl. 1 fuzz test)               |
| `cargo test` — `indexer/`             | **14/14 passing** (auth, validation, embedding)   |
| `cargo build` — `indexer/`            | **green**                                         |
| `npm run build` — `dashboard/`        | **green**                                         |
| `npx playwright test` — `dashboard/`  | **10/10 passing** UI + SIWE round-trip + similar  |
| `redocly lint indexer/openapi.yaml`   | **valid**                                         |

The Playwright SIWE round-trip uses a real `viem` private key to sign EIP-4361
messages and posts them to a small `tests/mock-indexer.mjs` server that runs
the `siwe` JS verifier — same flow the Rust `/auth/verify` route runs in
production, just without the database dependency in CI.

## Quickstart

### 1. Clone with submodules

```bash
git clone --recurse-submodules <repo-url>
git submodule update --init --recursive
```

### 2. EVM contract

```bash
cd contracts-evm
forge build
forge test -vv

# Deploy:
cp .env.example .env   # fill DEPLOYER_PRIVATE_KEY, RPC URL, etc.
forge script script/Deploy.s.sol --rpc-url sepolia --broadcast --verify
```

### 3. Indexer + Postgres (Docker)

```bash
docker compose up --build
# Postgres on :5432, indexer on :8080
```

To run the indexer outside Docker:

```bash
cd indexer
cp .env.example .env   # set DATABASE_URL, EVM_WS_URL, EVM_CONTRACT, JWT_SECRET (>=32 chars)
cargo run --release
```

### 4. Dashboard

```bash
cd dashboard
cp .env.example .env.local   # NEXT_PUBLIC_INDEXER_URL=http://localhost:8080
npm install
npm run dev                  # http://localhost:3000
```

### 5. Tests

```bash
# Foundry
( cd contracts-evm && forge test )

# Rust unit tests (no DB needed)
( cd indexer && cargo test )

# Dashboard E2E (spins up Next.js + a mock indexer that runs the real siwe verifier)
( cd dashboard && npm run test:e2e:install && npm run test:e2e )
```

## Auth flow (SIWE)

1. Browser → `POST /auth/nonce` — server stores a single-use nonce in
   `auth_nonces`.
2. Browser builds an **EIP-4361** message (`siwe` JS package), wallet signs.
3. Browser → `POST /auth/verify { message, signature }`.
4. Server: parse SIWE → confirm `domain` matches `SIWE_DOMAIN` → confirm nonce
   exists and is unused → `Message::verify` (recover signer) → mark nonce
   consumed → issue HS256 JWT (12h) bound to the lowercased address.
5. All gated routes require `Authorization: Bearer <jwt>`. Currently `/api/me`
   demonstrates the middleware path; expand by adding `AuthUser` extractor to
   any handler.

The indexer accepts only signatures that match `SIWE_DOMAIN`, preventing a
malicious site from harvesting valid signatures and reusing them here. Nonces
are single-use, so a replayed `(message, signature)` pair fails on the second
`verify`.

## Similarity search (pgvector)

Each `Minted` event triggers an embedding pass: `src/embed.rs` runs a
deterministic hash-trick projection from the NFT's `(contract, token_id, uri)`
text into a `vector(384)`, persisted onto the `nfts.embedding` column. The
column is indexed with `ivfflat (vector_cosine_ops)`.

`GET /api/nfts/similar/{contract}/{token_id}?limit=N` returns the nearest
neighbours, ordered by `embedding <=> anchor` cosine distance.

The hash-trick embedding is a one-line stand-in — replace with `fastembed-rs`,
OpenAI `ada-002`, or any other 384-dim embedder by editing `src/embed.rs`. The
schema, query, and HTTP surface stay identical.

## The Graph alternative

`subgraph/` contains a parallel indexer expressed as a Graph subgraph:

- `subgraph.yaml` — manifest pointing at the same `IndexedNFT` contract.
- `schema.graphql` — entity types (`Token`, `Transfer`, `Holder`).
- `src/mapping.ts` — AssemblyScript `handleTransfer` / `handleMinted`.
- `abis/IndexedNFT.json` — the two events we care about.

Why both? The Rust indexer wins on push-based realtime (`/ws/events`), custom
auth (`SIWE`), and embeddable extensions like pgvector. The subgraph wins on
operational simplicity and hosted infra. `subgraph/README.md` has the full
trade-off table and deploy commands.

## Indexing flow

1. Indexer connects to `EVM_WS_URL` via alloy `provider-ws`.
2. Subscribes to logs filtered by `EVM_CONTRACT` and the `Transfer` / `Minted`
   topic-0 hashes.
3. Each log: decode via `alloy_sol_types::SolEvent::decode_log`, persist to
   `events`, upsert `nfts.current_owner`, broadcast on the in-process tokio
   channel that fans out to `/ws/events`.
4. `indexer_state.last_block` is the high watermark for resume after restart.

The watcher is reorg-tolerant only at the "latest log idempotency" level —
`UNIQUE (chain_id, tx_hash, log_index)` plus `ON CONFLICT DO NOTHING`. A
production deployment would add a confirmations buffer.

## Why this exists

This is the **second** of two paired portfolio projects, designed to compose:

- [`Multi-Chain-NFT-DApp`](https://github.com/NaviteLogger/Multi-Chain-NFT-DApp)
  — onboarding side: wallet connectors, signature-gated mint, EVM + Sui Move.
- **This repo** — backend side: Rust indexer, Postgres, SIWE-authenticated REST
  + WebSocket dashboard.

Together they show full-stack Web3 work end-to-end across Solidity, TypeScript,
and Rust.

## License

MIT.
