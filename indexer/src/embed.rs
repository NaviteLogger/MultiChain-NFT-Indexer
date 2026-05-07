use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub const EMBED_DIM: usize = 384;

/// Hash-trick embedding: tokenise text, project each token into a fixed-size
/// vector via 4 different hash functions (à la Vowpal Wabbit's hashing trick),
/// then L2-normalise. Two strings sharing tokens land near each other in
/// cosine distance.
///
/// This is a stand-in for a real embedding model (OpenAI ada-002, fastembed,
/// sentence-transformers). Swap by changing this function — the schema column
/// already accepts a `vector(384)` of any provenance.
pub fn embed(text: &str) -> Vec<f32> {
    let mut v = vec![0f32; EMBED_DIM];

    for raw in text.split(|c: char| !c.is_alphanumeric()) {
        let tok = raw.to_ascii_lowercase();
        if tok.is_empty() {
            continue;
        }
        for salt in 0u32..4 {
            let mut h = DefaultHasher::new();
            (salt, tok.as_str()).hash(&mut h);
            let bucket = (h.finish() as usize) % EMBED_DIM;
            let sign = if (h.finish() >> 32) & 1 == 0 { 1.0 } else { -1.0 };
            v[bucket] += sign;
        }
    }

    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
    v
}

pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum::<f32>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embed_is_correct_length() {
        assert_eq!(embed("anything").len(), EMBED_DIM);
    }

    #[test]
    fn embed_is_l2_normalised() {
        let v = embed("Genesis: First NFT in the multi-chain demo.");
        let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-5, "expected unit norm, got {norm}");
    }

    #[test]
    fn empty_text_returns_zero_vector() {
        let v = embed("");
        assert!(v.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn similar_texts_score_higher_than_dissimilar() {
        let alpha = embed("genesis nft cyberpunk warrior token");
        let close = embed("genesis nft cyberpunk warrior glyph");
        let far = embed("ipfs://QmCID/random/garbage/token");

        let close_sim = cosine(&alpha, &close);
        let far_sim = cosine(&alpha, &far);
        assert!(
            close_sim > far_sim,
            "expected similar text closer than unrelated; close={close_sim} far={far_sim}"
        );
    }

    #[test]
    fn embedding_is_deterministic() {
        let a = embed("ipfs://QmCID/0.json");
        let b = embed("ipfs://QmCID/0.json");
        assert_eq!(a, b);
    }
}
