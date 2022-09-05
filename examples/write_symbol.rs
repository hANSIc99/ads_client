use ads_client::{Client, AdsTimeout};
use tokio::runtime::Runtime;

fn main() {
    let rt = Runtime::new().unwrap();
    let ads_client = rt.block_on(Client::new("5.80.201.232.1.1", 851, AdsTimeout::DefaultTimeout)).unwrap();

    // Get symbol handle
    let mut hdl : [u8; 4] = [0; 4];
    let symbol = b"MAIN.n_cnt_a";

    if let Err(err) = rt.block_on(ads_client.read_write(0xF003, 0, &mut hdl, symbol)){
        println!("Error: {}", err.to_string());
    }

    let n_hdl = u32::from_ne_bytes(hdl.try_into().unwrap());

    if n_hdl != 0 {
        println!("Got handle!");
        
        let n_cnt_a : u16 = 1000;
        
        match rt.block_on(ads_client.write(0xF005, n_hdl, &n_cnt_a.to_ne_bytes())){
            Ok(_)     => println!("Variable successfully written!"),
            Err(err) => println!("Error: {}", err.to_string())
        }
    }
}