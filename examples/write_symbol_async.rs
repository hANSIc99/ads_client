use ads_client::{ClientBuilder, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let ads_client = ClientBuilder::new("5.80.201.232.1.1", 851).build().await?;

    // Get symbol handle
    let mut hdl : [u8; 4] = [0; 4];
    let symbol = b"MAIN.n_cnt_a";

    if let Err(err) = ads_client.read_write(0xF003, 0, &mut hdl, symbol).await{
        println!("Error: {}", err.to_string());
    }

    let n_hdl = u32::from_ne_bytes(hdl.try_into().unwrap());

    if n_hdl != 0 {
        println!("Got handle!");
        
        let n_cnt_a : u16 = 1000;
        
        match ads_client.write(0xF005, n_hdl, &n_cnt_a.to_ne_bytes()).await{
            Ok(_)     => println!("Variable successfully written!"),
            Err(err) => println!("Error: {}", err.to_string())
        }
    }
    Ok(())
}