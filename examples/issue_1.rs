use ads_client::{Client, AdsTimeout, Result};
use byteorder::{ByteOrder, LittleEndian, LE};

pub const SYM_DT_UPLOAD: u32 = 0xF00E;
pub const SYM_UPLOAD_INFO2: u32 = 0xF00F;
pub const SYM_UPLOAD: u32 = 0xF00B;


#[tokio::main]
async fn main() -> Result<()> {

    


    let ads_client = Client::new("5.80.201.232.1.1", 851, AdsTimeout::CustomTimeout(10)).await?;
    let mut read_data: [u8; 48] = [0; 48];

    if let Err(err) = ads_client.read(0xF00F, 0, &mut read_data).await {
        println!("Error: {}", err);
    }

    let n_symbols = LE::read_u32(&read_data[0..]) as usize;
    let symbol_len = LE::read_u32(&read_data[4..]) as usize;
    let n_types = LE::read_u32(&read_data[8..]) as usize;
    let types_len = LE::read_u32(&read_data[12..]) as usize;
    
    let mut symbol_data: Vec<u8> = Vec::with_capacity(symbol_len);
    symbol_data.resize(symbol_len, 0);

    let mut types_data: Vec<u8> = Vec::with_capacity(types_len);
    types_data.resize(types_len, 0);



    if let Err(err) = ads_client.read(0xF00B, 0, &mut symbol_data).await {
        println!("Error: {}", err);
    };

    if let Err(err) = ads_client.read(0xF00E, 0, &mut types_data).await {
        println!("Error: {}", err);
    };
    println!("n_symbols: {}", n_symbols);
    println!("n_types: {}", n_types);
    println!("issue_1");
    Ok(())
}