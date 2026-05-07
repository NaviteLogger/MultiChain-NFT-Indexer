pub fn is_evm_address(s: &str) -> bool {
    s.len() == 42 && s.starts_with("0x") && s[2..].chars().all(|c| c.is_ascii_hexdigit())
}

pub fn is_token_id(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    if !s.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    s == "0" || !s.starts_with('0')
}

pub fn is_tx_hash(s: &str) -> bool {
    s.len() == 66 && s.starts_with("0x") && s[2..].chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evm_address_accepts_well_formed_addresses() {
        assert!(is_evm_address("0x70997970c51812dc3a010c7d01b50e0d17dc79c8"));
        assert!(is_evm_address("0x0000000000000000000000000000000000000000"));
        assert!(is_evm_address("0xABCDEF0123456789ABCDEF0123456789ABCDEF01"));
    }

    #[test]
    fn evm_address_rejects_malformed() {
        assert!(!is_evm_address(""));
        assert!(!is_evm_address("0x"));
        assert!(!is_evm_address("70997970c51812dc3a010c7d01b50e0d17dc79c8"));
        assert!(!is_evm_address("0xZZ97970c51812dc3a010c7d01b50e0d17dc79c8"));
        assert!(!is_evm_address("0x70997970c51812dc3a010c7d01b50e0d17dc79c"));
    }

    #[test]
    fn token_id_accepts_digits_no_leading_zero() {
        assert!(is_token_id("0"));
        assert!(is_token_id("1"));
        assert!(is_token_id("42"));
        assert!(is_token_id(
            "115792089237316195423570985008687907853269984665640564039457584007913129639935"
        ));
    }

    #[test]
    fn token_id_rejects_leading_zero_or_non_digits() {
        assert!(!is_token_id(""));
        assert!(!is_token_id("00"));
        assert!(!is_token_id("0042"));
        assert!(!is_token_id("0x1"));
        assert!(!is_token_id("1.5"));
        assert!(!is_token_id("-1"));
    }

    #[test]
    fn tx_hash_validation() {
        assert!(is_tx_hash(&format!("0x{}", "a".repeat(64))));
        assert!(!is_tx_hash(&format!("0x{}", "a".repeat(63))));
        assert!(!is_tx_hash("0xnope"));
    }
}
