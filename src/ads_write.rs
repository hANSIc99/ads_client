use bytes::{Bytes, BytesMut};
use log::info;
use crate::{Client, Result, AdsCommand, AdsError, AdsErrorCode, HEADER_SIZE, LEN_W_REQ_MIN, misc::HandleData};

impl Client {

    fn pre_write(&self, idx_grp: u32, idx_offs: u32, data: &[u8], invoke_id : u32) -> Bytes {
        let write_length = data.len() as u32;

        let ams_header = self.c_init_ams_header(invoke_id, Some(LEN_W_REQ_MIN as u32 + write_length), AdsCommand::Write);
        let mut w_header : [u8; LEN_W_REQ_MIN] = [0; LEN_W_REQ_MIN];

        w_header[0..4].copy_from_slice(&idx_grp.to_ne_bytes());
        w_header[4..8].copy_from_slice(&idx_offs.to_ne_bytes());
        w_header[8..12].copy_from_slice(&write_length.to_ne_bytes());

        // Assemble read request: Create two iterators and chain them
        let iter_ams_header     = ams_header.into_iter();
        let iter_write_cmd      = w_header.into_iter();
        let iter_wr_data        = data.iter().cloned();

        let mut _w_request = BytesMut::with_capacity(HEADER_SIZE + LEN_W_REQ_MIN + write_length as usize);
        _w_request = iter_ams_header.chain(iter_write_cmd.chain(iter_wr_data)).collect();

        _w_request.freeze()
    }

    fn post_write(w_response : HandleData) -> Result<()> {

        Client::eval_ams_error(w_response.ams_err)?;

        w_response.payload
                    .map(|p| Client::eval_return_code(p.as_ref()))
                    .ok_or_else(|| AdsError{n_error : AdsErrorCode::ADSERR_DEVICE_INVALIDDATA.into(), s_msg : String::from("Invalid data values")})??;
        Ok(())
    }
    /// Submit an asynchronous [ADS Write](https://infosys.beckhoff.com/content/1033/tc3_ads_intro/115877899.html) request.
    /// 
    /// # Example
    /// 
    /// ```rust
    ///use ads_client::{ClientBuilder, Result};
    ///
    ///#[tokio::main]
    ///async fn main() -> Result<()> {
    ///    let ads_client = ClientBuilder::new("5.80.201.232.1.1", 851).build().await?;
    ///
    ///    // Get symbol handle
    ///    let mut hdl : [u8; 4] = [0; 4];
    ///    let symbol = b"MAIN.n_cnt_a";
    ///
    ///    if let Err(err) = ads_client.read_write(0xF003, 0, &mut hdl, symbol).await{
    ///        println!("Error: {}", err.to_string());
    ///    }
    ///
    ///    let n_hdl = u32::from_ne_bytes(hdl.try_into().unwrap());
    ///
    ///    if n_hdl != 0 {
    ///        println!("Got handle!");
    ///        
    ///        let n_cnt_a : u16 = 1000;
    ///        
    ///        match ads_client.write(0xF005, n_hdl, &n_cnt_a.to_ne_bytes()).await{
    ///            Ok(_)     => println!("Variable successfully written!"),
    ///            Err(err) => println!("Error: {}", err.to_string())
    ///        }
    ///    }
    ///    Ok(())
    ///}
    /// ```
    /// Checkout the examples [write_symbol](https://github.com/hANSIc99/ads_client/blob/main/examples/write_symbol.rs) 
    /// and [write_symbol_async](https://github.com/hANSIc99/ads_client/blob/main/examples/write_symbol_async.rs).
    pub async fn write(&self, idx_grp: u32, idx_offs: u32, data: &[u8]) -> Result<()> {
        // Prepare write request
        let invoke_id = self.create_invoke_id();
        let _w_request = self.pre_write(idx_grp, idx_offs, data, invoke_id);

        info!("Submit Write Request: Invoke ID: {}, Write length: {}", invoke_id, data.len());

        // Create handle
        self.register_command_handle(invoke_id, AdsCommand::Write);

        // Launch the CommandManager future
        let cmd_man_future = self.create_cmd_man_future(invoke_id);
    
        // Launch socket future
        let socket_future = self.socket_write(&_w_request);

        tokio::try_join!(cmd_man_future, socket_future).and_then(| (w_response, _) | Client::post_write(w_response))
    }
}