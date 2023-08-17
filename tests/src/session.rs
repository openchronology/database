use crate::consts::{PGRST_JWT_KEY, PGRST_JWT_AUD};

use jwt::SignWithKey;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Serialize, Serializer, ser::SerializeMap};

#[derive(Debug)]
pub struct JWT(String);

impl ToString for JWT {
    fn to_string(&self) -> String {
        format!("Bearer {}", self.0)
    }
}

#[derive(Debug)]
struct Claims<'a> {
    aud: &'a str,
    iat: u64,
    exp: u64,
    role_key: &'a str,
    role: &'a str,
}

impl<'a> Serialize for Claims<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let mut state = serializer.serialize_map(Some(4))?;
        state.serialize_entry("aud", self.aud)?;
        state.serialize_entry("iat", &self.iat)?;
        state.serialize_entry("exp", &self.exp)?;
        state.serialize_entry(self.role_key, self.role)?;
        state.end()
    }
}

pub fn gen_jwt(role: &str) -> JWT {
    let role_key = format!("{}/role", *PGRST_JWT_AUD);
    let now = SystemTime::now();
    let exp = now + Duration::from_secs(60 * 5); // 5 minutes
    let iat = now.duration_since(UNIX_EPOCH).unwrap().as_secs();
    let exp = exp.duration_since(UNIX_EPOCH).unwrap().as_secs();
    let claims = Claims {
        aud: *PGRST_JWT_AUD,
        iat,
        exp,
        role_key: &role_key,
        role,
    };
    JWT(claims.sign_with_key(&(*PGRST_JWT_KEY)).unwrap())
}
