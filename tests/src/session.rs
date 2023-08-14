use crate::consts::{PGRST_JWT_KEY, PGRST_JWT_AUD};

use jwt::SignWithKey;
use std::collections::BTreeMap;

pub fn gen_jwt(role: &str) -> String {
    let role_key = format!("{}/role", *PGRST_JWT_AUD);
    let claims = BTreeMap::from([
        ("aud", *PGRST_JWT_AUD),
        (&role_key, role),
    ]);
    claims.sign_with_key(&(*PGRST_JWT_KEY)).unwrap()
}
