use ads_client::{Client, AdsTimeout, Result};

#[tokio::main]
async fn main() -> Result<()> {

    let ads_client = Client::new("5.80.201.232.1.1", 10000, AdsTimeout::DefaultTimeout).await?;
    
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