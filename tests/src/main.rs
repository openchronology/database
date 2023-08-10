#[macro_use]
extern crate dotenv_codegen;
#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref PGRST_HOST: &'static str = dotenv!("PGRST_HOST");
    static ref PGRST_JWT_SECRET: &'static str = dotenv!("PGRST_JWT_SECRET");
    static ref PGRST_JWT_AUD: &'static str = dotenv!("PGRST_JWT_AUD");
}

fn main() {
    println!("Hello, world!, {}", *PGRST_HOST);
}
