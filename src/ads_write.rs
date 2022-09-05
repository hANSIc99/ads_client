use std::sync::{Arc, atomic::Ordering};
use bytes::{Bytes, BytesMut};
use crate::{Client, Result, AdsCommand, CommandManager, HEADER_SIZE, LEN_W_REQ_MIN};

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

    fn post_write(w_response : Bytes) -> Result<()> {
        Client::eval_return_code(w_response.as_ref())?;
        Ok(())
    }

    pub async fn write(&self, idx_grp: u32, idx_offs: u32, data: &[u8]) -> Result<()> {

        // Prepare write request
        let invoke_id : u32 = u32::from(self.hdl_cnt.fetch_add(1, Ordering::SeqCst));
        let _w_request = self.pre_write(idx_grp, idx_offs, data, invoke_id);

        // Create handle
        self.register_command_handle(invoke_id, AdsCommand::Write);

        // Launch the CommandManager future
        let a_handles = Arc::clone(&self.handles);
        let cmd_man_future = CommandManager::new(self.timeout, invoke_id, a_handles);
    
        // Launch socket future
        let socket_future = self.socket_write(&_w_request);

        tokio::try_join!(cmd_man_future, socket_future).and_then(| (w_response, _) | Client::post_write(w_response))
    }
}