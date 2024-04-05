use std::thread;
use std::time::Duration;
use ads_client::{Client, AdsTimeout, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let ads_client = Client::new("5.80.201.232.1.1", 851, AdsTimeout::DefaultTimeout).await?;

    // Get symbol handle
    let mut hdl : [u8; 4] = [0; 4];
    let symbol = b"MAIN.n_cnt_a";

    if let Err(err) = ads_client.read_write(0xF003, 0, &mut hdl, symbol).await{
        println!("Error: {}", err.to_string());
    }

    let n_hdl = u32::from_ne_bytes(hdl.try_into().unwrap());

    if n_hdl != 0 {
        println!("Got handle!");

        let mut plc_n_cnt_a : [u8; 2] = [0; 2];
        
        loop {
            let read_hdl = ads_client.read(0xF005, n_hdl, &mut plc_n_cnt_a).await;

            match read_hdl {
                Ok(_bytes_read)     => {
                    let n_cnt_a = u16::from_ne_bytes(plc_n_cnt_a.try_into().unwrap());
                    println!("MAIN.n_cnt_a: {}", n_cnt_a);
                },
                Err(err) => println!("Read failed: {}", err.to_string())
            }
    
            thread::sleep(Duration::from_millis(1000));
     
        }

    }
    Ok(())
}