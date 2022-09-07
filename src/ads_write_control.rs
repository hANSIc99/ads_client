use std::sync::{Arc, atomic::Ordering};
use bytes::{Bytes, BytesMut};
use crate::{Client, Result, AdsCommand, CommandManager, StateInfo, HEADER_SIZE, LEN_WR_CTRL_MIN};

impl Client {

    fn pre_write_ctrl(&self, state : &StateInfo, data : Option<&[u8]>, invoke_id : u32) -> Bytes {
        let mut data_length : u32 = 0;
        
        if let Some(v) = data {
            data_length = v.len() as u32;
        }
        
        let ams_header = self.c_init_ams_header(invoke_id, Some(LEN_WR_CTRL_MIN as u32 + data_length), AdsCommand::WriteControl);
        let mut wr_ctrl_header : [u8; LEN_WR_CTRL_MIN] = [0; LEN_WR_CTRL_MIN];

        wr_ctrl_header[0..2].copy_from_slice(&(state.ads_state as u16).to_ne_bytes());
        wr_ctrl_header[2..4].copy_from_slice(&state.device_state.to_ne_bytes());
        wr_ctrl_header[4..8].copy_from_slice(&data_length.to_ne_bytes());

        let iter_ams_header = ams_header.into_iter();
        let iter_wrt_ctrl   = wr_ctrl_header.into_iter();

        let mut _wr_ctrl_request = BytesMut::with_capacity(HEADER_SIZE + LEN_WR_CTRL_MIN + data_length as usize);
        
        match data {
            Some(data) => _wr_ctrl_request = iter_ams_header.chain(iter_wrt_ctrl.chain(data.iter().cloned())).collect(),
            None => _wr_ctrl_request = iter_ams_header.chain(iter_wrt_ctrl).collect()
        }
        
        _wr_ctrl_request.freeze()
    }

    fn post_write_ctrl(wr_ctrl_response : Bytes) -> Result<()>{
        Client::eval_return_code(wr_ctrl_response.as_ref())?;
        Ok(())
    }

    pub async fn write_control(&self, state : &StateInfo, data: Option<&[u8]>) -> Result<()>{
        // Prepare write control request
        let invoke_id : u32 = u32::from(self.hdl_cnt.fetch_add(1, Ordering::SeqCst));
        let _wr_ctr_request = self.pre_write_ctrl(state, data, invoke_id);

        // Create handle
        self.register_command_handle(invoke_id, AdsCommand::WriteControl);

        // Launch the CommandManager future
        let a_handles = Arc::clone(&self.handles);
        let cmd_man_future = CommandManager::new(self.timeout, invoke_id, a_handles);

        // Launch socket future
        let socket_future = self.socket_write(&_wr_ctr_request);

        tokio::try_join!(cmd_man_future, socket_future).and_then( | (wr_ctr_response, _) | Client::post_write_ctrl(wr_ctr_response))
    }
}