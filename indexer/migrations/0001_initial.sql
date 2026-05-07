CREATE TABLE IF NOT EXISTS events (
    id              BIGSERIAL PRIMARY KEY,
    chain_id        BIGINT       NOT NULL,
    contract        TEXT         NOT NULL,
    event           TEXT         NOT NULL,
    from_address    TEXT,
    to_address      TEXT         NOT NULL,
    token_id        TEXT         NOT NULL,
    uri             TEXT,
    tx_hash         TEXT         NOT NULL,
    log_index       BIGINT       NOT NULL,
    block_number    BIGINT       NOT NULL,
    block_timestamp TIMESTAMPTZ  NOT NULL,
    UNIQUE (chain_id, tx_hash, log_index)
);

CREATE INDEX IF NOT EXISTS events_block_idx
    ON events (chain_id, block_number DESC, log_index DESC);
CREATE INDEX IF NOT EXISTS events_to_idx
    ON events (chain_id, to_address);
CREATE INDEX IF NOT EXISTS events_contract_token_idx
    ON events (chain_id, contract, token_id);

CREATE TABLE IF NOT EXISTS nfts (
    chain_id            BIGINT       NOT NULL,
    contract            TEXT         NOT NULL,
    token_id            TEXT         NOT NULL,
    current_owner       TEXT         NOT NULL,
    uri                 TEXT,
    mint_block          BIGINT       NOT NULL,
    mint_tx             TEXT         NOT NULL,
    last_transfer_block BIGINT       NOT NULL,
    last_updated        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    PRIMARY KEY (chain_id, contract, token_id)
);

CREATE INDEX IF NOT EXISTS nfts_owner_idx ON nfts (chain_id, current_owner);

CREATE TABLE IF NOT EXISTS auth_nonces (
    nonce      TEXT        PRIMARY KEY,
    issued_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    consumed   BOOLEAN     NOT NULL DEFAULT FALSE
);

CREATE INDEX IF NOT EXISTS auth_nonces_issued_idx ON auth_nonces (issued_at);

CREATE TABLE IF NOT EXISTS indexer_state (
    chain_id        BIGINT PRIMARY KEY,
    last_block      BIGINT NOT NULL,
    last_block_time TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
