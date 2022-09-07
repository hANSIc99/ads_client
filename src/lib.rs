//! Welcome to the ADS client library.
//! 
//! This library can be used for communication over the [Beckhoff ADS](https://infosys.beckhoff.com/content/1033/tcinfosys3/11291871243.html) protocoll.
//! 
//! The ADS client is used to work beside a 
//! [TC1000 ADS router](https://www.beckhoff.com/en-en/products/automation/twincat/tc1xxx-twincat-3-base/tc1000.html)
//! which is part of every TwinCAT installation. The client requires at least TwinCAT Version 3.1.4024.x.
//! 
//! The methods are implemented asynchronous and non-blocking based on the [tokio](https://tokio.rs/) runtime.
//! 
//! # Usage
//! 
//! Checkout the [example section](https://github.com/hANSIc99/ads_client/tree/main/examples) in the repsoitory.

#[macro_use]
mod misc;
mod command_manager;
mod command_cleaner;
mod ads_read;
mod ads_write;
mod ads_read_state;
mod ads_read_write;
mod ads_add_device_notification;
mod ads_delete_device_notification;
mod ads_write_control;

use std::time::Instant;
use std::io;
use std::net::SocketAddr;
use std::mem::{size_of_val};
use std::sync::{Arc, Mutex, atomic::AtomicU16};

use tokio::net::TcpStream;
use tokio::runtime;
use tokio::io::{ReadHalf, WriteHalf};
use tokio::io::{AsyncWriteExt, AsyncReadExt};

use bytes::{Bytes, BytesMut};

use command_cleaner::CommandCleaner;
use command_manager::CommandManager;

use misc::{AdsError, AdsCommand, Handle, NotHandle, AmsNetId, AdsStampHeader, AdsNotificationSample};
pub use misc::{AdsTimeout, AdsNotificationAttrib, AdsTransMode, StateInfo, AdsState, Notification, Result}; // Re-export type


/// Size of the AMS/TCP + ADS headers
// https://infosys.beckhoff.com/content/1033/tc3_ads_intro/115845259.html?id=6032227753916597086
const HEADER_SIZE           : usize = 38;
const AMS_HEADER_SIZE       : usize = HEADER_SIZE - 6; // without leading nulls and length
const LEN_READ_REQ          : usize = 12;
const LEN_RW_REQ_MIN        : usize = 16;
const LEN_W_REQ_MIN         : usize = 12;
const LEN_ADD_DEV_NOT       : usize = 38;
const LEN_STAMP_HEADER_MIN  : usize = 12;
const LEN_NOT_SAMPLE_MIN    : usize = 8;
const LEN_DEL_DEV_NOT       : usize = 4;
const LEN_WR_CTRL_MIN       : usize = 8;

enum ProcessStateMachine{
    ReadHeader,
    ReadPayload { len_payload: usize, invoke_id: u32, cmd: AdsCommand}
}

/// An ADS client to use in combination with the [TC1000 ADS router](https://www.beckhoff.com/en-en/products/automation/twincat/tc1xxx-twincat-3-base/tc1000.html).
/// 
/// The client opens a port on the local ADS router in order to submit ADS requests.
/// Use the [Client::new] method to create an instance.
pub struct Client {
    _dst_addr       : AmsNetId,
    _dst_port       : u16,
    _src_addr       : AmsNetId,
    _src_port       : u16,
    timeout         : u64, // ADS Timeout [s]
    socket_wrt      : Arc<Mutex<WriteHalf<TcpStream>>>,
    handles         : Arc<Mutex<Vec<Handle>>>, // Internal stack of Handles (^=ADS CommandsInvoke) for decoupling requests and responses
    not_handles     : Arc<Mutex<Vec<NotHandle>>>,
    ams_header      : [u8; HEADER_SIZE],
    hdl_cnt         : Arc<AtomicU16>
}

// TODO: Implement Defaul trait
// https://doc.rust-lang.org/std/default/trait.Default.html

