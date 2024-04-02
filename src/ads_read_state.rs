use crate::{AdsError, AdsErrorCode, Client, Result, AdsCommand, StateInfo, HandleData};

impl Client {

    fn post_read_state(rs_response : HandleData) -> Result<StateInfo> {

        let payload = rs_response.payload
                    .ok_or_else(|| AdsError{n_error : AdsErrorCode::ADSERR_DEVICE_INVALIDDATA.into(), s_msg : String::from("Invalid data values.")})?;
        //let mut b_respone : [u8; 4] = payload.slice(0..4)[..].try_into().unwrap(); // TODO: Debug only
        
        Client::eval_ams_error(rs_response.ams_err)?;
        Client::eval_return_code(&payload.slice(0..4))?;

         if payload.len() != 8 {
            return Err(AdsError{n_error : AdsErrorCode::ERR_INVALIDAMSLENGTH.into(), s_msg : String::from("Invalid AMS length") });
        } else {

            Client::eval_return_code(&payload.slice(0..4))?;

            let stateInfo = StateInfo{
                ads_state       : u16::from_ne_bytes(payload.slice(4..6)[..].try_into().unwrap_or_default()).try_into()?,
                device_state    : u16::from_ne_bytes(payload.slice(6..8)[..].try_into().unwrap_or_default())
            };


            if (stateInfo == StateInfo::default()){
                return Err(AdsError{n_error : AdsErrorCode::ERR_INTERNAL.into(), s_msg : String::from("Internal error - conversion of payload failed.")});
            }

            Ok(stateInfo)
        }

    }

    /// Submit an asynchronous [ADS Read State](https://infosys.beckhoff.com/content/1033/tc3_ads_intro/115878923.html) request.
    /// 
    /// 
    /// # Example
    ///
    /// ```rust
    /// use ads_client::{Client, AdsTimeout, Result};
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///
    ///    let ads_client = Client::new("5.80.201.232.1.1", 10000, AdsTimeout::DefaultTimeout).await?;
    ///
    ///    match ads_client.read_state().await {
    ///        Ok(state) => println!("State: {:?}", state),
    ///        Err(err) => println!("Error: {}", err.to_string())
    ///    }
    ///    Ok(())
    ///}
    /// ```
    /// Checkout the examples [read_state](https://github.com/hANSIc99/ads_client/blob/main/examples/read_state.rs) 
    /// and [read_state_async](https://github.com/hANSIc99/ads_client/blob/main/examples/read_state_async.rs).
    pub async fn read_state(&self) -> Result<StateInfo> {
        // Prepare read state request
        let invoke_id = self.create_invoke_id();
        let ams_header = self.c_init_ams_header(invoke_id, None, AdsCommand::ReadState);
        
        // Create handle
        self.register_command_handle(invoke_id, AdsCommand::ReadState);

        // Launch the CommandManager future
        let cmd_man_future = self.create_cmd_man_future(invoke_id);

        // Launch socket future
        let socket_future = self.socket_write(&ams_header);

        tokio::try_join!(cmd_man_future, socket_future).and_then( | (rs_response, _) | Client::post_read_state(rs_response))
    }
}