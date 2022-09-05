use std::sync::{Arc, atomic::Ordering};
use bytes::Bytes;
use crate::{AdsError, Client, Result, AdsCommand, CommandManager, AdsStateInfo};

impl Client {

    fn post_read_state(rs_response : Bytes) -> Result<AdsStateInfo> {

        if rs_response.len() != 8 {
            return Err(Box::new(AdsError{n_error : 0xE })); // Invalid length
        } else {

            Client::eval_return_code(&rs_response.slice(0..4))?;

            Ok(AdsStateInfo{
                ads_state       : u16::from_ne_bytes(rs_response.slice(4..6)[..].try_into().unwrap()),
                device_state    : u16::from_ne_bytes(rs_response.slice(6..8)[..].try_into().unwrap())
            })
        }
    }

    /// Submit an [ADS Read State](https://infosys.beckhoff.com/content/1033/tc3_ads_intro/115878923.html) request synchronously
    /// 
    /// 
    /// # Example
    /// Basic usage
    /// ```rust
    /// test
    /// ```
    pub async fn read_state(&self) -> Result<AdsStateInfo> {

        // Prepare read state request
        let invoke_id : u32 = u32::from(self.hdl_cnt.fetch_add(1, Ordering::SeqCst));
        let ams_header = self.c_init_ams_header(invoke_id, None, AdsCommand::ReadState);
        
        // Create handle
        self.register_command_handle(invoke_id, AdsCommand::ReadState);

        // Launch the CommandManager future
        let a_handles = Arc::clone(&self.handles);
        let cmd_man_future = CommandManager::new(self.timeout, invoke_id, a_handles);

        // Launch socket future
        let socket_future = self.socket_write(&ams_header);

        tokio::try_join!(cmd_man_future, socket_future).and_then( | (rs_response, _) | Client::post_read_state(rs_response))
    }
}