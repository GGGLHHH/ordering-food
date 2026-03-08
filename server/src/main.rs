use ordering_food_server::app::run;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run().await
}
