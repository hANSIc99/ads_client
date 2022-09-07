use std::sync::{Arc, Mutex};
use bytes::{Bytes, BytesMut};
use crate::{AdsError, Client, AdsCommand, Notification, AdsNotificationAttrib, HEADER_SIZE, LEN_ADD_DEV_NOT, Result};

impl Client {

    fn pre_add_dev_not(&self, idx_grp: u32, idx_offs: u32, attributes : &AdsNotificationAttrib, invoke_id : u32) -> Bytes {
        let ams_header = self.c_init_ams_header(invoke_id, Some(LEN_ADD_DEV_NOT as u32), AdsCommand::AddDeviceNotification);
        let mut add_not_header : [u8; LEN_ADD_DEV_NOT] = [0; LEN_ADD_DEV_NOT];

        // Prepare AddDeviceNotificationRequest
        add_not_header[0..4].copy_from_slice(&idx_grp.to_ne_bytes());
        add_not_header[4..8].copy_from_slice(&idx_offs.to_ne_bytes());
        add_not_header[8..12].copy_from_slice(&attributes.cb_length.to_ne_bytes());
        add_not_header[12..16].copy_from_slice(&(attributes.trans_mode as u32).to_ne_bytes());
        add_not_header[16..20].copy_from_slice(&attributes.max_delay.to_ne_bytes());
        add_not_header[20..24].copy_from_slice(&attributes.cycle_time.to_ne_bytes());

        let iter_ams_header = ams_header.into_iter();
        let iter_add_not    = add_not_header.into_iter();

        let mut _add_not_req = BytesMut::with_capacity(HEADER_SIZE + LEN_ADD_DEV_NOT);
        _add_not_req = iter_ams_header.chain(iter_add_not).collect();

        _add_not_req.freeze()
    }

    fn post_add_dev_not(&self, add_dev_not_response : Bytes, handle: &mut u32, callback : Notification, user_data: Option<&Arc<Mutex<BytesMut>>>) -> Result<()>{
        Client::eval_return_code(add_dev_not_response.as_ref())?;

        *handle = u32::from_ne_bytes(add_dev_not_response[4..8].try_into().map_err(|_| AdsError{n_error : 1})?);

        // Check if registration of device notification was successfull
        if *handle != 0 {
            // Register notification handle
            self.register_not_handle(*handle, callback, user_data);
        }

        Ok(())
    }
    /// Submit an asynchronous [ADS Add Device Notification](https://infosys.beckhoff.com/content/1033/tc3_ads_intro/115880971.html?id=7388557527878561663) request.
    /// 
    /// Checkout the extensive examples [notification](https://github.com/hANSIc99/ads_client/blob/main/examples/notification.rs) 
    /// and [notification_async](https://github.com/hANSIc99/ads_client/blob/main/examples/notification_async.rs).
    pub async fn add_device_notification(&self, idx_grp: u32, idx_offs: u32, attributes : &AdsNotificationAttrib, handle: &mut u32, callback : Notification, user_data: Option<&Arc<Mutex<BytesMut>>> ) -> Result<()>{
        // Prepare AddDeviceNotification request
        let invoke_id = self.create_invoke_id();
        let _add_not_req = self.pre_add_dev_not(idx_grp, idx_offs, attributes, invoke_id);

        // Create handle for request
        self.register_command_handle(invoke_id, AdsCommand::AddDeviceNotification);

        // Launch CommandManager future
        let cmd_man_future = self.create_cmd_man_future(invoke_id);
        
        // Launch socket future
        let socket_future = self.socket_write(&_add_not_req);

        tokio::try_join!(cmd_man_future, socket_future).and_then( | (add_not_response, _) | {
            self.post_add_dev_not(add_not_response, handle, callback, user_data)
        })
    }
}