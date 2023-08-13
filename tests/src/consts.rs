
lazy_static! {
    pub static ref PGRST_HOST: &'static str = dotenv!("PGRST_HOST");
    pub static ref PGRST_JWT_SECRET: &'static str = dotenv!("PGRST_JWT_SECRET");
    pub static ref PGRST_JWT_AUD: &'static str = dotenv!("PGRST_JWT_AUD");
}
