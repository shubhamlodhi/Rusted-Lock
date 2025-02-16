use jsonwebtoken::{decode, DecodingKey, Validation, TokenData};
use serde::{Serialize, Deserialize};
use chrono::Utc;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: usize,
}

pub async fn validate_jwt(token: &str, secret: &str) -> Result<TokenData<Claims>, String> {
    let validation = Validation::default();
    match decode::<Claims>(token, &DecodingKey::from_secret(secret.as_ref()), &validation) {
        Ok(token_data) => {
            // Check if the token is expired
            if token_data.claims.exp < Utc::now().timestamp() as usize {
                Err("Token has expired".to_string())
            } else {
                Ok(token_data)
            }
        }
        Err(_) => Err("Invalid token".to_string()),
    }
}