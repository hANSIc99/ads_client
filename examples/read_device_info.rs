use ads_client::{Client, AdsTimeout};
use tokio::runtime::Runtime;

fn main() {
    let rt = Runtime::new().unwrap();
    let ads_client = rt.block_on(Client::new("5.80.201.232.1.1", 10000, AdsTimeout::DefaultTimeout)).unwrap();

    match rt.block_on(ads_client.read_device_info()) {
        Ok(device_info) => println!("DeviceInfo: {:?}", device_info),
        Err(err) => println!("Error: {}", err.to_string())
    }
}