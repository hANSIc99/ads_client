use ads_client::{Client, AdsTimeout, Result};

#[tokio::main]
async fn main() -> Result<()> {

    let ads_client = Client::new("5.80.201.232.1.1", 10000, AdsTimeout::DefaultTimeout).await?;

    match ads_client.read_state().await {
        Ok(state) => println!("State: {:?}", state),
        Err(err) => println!("Error: {}", err.to_string())
    }
    Ok(())
}