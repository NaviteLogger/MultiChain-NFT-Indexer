CREATE EXTENSION IF NOT EXISTS vector;

ALTER TABLE nfts ADD COLUMN IF NOT EXISTS embedding vector(384);

CREATE INDEX IF NOT EXISTS nfts_embedding_idx
    ON nfts USING ivfflat (embedding vector_cosine_ops)
    WITH (lists = 50);
