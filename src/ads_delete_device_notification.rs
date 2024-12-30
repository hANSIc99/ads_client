use bytes::{Bytes, BytesMut};
use log::info;
use crate::{Client, AdsCommand, AdsError, AdsErrorCode, HEADER_SIZE, LEN_DEL_DEV_NOT, Result, misc::HandleData};

impl Client {

    fn pre_delete_device_notification(&self, handle : u32, invoke_id : u32) -> Bytes {
        let ams_header = self.c_init_ams_header(invoke_id, Some(LEN_DEL_DEV_NOT as u32), AdsCommand::DeleteDeviceNotification);

        let del_not_header  : [u8; LEN_DEL_DEV_NOT] = [
            u32_lw_lb!(handle),
            u32_lw_hb!(handle),
            u32_hw_lb!(handle),
            u32_hw_hb!(handle), 
        ];

        let iter_ams_header = ams_header.into_iter();
        let iter_del_not    = del_not_header.into_iter();

        let mut _del_not_req = BytesMut::with_capacity(HEADER_SIZE + LEN_DEL_DEV_NOT);
        _del_not_req = iter_ams_header.chain(iter_del_not).collect();

        _del_not_req.freeze()
    }

    fn post_delete_device_notification(del_not_response : HandleData) -> Result<()>{
        Client::eval_ams_error(del_not_response.ams_err)?;

        del_not_response.payload
            .map(|p| Client::eval_return_code(p.as_ref()))
            .ok_or_else(|| AdsError{n_error : AdsErrorCode::ADSERR_DEVICE_INVALIDDATA.into(), s_msg : String::from("Invalid data values")})??;

        Ok(())
    }

    /// Submit an asynchronous [ADS Delete Device Notification](https://infosys.beckhoff.com/content/1033/tc3_ads_intro/115881995.html?id=6216061301016726131) request.
    /// 
    /// Checkout the extensive examples [notification](https://github.com/hANSIc99/ads_client/blob/main/examples/notification.rs) 
    /// and [notification_async](https://github.com/hANSIc99/ads_client/blob/main/examples/notification_async.rs).
    pub async fn delete_device_notification(&self, handle: u32 ) -> Result<()>{
        // Prepare delete device notification request
        let invoke_id = self.create_invoke_id();
        let _del_not_req = self.pre_delete_device_notification(handle, invoke_id);

        info!("Submit Delete Notification Request: Invoke ID: {}", invoke_id);

        // Create handle for request
        self.register_command_handle(invoke_id, AdsCommand::DeleteDeviceNotification);

        // Launch the CommandManager future
        let cmd_man_future = self.create_cmd_man_future(invoke_id);

        // Launch socket future
        let socket_future = self.socket_write(&_del_not_req);

        tokio::try_join!(cmd_man_future, socket_future).and_then(| (del_not_response, _)| {
            Client::post_delete_device_notification(del_not_response)
        }) 
    }
}