use realworld_rust_app::run;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run().await
}
