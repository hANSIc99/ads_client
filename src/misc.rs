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
/// Type definition for notification callback
/// 
/// Arguments:
/// 1. Handle
/// 2. Timestamp
/// 3. Value of monitored variable
/// 4. User data
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

/// Determines the notification mechanism
/// 
/// - `ServerCycle` The notification is fired cyclically at intervals of [AdsNotificationAttrib::cycle_time].
/// - `OnChange` The notification is fired only if the values has changed.
/// 
/// Please also read the related documentation in the [InfoSys](https://infosys.beckhoff.com/content/1031/tc3_adsdll2/117553803.html).
#[derive(Copy, Clone)]
pub enum AdsTransMode {
    ServerCycle = 3,
    OnChange    = 4
}

/// Defines the notification attributes
/// 
/// Please also read the related documentation in the [InfoSys](https://infosys.beckhoff.com/content/1033/tc3_adsdll2/117553803.html).
///
/// - `cb_length` Size of the datatype to monitor.
/// - `AdsTransMode` Specifies when to trigger a notification (see [AdsTransMode]).
/// - `max_delay` Maximal acceptable delay \[100ns\].
/// - `cycle_time` The interval at which the variable is checked \[100ns]\.
/// 
/// # Examples
/// 
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
/// Specifies the maximum waiting time for an ADS response
/// 
/// `DefaultTimeout` Corresponds to 5 seconds
/// `CustomTimeout` Value in seconds
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