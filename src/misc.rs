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
pub struct StateInfo {
    pub ads_state    : AdsState,
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

impl TryFrom<u16> for AdsCommand{
    type Error = Box<dyn error::Error>;

    fn try_from(v: u16) -> Result<Self> {
        match v {
            x if x == AdsCommand::ReadDeviceInfo as u16 => Ok(AdsCommand::ReadDeviceInfo),
            x if x == AdsCommand::Read as u16 => Ok(AdsCommand::Read),
            x if x == AdsCommand::Write as u16 => Ok(AdsCommand::Write),
            x if x == AdsCommand::ReadState as u16 => Ok(AdsCommand::ReadState),
            x if x == AdsCommand::WriteControl as u16 => Ok(AdsCommand::WriteControl),
            x if x == AdsCommand::AddDeviceNotification as u16 => Ok(AdsCommand::AddDeviceNotification),
            x if x == AdsCommand::DeleteDeviceNotification as u16 => Ok(AdsCommand::DeleteDeviceNotification),
            x if x == AdsCommand::DeviceNotification as u16 => Ok(AdsCommand::DeviceNotification),
            x if x == AdsCommand::ReadWrite as u16 => Ok(AdsCommand::ReadWrite),
            _ => Err(Box::new(AdsError{n_error : 1}))
        }
    }
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
#[derive(Debug)]
pub enum AdsState {
    Invalid         = 0,
    Idle            = 1,
    Reset           = 2,
    Init            = 3,
    Start           = 4,
    Run             = 5,
    Stop            = 6,
    SaveCFG         = 7,
    LoadCFG         = 8,
    Powerfailure    = 9,
    PowerGood       = 10,
    Error           = 11,
    Shutdown        = 12,
    Suspend         = 13,
    Resume          = 14,
    Config          = 15, // system is in config mode
    Reconfig        = 16, // system should restart in config mode
}

// u16 to AdsState
impl TryFrom<u16> for AdsState {
    type Error = Box<dyn error::Error>;

    fn try_from(v: u16) -> Result<Self> {
        match v {
            x if x == AdsState::Invalid as u16          => Ok(AdsState::Invalid),
            x if x == AdsState::Idle as u16             => Ok(AdsState::Idle),
            x if x == AdsState::Reset as u16            => Ok(AdsState::Reset),
            x if x == AdsState::Init as u16             => Ok(AdsState::Init),
            x if x == AdsState::Start as u16            => Ok(AdsState::Start),
            x if x == AdsState::Run as u16              => Ok(AdsState::Run),
            x if x == AdsState::Stop as u16             => Ok(AdsState::Stop),
            x if x == AdsState::SaveCFG as u16          => Ok(AdsState::SaveCFG),
            x if x == AdsState::LoadCFG as u16          => Ok(AdsState::LoadCFG),
            x if x == AdsState::Powerfailure as u16     => Ok(AdsState::Powerfailure),
            x if x == AdsState::PowerGood as u16        => Ok(AdsState::PowerGood),
            x if x == AdsState::Error as u16            => Ok(AdsState::Error),
            x if x == AdsState::Shutdown as u16         => Ok(AdsState::Shutdown),
            x if x == AdsState::Suspend as u16          => Ok(AdsState::Suspend),
            x if x == AdsState::Resume as u16           => Ok(AdsState::Resume),
            x if x == AdsState::Config as u16           => Ok(AdsState::Config),
            x if x == AdsState::Reconfig as u16         => Ok(AdsState::Reconfig),
            _ => Err(Box::new(AdsError{n_error : 1}))
        }
    }
}