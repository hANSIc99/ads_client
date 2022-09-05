use std::fmt;
use std::error;
use std::time::Instant;
use std::sync::{Arc, Mutex};
use bytes::{Bytes, BytesMut};

#[macro_use]
mod misc {
    #[macro_export]
    macro_rules!  u16_low_byte {
        ($x:expr) => {
            $x  as  u8 & 0xff
        };
    }
    #[macro_export]
    macro_rules!  u16_high_byte {
        ($x:expr) => {
            ($x >> 8) as  u8 & 0xff
        };
    }
    #[macro_export]
    macro_rules!  u32_hw_hb {
        ($x:expr) => {
            ($x >> 24) as  u8 & 0xff
        };
    }
    #[macro_export]
    macro_rules!  u32_hw_lb {
        ($x:expr) => {
            ($x >> 16) as  u8 & 0xff
        };
    }
    #[macro_export]
    macro_rules!  u32_lw_hb {
        ($x:expr) => {
            ($x >> 8) as  u8 & 0xff
        };
    }
    #[macro_export]
    macro_rules!  u32_lw_lb {
        ($x:expr) => {
            $x  as  u8 & 0xff
        };
    }

    #[allow(dead_code)]
    fn print_type_of<T>(_: &T) {
        println!("{}", std::any::type_name::<T>())
    }
}

pub type AmsNetId = [u8; 6];
pub type Result<T> = std::result::Result<T, Box<dyn error::Error>>;
pub type Notification = fn(u32, u64, Bytes, Option<Arc<Mutex<BytesMut>>>) -> (); // handle, timestamp and user data


#[derive(Debug, Clone)]
pub struct AdsError {
    pub n_error : u32
}

impl error::Error for AdsError{}

impl fmt::Display for AdsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AdsClientError 0x{:x}", self.n_error)
    }
}

#[derive(Copy, Clone)]
pub enum AdsTransMode {
    NoTrans     = 0,
    ServerCycle = 3,
    OnChange    = 4
}

pub struct AdsNotificationAttrib {
    pub cb_length   : u32,
    pub trans_mode  : AdsTransMode,
    pub max_delay   : u32,
    pub cycle_time  : u32
}

#[derive(Debug)]
pub struct AdsStampHeader {
    pub timestamp   : u64,
    pub samples     : u32
}

#[derive(Debug)]
pub struct AdsNotificationSample {
    pub not_hdl     : u32,
    pub sample_size : u32
}

pub struct Handle {
    pub cmd_type  : AdsCommand,
    pub invoke_id : u32,
    pub data      : Option<Bytes>,
    pub timestamp : Instant,
}

pub struct NotHandle {
    pub callback  : Notification,
    pub not_hdl   : u32,
    pub user_data : Option<Arc<Mutex<BytesMut>>>,
}

pub enum AdsTimeout {
    DefaultTimeout,
    CustomTimeout(u64)
}

#[derive(Debug)]
pub struct AdsStateInfo {
    pub ads_state    : u16,
    pub device_state : u16
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
#[derive(Debug)]
pub enum AdsCommand {
    ReadDeviceInfo = 1,
    Read = 2,
    Write = 3,
    ReadState = 4,
    WriteControl = 5,
    AddDeviceNotification = 6,
    DeleteDeviceNotification = 7,
    DeviceNotification = 8,
    ReadWrite = 9
}