use ads_client::ClientBuilder;
use tokio::runtime::Runtime;

fn main() {
    let rt = Runtime::new().unwrap();
    let ads_client = rt.block_on(ClientBuilder::new("5.80.201.232.1.1", 10000).build()).unwrap();

    match rt.block_on(ads_client.read_device_info()) {
        Ok(device_info) => {
            println!("DeviceInfo: TwinCAT {}.{}.{} , Device Name: {}", 
                device_info.major, 
                device_info.minor,
                device_info.build,
                device_info.device_name)
        }
        Err(err) => println!("Error: {}", err.to_string())
    }
}