impl Client {
   
    async fn connect(answer: &mut [u8]) -> Result<TcpStream> {
        let stream  = TcpStream::connect(&SocketAddr::from(([127, 0, 0, 1], 48898))).await?;
        let handshake : [u8; 8] = [0x00, 0x10, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00 ];

        // WRITING
        loop {
            // Wait for the socket to be writable
            stream.writable().await?;
    
            // Try to write data, this may still fail with `WouldBlock`
            // if the readiness event is a false positive.
            match stream.try_write(&handshake) {
                Ok(_) => {
                    break;
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }

        // READING
        loop {
            // Wait for the socket to be readable
            stream.readable().await?;
    
            // Try to read data, this may still fail with `WouldBlock`
            // if the readiness event is a false positive.
            match stream.try_read(answer) {
                Ok(0) => break,
                Ok(n) => {
                    if n == 14 {
                        // println!("Connection established");
                        break;
                    } else {
                        return Err(Box::new(AdsError{n_error : 18})) // ERR_PORTDISABLED
                    }
                }

                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                
                Err(_) => {
                    //return Err(e.into()); // Default error
                    return Err(Box::new(AdsError{n_error : 18})) // ERR_PORTDISABLED
                }
            }
        }

        Ok(stream)
    } 

    async fn process_response(handles: Arc<Mutex<Vec<Handle>>>, not_handles: Arc<Mutex<Vec<NotHandle>>>, mut rd_stream : ReadHalf<TcpStream>) {
        
        let mut state = ProcessStateMachine::ReadHeader;
        let rt = runtime::Handle::current();
        loop {

            match &mut state {

                ProcessStateMachine::ReadHeader => {
                    //println!("READ AMS/ADS HEADER...");
                    let mut header_buf : [u8; HEADER_SIZE] = [0; HEADER_SIZE];

                    match rd_stream.read(&mut header_buf).await {
                        Ok(0) => {
                           //println!("Zero Bytes read");
                        }
                        Ok(_) => {
                            let len_payload = Client::extract_length(&header_buf).map_err(|_| Box::new(AdsError{n_error : 1})).unwrap();
                            let invoke_id   = Client::extract_invoke_id(&header_buf).map_err(|_| Box::new(AdsError{n_error : 1})).unwrap();   
                            let ads_cmd     = Client::extract_cmd_tyte(&header_buf).map_err(|_| Box::new(AdsError{n_error : 1})).unwrap();
                            // Create buffer of size payload
                            //println!("Payload: {:?}", len_payload);
                            //if len_payload > 0 {
                            state = ProcessStateMachine::ReadPayload{
                                len_payload : len_payload,
                                invoke_id   : invoke_id,
                                cmd         : ads_cmd
                            };
                            //}
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            //continue;
                        }
                        Err(e) => {
                            panic!("Socket Error (0x1): {:?}", e);
                        }
                    }
                }
                
                ProcessStateMachine::ReadPayload {len_payload, invoke_id, cmd} => {

                    let mut payload = BytesMut::with_capacity(*len_payload);
                    match rd_stream.read_buf(&mut payload).await {
                        Ok(0) => {
                           state = ProcessStateMachine::ReadHeader;
                        }
                        Ok(_) => {
                            
                            let buf = payload.freeze(); // Convert to Bytes
                            match cmd {
                                AdsCommand::DeviceNotification => {
                                    let _not_handles = Arc::clone(&not_handles); 
                                    rt.spawn(Client::process_device_notification(_not_handles, buf));

                                },
                                _ => {
                                    let _handles = Arc::clone(&handles);
                                    rt.spawn(Client::process_command(*invoke_id, _handles, buf));
                                }

                            };

                            state = ProcessStateMachine::ReadHeader;
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            //continue;
                        }
                        Err(e) => {
                            panic!("Socket Error (0x1): {:?}", e);
                        }
                    } // match
                }
            } // match
        } // loop
    } // fn

    async fn socket_write(&self, data: &[u8] ) -> Result<()> {

                let a_wrt_stream = Arc::clone(&self.socket_wrt);
                {
                    let mut wrt_stream = a_wrt_stream.lock();

                    match wrt_stream {
                        Ok(ref mut stream) => {
                            stream.write(data).await?;
                        },
                        Err(_) => {
                            return Err(Box::new(AdsError{n_error : 1}));
                        }
                    }
                }
                //Err(Box::new(AdsError{ n_error : 1792 })) // DEBUG
                Ok(())
                
    }
    
    /// Create a new instance of an ADS client.
    /// 
    /// - `addr` AmsNetId of the target system
    /// - `port` ADS port number to communicate with
    /// - `timeout` Value for ADS timeout value ([AdsTimeout::DefaultTimeout] corresponds to 5s)
    /// 
    /// # Example
    /// ```rust
    /// use ads_client::{Client, AdsTimeout, Result};
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     let ads_client =  Client::new("5.80.201.232.1.1", 851, AdsTimeout::DefaultTimeout).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(addr :&str, port : u16, timeout : AdsTimeout) -> Result<Self> {
        let state_flag : u16 = 4;
        let error_code : u32 = 0;
        let mut b_vec = Vec::<u8>::new();

        // BAUSTELLE // Pass ADS Address
        for s_byte in addr.split('.') {
            // https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/reenter_question_mark.html
            let n_byte = s_byte.parse::<u8>()?;
            b_vec.push(n_byte);
        }

        let timeout = match timeout {
            AdsTimeout::DefaultTimeout => 5,
            AdsTimeout::CustomTimeout(time) => time
        };

        let hdl_rt = runtime::Handle::current();

        let mut answer : [u8; 14] = [0; 14];

        let _stream = Client::connect(&mut answer).await?;

        // let client_port : u16 = u16::from_ne_bytes(answer[12..14].try_into().unwrap());
        // println!("Client ADS Port opened: {:?}", client_port);

        // Split the stream into a read and write part
        //
        // Read-half goes to process_response()
        // Write-half goes to Self

        let (read, write) = tokio::io::split(_stream);

        let a_socket_wrt = Arc::new(Mutex::new(write));
        

        // Create atomic instances of the handle vector
        let a_handles = Arc::new(Mutex::new( Vec::<Handle>::new() ));
        let a_not_handles =  Arc::new(Mutex::new( Vec::<NotHandle>::new() ));

        // Process incoming ADS responses
        let response_vector_a  = Arc::clone(&a_handles);
        let not_response_vector_a = Arc::clone(&a_not_handles);
        hdl_rt.spawn(Client::process_response(response_vector_a, not_response_vector_a, read));

        // Instantiate and spawn the CommandCleanter
        let response_vector_b = Arc::clone(&a_handles);
        hdl_rt.spawn(CommandCleaner::new(1, timeout, response_vector_b));

        Ok(Self {
            _dst_addr    : b_vec.clone().try_into().expect("AmsNetId consist of exact 6 bytes"), // https://stackoverflow.com/questions/25428920/how-to-get-a-slice-as-an-array-in-rust
            _dst_port    : port,
            _src_addr    : [answer[6], answer[7], answer[8], answer[9], answer[10], answer[11]],
            _src_port    : u16::from_ne_bytes(answer[12..14].try_into().expect("Parsing source port failed")),
            timeout      : timeout,
            socket_wrt   : a_socket_wrt,
            handles      : a_handles,
            not_handles  : a_not_handles,
            ams_header      : [
                0, // Reserved
                0,
                0, // Header size + playload
                0,
                0,
                0,
                b_vec[0], // Target NetId
                b_vec[1],
                b_vec[2],
                b_vec[3],
                b_vec[4],
                b_vec[5],
                u16_low_byte!(port), // Target port
                u16_high_byte!(port), 
                answer[6], //  Source NetId
                answer[7],
                answer[8],
                answer[9],
                answer[10],
                answer[11],
                answer[12], // Source Port
                answer[13], 
                0, // Command-Id
                0, 
                u16_low_byte!(state_flag), // State flags
                u16_high_byte!(state_flag), 
                0, // Length
                0,
                0,
                0, 
                u32_lw_lb!(error_code), // Error code
                u32_lw_hb!(error_code),
                u32_hw_lb!(error_code),
                u32_hw_hb!(error_code), 
                0, // Invoke Id
                0,
                0,
                0
            ],
            hdl_cnt         : Arc::new(AtomicU16::new(1))
        })
    }

    fn register_command_handle(&self, invoke_id : u32, cmd : AdsCommand){
        let a_handles = Arc::clone(&self.handles);

        let rs_req_hdl = Handle {
            cmd_type  : cmd,
            invoke_id : invoke_id,
            data      : None,
            timestamp : Instant::now(),
        };
    
        {
            let mut handles = a_handles.lock().expect("Threading Error");
            handles.push(rs_req_hdl);
        }
    }

    fn register_not_handle(&self, not_hdl: u32, callback: Notification, user_data: Option<&Arc<Mutex<BytesMut>>>) {
        let a_not_handles = Arc::clone(&self.not_handles);

        let not_hdl = NotHandle {
            callback  : callback,
            not_hdl   : not_hdl,
            user_data : user_data.and_then(|arc_bytes| Some(Arc::clone(arc_bytes)) )
        };

        {
            let mut not_handles = a_not_handles.lock().expect("Threading Error");
            not_handles.push(not_hdl);
        }
    }

    fn c_init_ams_header(&self, invoke_id : u32, length_payload : Option<u32>, cmd : AdsCommand) -> [u8; HEADER_SIZE] {
        let length_payload = length_payload.unwrap_or(0);
        let length_header : u32 = AMS_HEADER_SIZE as u32 + length_payload;

        let mut ams_header : [u8; HEADER_SIZE] = self.ams_header;
        // length header + payload
        ams_header[2..6].copy_from_slice(&length_header.to_ne_bytes());
        // command id
        ams_header[22..24].copy_from_slice(&(cmd as u16).to_ne_bytes());
        // length payload
        ams_header[26..30].copy_from_slice(&length_payload.to_ne_bytes());
        // invoke Id
        ams_header[34..38].copy_from_slice(&invoke_id.to_ne_bytes());

        ams_header
    }

    fn eval_return_code(answer: &[u8]) -> Result<u32> {
        let ret_code = u32::from_ne_bytes(answer[0..4].try_into().unwrap());

        if ret_code != 0 {
            return Err(Box::new(AdsError{ n_error : ret_code }));
        } else {
            Ok(ret_code)
        }
    }

    fn extract_invoke_id(answer: &[u8]) -> Result<u32> {
        Ok(u32::from_ne_bytes(answer[HEADER_SIZE-4..HEADER_SIZE].try_into()?))
    }

    fn extract_cmd_tyte(answer: &[u8]) -> Result<AdsCommand>{
        u16::from_ne_bytes(answer[HEADER_SIZE-16..HEADER_SIZE-14]
            .try_into()
            .map_err(|_| AdsError{n_error : 1})?)
            .try_into()
    }

    fn extract_length(answer: &[u8]) -> Result<usize>{
        // length in AMS-Header https://infosys.beckhoff.com/content/1031/tc3_ads_intro/115847307.html
        let tmp = u32::from_ne_bytes(answer[HEADER_SIZE-12..HEADER_SIZE-8].try_into()?);
        // Err(Box::new(AdsError{n_error : 1212})) // DEBUG
        Ok(usize::try_from(tmp)?)
    }

    fn not_extract_length(answer: &[u8]) -> Result<usize>{
        let tmp = u32::from_ne_bytes(answer[0..4].try_into()?);
        Ok(usize::try_from(tmp)?)
    }

    /// Panics if the input slice is less than 8 bytes
    fn not_extract_stamps(answer: &[u8]) -> Result<u32>{
        Ok(u32::from_ne_bytes(answer[4..8].try_into()?))
    }

    async fn process_command(invoke_id: u32, cmd_register: Arc<Mutex<Vec<Handle>>>, data: Bytes){
        let mut _handles = cmd_register.lock().expect("Threading Error");
        let mut _iter = _handles.iter_mut();

        if let Some(hdl) = _iter.find( | hdl | hdl.invoke_id == invoke_id) {
            hdl.data = Some(data);
        }
    }

    async fn process_device_notification(not_register: Arc<Mutex<Vec<NotHandle>>>, data: Bytes){
        let stream_size = Client::not_extract_length(&data).unwrap();
        let stamps      = Client::not_extract_stamps(&data).unwrap();
        let rt          = runtime::Handle::current();
        // Maximum stamp_header_offset == stream_size - sizeof(stamps)
        // ^= stream_size - 4
        let max_stamp_header_offset = stream_size + size_of_val(&stamps);
        let mut stamp_header_offset : usize = 8; // Start Idx

        for _ in 0..stamps { // Iterate over AdsStampHeader 
            // Return if there is not enough data
            if (stamp_header_offset + LEN_STAMP_HEADER_MIN) > max_stamp_header_offset {
                return;
            }
           
            let stamp_header = AdsStampHeader {
                timestamp : u64::from_ne_bytes(data[stamp_header_offset.. stamp_header_offset + 8].try_into().unwrap()),
                samples : u32::from_ne_bytes(data[stamp_header_offset + 8..stamp_header_offset + 12].try_into().unwrap())
            };

            // Increase stamp header offset, move it to first AdsNotificaionSample (+= 12 byte)
            stamp_header_offset += LEN_STAMP_HEADER_MIN;
            // == 20 (after first call)

            for _ in 0..stamp_header.samples {
                // Return if there is not enough data
                if (stamp_header_offset + LEN_NOT_SAMPLE_MIN) > max_stamp_header_offset {
                    return;
                }

                let not_sample = AdsNotificationSample {
                    not_hdl : u32::from_ne_bytes(data[stamp_header_offset..stamp_header_offset + 4].try_into().unwrap()),
                    sample_size : u32::from_ne_bytes(data[stamp_header_offset + 4 ..stamp_header_offset + 8].try_into().unwrap())
                };

                stamp_header_offset += LEN_NOT_SAMPLE_MIN;

                if (stamp_header_offset + not_sample.sample_size as usize) > max_stamp_header_offset {
                    return;
                }

                let mut _cb_and_data : Option<(Notification, Option<Arc<Mutex<BytesMut>>>)> = None;
                
                // The callback must be called after the lock. 
                // If it is called during the lock, it could block the access to the notification handles infinitely.

                { // LOCK
                    let mut _not_handles = not_register.lock().expect("Threading Error");
                    let mut _iter = _not_handles.iter_mut();
                    
                    _cb_and_data = _iter.find( | hdl | hdl.not_hdl  == not_sample.not_hdl)
                            .and_then(| hdl : &mut NotHandle | Some( (hdl.callback, hdl.user_data.clone()) ) ); // Return callback and user data
                } // UNLOCK
                
                
                _cb_and_data.and_then(|(callback, user_data)| {
                    let payload = Bytes::from(data.slice(stamp_header_offset..stamp_header_offset + not_sample.sample_size as usize));
                    // let n_cnt = u16::from_ne_bytes(payload[..].try_into().expect("Failed to parse data")); // DEBUG

                    Some(
                            rt.spawn(async move  {
                            callback(not_sample.not_hdl, stamp_header.timestamp, payload, user_data);
                        })
                    )
                    
                }); // Process join handles?

                stamp_header_offset += not_sample.sample_size as usize;
            } // for idx_notification_sample in 0..stamp_header.samples
        } // for idx_stamp_header in 0..stamps
    }
}