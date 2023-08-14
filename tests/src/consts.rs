use hmac::{Hmac, Mac};
use sha2::Sha256;

lazy_static! {
    pub static ref PGRST_HOST: &'static str = dotenv!("PGRST_HOST");
    pub static ref PGRST_JWT_SECRET: &'static str = dotenv!("PGRST_JWT_SECRET");
    pub static ref PGRST_JWT_AUD: &'static str = dotenv!("PGRST_JWT_AUD");

    pub static ref PGRST_JWT_KEY: Hmac<Sha256> = Hmac::new_from_slice(
        (*PGRST_JWT_SECRET).as_bytes()
    ).unwrap();
}
