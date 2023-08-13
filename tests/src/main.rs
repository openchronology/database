use tests::{
    tests::run_tests,
    prepare_db,
};

use quickcheck::Gen;

#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();
    let mut g = Gen::new(10);

    prepare_db(&client, &mut g).await;

    run_tests(&client, &mut g).await;
}

