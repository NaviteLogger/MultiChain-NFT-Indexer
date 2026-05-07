# IndexedNFT subgraph (The Graph)

The same indexing job the Rust service performs, expressed as a Graph
subgraph. Kept here as a **comparison alternative** rather than the
canonical pipeline — the parent `indexer/` directory is what runs in
production.

## Trade-offs

| Aspect              | Custom Rust indexer                              | The Graph subgraph                       |
| ------------------- | ------------------------------------------------ | ---------------------------------------- |
| Hosting             | Self-hosted (Postgres + Docker / K8s)            | Hosted Service / The Graph Network       |
| Realtime push       | First-class WebSocket (`/ws/events`)             | Polled GraphQL (no native subscriptions) |
| Custom logic        | Anything Rust can express                        | AssemblyScript handlers + schema only    |
| Auth-gated views    | Native (SIWE → JWT in `/api/me`)                 | Bring-your-own gateway                   |
| Vector / similarity | pgvector embeddings on the same row              | Out of scope for The Graph               |
| Operational cost    | Postgres + indexer process to run                | Indexing fees / hosted service fees      |
| Cold-start time     | Fast (binary boot)                               | Slow (sync from start block)             |

## Files

```
subgraph.yaml      Manifest — datasources, ABI, event handlers
schema.graphql     Entity types: Token, Transfer, Holder
abis/IndexedNFT.json  Minimal ABI with the two events we index
src/mapping.ts     AssemblyScript handlers (handleTransfer, handleMinted)
```

## Deploy

```bash
npm install                         # graph-cli + graph-ts
npm run codegen                     # generates AssemblyScript types
npm run build                       # compiles to wasm + IPFS metadata
npm run deploy-studio               # to hosted Studio (needs auth)
# or local stack:
npm run create-local && npm run deploy-local
```

Update `subgraph.yaml` `source.address` / `startBlock` after deploying
`IndexedNFT` to your target chain (and re-run `codegen`).
