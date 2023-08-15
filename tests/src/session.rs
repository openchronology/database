use crate::consts::{PGRST_JWT_KEY, PGRST_JWT_AUD};

use jwt::SignWithKey;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct JWT(String);

impl ToString for JWT {
    fn to_string(&self) -> String {
        format!("Bearer {}", self.0)
    }
}

pub fn gen_jwt(role: &str) -> JWT {
    let role_key = format!("{}/role", *PGRST_JWT_AUD);
    let claims = BTreeMap::from([
        ("aud", *PGRST_JWT_AUD),
        (&role_key, role),
    ]);
    JWT(claims.sign_with_key(&(*PGRST_JWT_KEY)).unwrap())
}
