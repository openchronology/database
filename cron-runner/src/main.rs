use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    loop {
        sleep(Duration::from_millis(1000)).await;
        println!("Hello, world!");
    }
}
