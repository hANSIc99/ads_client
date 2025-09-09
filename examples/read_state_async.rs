use ads_client::{ClientBuilder, Result};

#[tokio::main]
async fn main() -> Result<()> {

    let ads_client = ClientBuilder::new("5.80.201.232.1.1", 10000).build().await?;

    match ads_client.read_state().await {
        Ok(state) => println!("State: {:?}", state),
        Err(err) => println!("Error: {}", err.to_string())
    }
    Ok(())
}