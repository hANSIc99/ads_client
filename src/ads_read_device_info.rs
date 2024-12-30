use bytes::Buf;
use log::info;
use std::io::Read;
use crate::{AdsError, AdsErrorCode, Client, Result, AdsCommand, DeviceStateInfo, misc::HandleData};

impl Client {

    fn post_read_device_info(rd_dinfo_response : HandleData) -> Result<DeviceStateInfo> {

        let payload = rd_dinfo_response.payload
                        .ok_or_else(|| AdsError{n_error : AdsErrorCode::ADSERR_DEVICE_INVALIDDATA.into(), s_msg : String::from("Invalid data values.")})?;

        Client::eval_ams_error(rd_dinfo_response.ams_err)?;

        if payload.len() != 24 {
            return Err(AdsError{n_error : 0xE, s_msg : String::from("Invalid AMS length") });
        } else {

            Client::eval_return_code(&payload.slice(0..4))?;

            let mut s_device_name = String::new();
            payload.slice(8..24)[..].reader().read_to_string(&mut s_device_name)?;

            Ok(DeviceStateInfo{
                major       : u8::from_ne_bytes(payload.slice(4..5)[..].try_into().unwrap_or_default()),
                minor       : u8::from_ne_bytes(payload.slice(5..6)[..].try_into().unwrap_or_default()),
                build       : u16::from_ne_bytes(payload.slice(6..8)[..].try_into().unwrap_or_default()),
                device_name : s_device_name
            })
        }
    }

    /// Submit an asynchronous [ADS Read Device Info](https://infosys.beckhoff.com/content/1031/tc3_ads_intro/115875851.html?id=308075010936482438) request.
    /// 
    /// 
    /// # Example
    ///
    /// ```rust 
    ///use ads_client::{Client, AdsTimeout, Result};
    ///
    ///#[tokio::main]
    ///async fn main() -> Result<()> {
    ///
    ///    let ads_client = Client::new("5.80.201.232.1.1", 10000, AdsTimeout::DefaultTimeout).await?;
    ///    
    ///    match ads_client.read_device_info().await {
    ///        Ok(device_info) => {
    ///            println!("DeviceInfo: TwinCAT {}.{}.{} , Device Name: {}", 
    ///                device_info.major, 
    ///                device_info.minor,
    ///                device_info.build,
    ///                device_info.device_name)
    ///        }
    ///        Err(err) => println!("Error: {}", err.to_string())
    ///    }
    ///    Ok(())
    ///}
    /// ```
    /// Checkout the examples [read_device_info](https://github.com/hANSIc99/ads_client/blob/main/examples/read_device_info.rs) 
    /// and [read_device_info_async](https://github.com/hANSIc99/ads_client/blob/main/examples/read_device_info_async.rs).
    pub async fn read_device_info(&self) -> Result<DeviceStateInfo> {
        // Prepare read device info request
        let invoke_id = self.create_invoke_id();
        let ams_header = self.c_init_ams_header(invoke_id, None, AdsCommand::ReadDeviceInfo);

        info!("Submit Read Device Info: Invoke ID: {}", invoke_id);

        // Create handle
        self.register_command_handle(invoke_id, AdsCommand::ReadDeviceInfo);

        // Launch the CommandManager future
        let cmd_man_future = self.create_cmd_man_future(invoke_id);

        // Launch socket future
        let socket_future = self.socket_write(&ams_header);

        tokio::try_join!(cmd_man_future, socket_future).and_then( | (rs_response, _) | Client::post_read_device_info(rs_response))
    }

}