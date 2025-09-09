use ads_client::{ClientBuilder, Result};

#[tokio::main]
async fn main() -> Result<()> {

    let ads_client = ClientBuilder::new("5.80.201.232.1.1", 10000).build().await?;
    
    match ads_client.read_device_info().await {
        Ok(device_info) => {
            println!("DeviceInfo: TwinCAT {}.{}.{} , Device Name: {}", 
                device_info.major, 
                device_info.minor,
                device_info.build,
                device_info.device_name)
        }
        Err(err) => println!("Error: {}", err.to_string())
    }
    Ok(())
}