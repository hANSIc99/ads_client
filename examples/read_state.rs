use ads_client::ClientBuilder;
use tokio::runtime::Runtime;

fn main() {
    let rt = Runtime::new().unwrap();
    let ads_client = rt.block_on(ClientBuilder::new("5.80.201.232.1.1", 10000).build()).unwrap();

    match rt.block_on(ads_client.read_state()) {
        Ok(state) => println!("State: {:?}", state),
        Err(err) => println!("Error: {}", err.to_string())
    }
}