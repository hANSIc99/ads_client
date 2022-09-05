use std::sync::{Arc, atomic::Ordering};
use bytes::{Bytes, BytesMut};
use crate::{Client, Result, AdsCommand, CommandManager, HEADER_SIZE, LEN_RW_REQ_MIN};

impl Client{

    fn pre_read_write(&self,  idx_grp: u32, idx_offs: u32, read_data: &mut [u8], write_data: &[u8], invoke_id: u32) -> Bytes {

        let read_length     = read_data.len() as u32;
        let write_length    = write_data.len() as u32;
        let ams_header = self.c_init_ams_header(invoke_id, Some(LEN_RW_REQ_MIN as u32 + write_length), AdsCommand::ReadWrite);

        let mut rw_header : [u8; LEN_RW_REQ_MIN] = [0; LEN_RW_REQ_MIN];

        rw_header[0..4].copy_from_slice(&idx_grp.to_ne_bytes());
        rw_header[4..8].copy_from_slice(&idx_offs.to_ne_bytes());
        rw_header[8..12].copy_from_slice(&read_length.to_ne_bytes());
        rw_header[12..16].copy_from_slice(&write_length.to_ne_bytes());

        // Assemble ReadWrite request: Create two iterators and chain them
        let iter_ams_header     = ams_header.into_iter();
        let iter_rw_cmd         = rw_header.into_iter();
        let iter_wr_data        = write_data.iter().cloned();

        let mut _rw_request = BytesMut::with_capacity(HEADER_SIZE + LEN_RW_REQ_MIN + write_length as usize);
        _rw_request = iter_ams_header.chain(iter_rw_cmd.chain(iter_wr_data)).collect();

        _rw_request.freeze()
    }

    fn post_read_write(rw_response : Bytes, read_data: &mut [u8]) -> Result<u32> {

        Client::eval_return_code(rw_response.as_ref())?;

        // Copy payload to destination buffer
        // Payload starts at offset 8
        let iter_payload = rw_response[8..].into_iter();
        let iter_read_data = read_data.iter_mut();
    
        let iter_data = iter_read_data.zip(iter_payload);
    
        // Iterate till the first iterator is exhausted
        for data in iter_data {
            let (rd, pl) = data;
            *rd = *pl; // Copy from response to read data
        }

        Ok(rw_response[8..].len() as u32)
    }

    pub async fn read_write(&self, idx_grp: u32, idx_offs: u32, read_data: &mut [u8], write_data: &[u8]) -> Result<u32>{
        
        // Prepare ReadWrite request
        let invoke_id : u32 = u32::from(self.hdl_cnt.fetch_add(1, Ordering::SeqCst));
        let _rw_request = self.pre_read_write(idx_grp, idx_offs, read_data, write_data, invoke_id);

        // Create handle
        self.register_command_handle(invoke_id, AdsCommand::ReadWrite);

        // Launch CommandManager future
        let a_handles = Arc::clone(&self.handles);
        let cmd_man_future = CommandManager::new(self.timeout, invoke_id, a_handles);
    
        // Launch socket future
        let socket_future = self.socket_write(&_rw_request);
        
        tokio::try_join!(cmd_man_future, socket_future).and_then(| (rw_response, _) | Client::post_read_write(rw_response, read_data))
    }
}
