use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
}

pub fn issue(secret: &str, address: &str) -> AppResult<String> {
    let now = Utc::now();
    let exp = now + Duration::hours(12);
    let claims = Claims {
        sub: address.to_lowercase(),
        iat: now.timestamp(),
        exp: exp.timestamp(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("jwt encode: {e}")))
}

pub fn verify(secret: &str, token: &str) -> AppResult<Claims> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized)?;
    Ok(data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "this-is-a-very-secret-string-min-32-chars";

    #[test]
    fn round_trip_issue_and_verify() {
        let token = issue(SECRET, "0xAbC0000000000000000000000000000000000001").unwrap();
        let claims = verify(SECRET, &token).unwrap();
        assert_eq!(claims.sub, "0xabc0000000000000000000000000000000000001");
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn verify_rejects_wrong_secret() {
        let token = issue(SECRET, "0xAbC0000000000000000000000000000000000001").unwrap();
        let err = verify("a-different-secret-also-32-chars-long!", &token).unwrap_err();
        assert!(matches!(err, AppError::Unauthorized));
    }

    #[test]
    fn verify_rejects_garbage_token() {
        let err = verify(SECRET, "not.a.jwt").unwrap_err();
        assert!(matches!(err, AppError::Unauthorized));
    }

    #[test]
    fn issued_address_is_lowercased() {
        let token = issue(SECRET, "0xAbC0000000000000000000000000000000000001").unwrap();
        let claims = verify(SECRET, &token).unwrap();
        assert_eq!(claims.sub, claims.sub.to_lowercase());
    }
}
