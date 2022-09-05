use std::sync::{Arc, atomic::Ordering};
use bytes::{Bytes, BytesMut};
use crate::{Client, Result, AdsCommand, CommandManager, HEADER_SIZE, LEN_READ_REQ};

impl Client {
    //! ADS Read command
    //! 
    //! Test test 123

    fn pre_read(&self, idx_grp: u32, idx_offs: u32, rd_len : usize, invoke_id: u32) -> Bytes {

        let ams_header = self.c_init_ams_header(invoke_id, Some(LEN_READ_REQ as u32), AdsCommand::Read);
        let mut read_header : [u8; LEN_READ_REQ] = [0; LEN_READ_REQ];

        let read_length = rd_len as u32;

        read_header[0..4].copy_from_slice(&idx_grp.to_ne_bytes());
        read_header[4..8].copy_from_slice(&idx_offs.to_ne_bytes());
        read_header[8..12].copy_from_slice(&read_length.to_ne_bytes());

        // Assemble read request: Create two iterators and chain them
        let iter_ams_header = ams_header.into_iter();
        let iter_read = read_header.into_iter();

        let mut _read_request = BytesMut::with_capacity(HEADER_SIZE + LEN_READ_REQ);
        _read_request = iter_ams_header.chain(iter_read).collect();

        _read_request.freeze()
    }

    fn post_read(read_response : Bytes, data: &mut [u8]) -> Result<u32> {

        Client::eval_return_code(read_response.as_ref())?;

        // Copy payload to destination argument
        // Payload starts at offset 8
        let iter_payload = read_response[8..].into_iter();
        let iter_read_data = data.iter_mut();

        // Zip payload and destination together
        //let iter_data = zip(iter_read_data, iter_payload); // BAUSTELLEX
        let iter_data = iter_read_data.zip(iter_payload);
    
        // Iterate till the first iterator is exhausted
        for data in iter_data {
            let (rd, pl) = data;
            *rd = *pl; // Copy from response to data
        }

        Ok(read_response[8..].len() as u32)
    }

    pub async fn read(&self, idx_grp: u32, idx_offs: u32, data: &mut [u8]) -> Result<u32> {
        
        // Preprocessing
        let invoke_id : u32 = u32::from(self.hdl_cnt.fetch_add(1, Ordering::SeqCst));
        let _read_req = self.pre_read(idx_grp, idx_offs, data.len(), invoke_id);

        // Create handle
        self.register_command_handle(invoke_id, AdsCommand::Read);

        // Launch the CommandManager future
        let a_handles = Arc::clone(&self.handles);
        let cmd_man_future = CommandManager::new(self.timeout, invoke_id, a_handles);

        // Launch socket future
        let socket_future = self.socket_write(&_read_req);

        // https://docs.rs/tokio/latest/tokio/macro.try_join.html
        // INFO https://stackoverflow.com/questions/69031447/tokiotry-join-doesnt-return-the-err-variant-when-one-of-the-tasks-returns-er

        tokio::try_join!(cmd_man_future, socket_future).and_then(| (rd_response, _) | Client::post_read(rd_response, data))
    }
}