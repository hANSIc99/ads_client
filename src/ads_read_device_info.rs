use bytes::Bytes;
use crate::{AdsError, Client, Result, AdsCommand, DeviceStateInfo};

impl Client {

    fn post_read_device_info(rd_dinfo_response : Bytes) -> Result<DeviceStateInfo> {

        if rd_dinfo_response.len() != 24 {
            return Err(AdsError{n_error : 0xE, s_msg : String::from("Invalid AMS length") });
        } else {

            Client::eval_return_code(&rd_dinfo_response.slice(0..4))?;

            Ok(DeviceStateInfo{
                major       : u8::from_ne_bytes(rd_dinfo_response.slice(4..5)[..].try_into().unwrap()),
                minor       : u8::from_ne_bytes(rd_dinfo_response.slice(5..6)[..].try_into().unwrap()),
                build       : u16::from_ne_bytes(rd_dinfo_response.slice(6..8)[..].try_into().unwrap())
            })
        }
    }

    /// Submit an asynchronous [ADS Read Device Info](https://infosys.beckhoff.com/content/1031/tc3_ads_intro/115875851.html?id=308075010936482438) request.
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
    ///    match ads_client.read_device_info().await {
    ///        Ok(device_info) => println!("DeviceInfo: {:?}", device_info),
    ///        Err(err) => println!("Error: {}", err.to_string())
    ///    }
    ///    Ok(())
    ///}
    /// ```
    /// Checkout the examples [read_state](https://github.com/hANSIc99/ads_client/blob/main/examples/read_device_info.rs) 
    /// and [read_state_async](https://github.com/hANSIc99/ads_client/blob/main/examples/read_device_info_async.rs).
    pub async fn read_device_info(&self) -> Result<DeviceStateInfo> {
        // Prepare read device info request
        let invoke_id = self.create_invoke_id();
        let ams_header = self.c_init_ams_header(invoke_id, None, AdsCommand::ReadDeviceInfo);

        // Create handle
        self.register_command_handle(invoke_id, AdsCommand::ReadDeviceInfo);

        // Launch the CommandManager future
        let cmd_man_future = self.create_cmd_man_future(invoke_id);

        // Launch socket future
        let socket_future = self.socket_write(&ams_header);

        tokio::try_join!(cmd_man_future, socket_future).and_then( | (rs_response, _) | Client::post_read_device_info(rs_response))
    }

}