use std::{fmt, io, num, error, convert, array};
use std::time::Instant;
use std::sync::{Arc, Mutex};
use bytes::{Bytes, BytesMut};
use num_enum::{IntoPrimitive, TryFromPrimitive};

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
pub type Result<T> = std::result::Result<T, AdsError>;

/// Type definition for notification callback.
/// 
/// Arguments:
/// 1. Handle
/// 2. Timestamp
/// 3. Value of monitored variable
/// 4. User data
pub type Notification = fn(u32, u64, Bytes, Option<Arc<Mutex<BytesMut>>>) -> (); // handle, timestamp and user data

/// Error type of returned Result
///  
/// An overview of possible error codes can be found in the [InfoSys](https://infosys.beckhoff.com/content/1033/devicemanager/374277003.html).
#[derive(Debug, Clone)]
pub struct AdsError {
    pub n_error : u32,
    pub s_msg   : String
}

impl error::Error for AdsError{}

impl fmt::Display for AdsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AdsClientError 0x{:x} - {}", self.n_error, self.s_msg)
    }
}

impl From<io::Error> for AdsError{
    fn from(error: io::Error) -> Self {
        // 10 / 0xA : ERR_NOIO
        AdsError {n_error : 10, s_msg :  error.to_string()}
    }
}

impl From<num::TryFromIntError> for AdsError{
    fn from(error: num::TryFromIntError) -> Self {
        // 1 : Internal Error
        AdsError {n_error : 1, s_msg : error.to_string() }
    }
}

impl From<array::TryFromSliceError> for AdsError{
    fn from(error: array::TryFromSliceError) -> Self {
        // 1 : Internal Error
        AdsError {n_error : 1, s_msg : error.to_string() }
    }
}

impl From<num::ParseIntError> for AdsError{
    fn from(error: num::ParseIntError) -> Self {
        // 1 : Internal Error
        AdsError {n_error : 1, s_msg :  error.to_string() }
    }
}

impl From<convert::Infallible> for AdsError{
    fn from(_error: convert::Infallible) -> Self {
        // 1 : Internal Error
        AdsError {n_error : 1, s_msg : String::from("") }
    }
}


/// Determines the notification mechanism.
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
pub struct AdsNotificationAttrib {
    pub cb_length   : u32,
    pub trans_mode  : AdsTransMode,
    pub max_delay   : u32,
    pub cycle_time  : u32
}

#[derive(Debug)]
#[derive(Default)]
#[derive(PartialEq)]
#[derive(Eq)]
pub struct AdsStampHeader {
    pub timestamp   : u64,
    pub samples     : u32
}

#[derive(Debug)]
#[derive(Default)]
#[derive(PartialEq)]
#[derive(Eq)]
pub struct AdsNotificationSample {
    pub not_hdl     : u32,
    pub sample_size : u32
}

#[derive(Default)]
pub struct HandleData {
    pub ams_err : u32,
    pub payload : Option<Bytes>
}

pub struct Handle {
    pub cmd_type  : AdsCommand,
    pub invoke_id : u32,
    pub data      : HandleData,
    pub timestamp : Instant, // Timestamp of creation
}

pub struct NotHandle {
    pub callback  : Notification,
    pub not_hdl   : u32,
    pub user_data : Option<Arc<Mutex<BytesMut>>>,
}
/// Specifies the maximum waiting time for an ADS response.
/// 
/// - [AdsTimeout::DefaultTimeout] Corresponds to 5 seconds.
/// - [AdsTimeout::CustomTimeout] Value in seconds.
pub enum AdsTimeout {
    DefaultTimeout,
    CustomTimeout(u64)
}

/// ADS State and device state of a target system.
#[derive(Default)]
#[derive(Debug)]
#[derive(PartialEq)]
pub struct StateInfo {
    pub ads_state    : AdsState,
    pub device_state : u16
}

/// Device information
#[derive(Debug)]
pub struct DeviceStateInfo {
    pub major : u8,
    pub minor : u8,
    pub build : u16,
    pub device_name : String
    // pub device_name : [u8; 16],
    // pub s_device_name : &'a str
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
#[derive(Debug)]
pub enum AdsCommand {
    Invalid = 0,
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

impl Default for AdsCommand {
    fn default() -> Self { AdsCommand::Invalid }
}

impl TryFrom<u16> for AdsCommand{
    type Error = AdsError;

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
            _ => Err(AdsError{n_error : 1, s_msg : String::from("AdsCommand: Conversion from u16 failed")})
        }
    }
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
#[derive(PartialEq)]
#[derive(Debug)]
/// ADS State of target system.
/// 
/// To switch a TwinCAT 3 system to Config mode, set it to [AdsState::Reconfig], 
/// for Run mode set it to [AdsState::Reset].
/// 
/// Checkout the [ADS Write Control example](https://github.com/hANSIc99/ads_client/blob/main/examples/write_control.rs) in the repsoitory.
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

impl Default for AdsState {
    fn default() -> Self { AdsState::Invalid }
}

impl TryFrom<u16> for AdsState {
    type Error = AdsError;

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
            _ => Err(AdsError{n_error : 1, s_msg : String::from("AdsState: Conversion from u16 failed")})
        }
    }
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive, Clone, Copy, PartialOrd)]
#[repr(u32)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub enum AdsErrorCode {
    /// No error.
    ERR_NOERROR = 0,
    /// Internal error.
    ERR_INTERNAL = 1,
    /// No real time.
    ERR_NORTIME = 2,
    /// Allocation locked – memory error.
    ERR_ALLOCLOCKEDMEM = 3,
    /// Mailbox full – the ADS message could not be sent. Reducing the number of ADS messages per cycle will help.
    ERR_INSERTMAILBOX = 4,
    /// Wrong HMSG.
    ERR_WRONGRECEIVEHMSG = 5,
    /// Target port not found – ADS server is not started or is not reachable.
    ERR_TARGETPORTNOTFOUND = 6,
    /// Target computer not found – AMS route was not found.
    ERR_TARGETMACHINENOTFOUND = 7,
    /// Unknown command ID.
    ERR_UNKNOWNCMDID = 8,
    /// Invalid task ID.
    ERR_BADTASKID = 9,
    /// No IO.
    ERR_NOIO = 10,
    /// Unknown AMS command.
    ERR_UNKNOWNAMSCMD = 11,
    /// Win32 error.
    ERR_WIN32ERROR = 12,
    /// Port not connected.
    ERR_PORTNOTCONNECTED = 13,
    /// Invalid AMS length.
    ERR_INVALIDAMSLENGTH = 14,
    /// Invalid AMS Net ID.
    ERR_INVALIDAMSNETID = 15,
    /// Installation level is too low –TwinCAT 2 license error.
    ERR_LOWINSTLEVEL = 16,
    /// No debugging available.
    ERR_NODEBUGINTAVAILABLE = 17,
    /// Port disabled – TwinCAT system service not started.
    ERR_PORTDISABLED = 18,
    /// Port already connected.
    ERR_PORTALREADYCONNECTED = 19,
    /// AMS Sync Win32 error.
    ERR_AMSSYNC_W32ERROR = 20,
    /// AMS Sync Timeout.
    ERR_AMSSYNC_TIMEOUT = 21,
    /// AMS Sync error.
    ERR_AMSSYNC_AMSERROR = 22,
    /// No index map for AMS Sync available.
    ERR_AMSSYNC_NOINDEXINMAP = 23,
    /// Invalid AMS port.
    ERR_INVALIDAMSPORT = 24,
    /// No memory.
    ERR_NOMEMORY = 25,
    /// TCP send error.
    ERR_TCPSEND = 26,
    /// Host unreachable.
    ERR_HOSTUNREACHABLE = 27,
    /// Invalid AMS fragment.
    ERR_INVALIDAMSFRAGMENT = 28,
    /// TLS send error – secure ADS connection failed.
    ERR_TLSSEND = 29,
    /// Access denied – secure ADS access denied.
    ERR_ACCESSDENIED = 30,
    /// Locked memory cannot be allocated.
    ROUTERERR_NOLOCKEDMEMORY = 1280,
    /// The router memory size could not be changed.
    ROUTERERR_RESIZEMEMORY = 1281,
    /// The mailbox has reached the maximum number of possible messages.
    ROUTERERR_MAILBOXFULL = 1282,
    /// The Debug mailbox has reached the maximum number of possible messages.
    ROUTERERR_DEBUGBOXFULL = 1283,
    /// The port type is unknown.
    ROUTERERR_UNKNOWNPORTTYPE = 1284,
    /// The router is not initialized.
    ROUTERERR_NOTINITIALIZED = 1285,
    /// The port number is already assigned.
    ROUTERERR_PORTALREADYINUSE = 1286,
    /// The port is not registered.
    ROUTERERR_NOTREGISTERED = 1287,
    /// The maximum number of ports has been reached.
    ROUTERERR_NOMOREQUEUES = 1288,
    /// The port is invalid.
    ROUTERERR_INVALIDPORT = 1289,
    /// The router is not active.
    ROUTERERR_NOTACTIVATED = 1290,
    /// The mailbox has reached the maximum number for fragmented messages.
    ROUTERERR_FRAGMENTBOXFULL = 1291,
    /// A fragment timeout has occurred.
    ROUTERERR_FRAGMENTTIMEOUT = 1292,
    /// The port is removed.
    ROUTERERR_TOBEREMOVED = 1293,
    /// General device error.
    ADSERR_DEVICE_ERROR = 1792,
    /// Service is not supported by the server.
    ADSERR_DEVICE_SRVNOTSUPP = 1793,
    /// Invalid index group.
    ADSERR_DEVICE_INVALIDGRP = 1794,
    /// Invalid index offset.
    ADSERR_DEVICE_INVALIDOFFSET = 1795,
    /// Reading or writing not permitted.
    ADSERR_DEVICE_INVALIDACCESS = 1796,
    /// Parameter size not correct.
    ADSERR_DEVICE_INVALIDSIZE = 1797,
    /// Invalid data values.
    ADSERR_DEVICE_INVALIDDATA = 1798,
    /// Device is not ready to operate.
    ADSERR_DEVICE_NOTREADY = 1799,
    /// Device is busy.
    ADSERR_DEVICE_BUSY = 1800,
    /// Invalid operating system context. This can result from use of ADS blocks in different tasks. It may be possible to resolve this through multitasking synchronization in the PLC.
    ADSERR_DEVICE_INVALIDCONTEXT = 1801,
    /// Insufficient memory.
    ADSERR_DEVICE_NOMEMORY = 1802,
    /// Invalid parameter values.
    ADSERR_DEVICE_INVALIDPARM = 1803,
    /// Not found (files, ...).
    ADSERR_DEVICE_NOTFOUND = 1804,
    /// Syntax error in file or command.
    ADSERR_DEVICE_SYNTAX = 1805,
    /// Objects do not match.
    ADSERR_DEVICE_INCOMPATIBLE = 1806,
    /// Object already exists.
    ADSERR_DEVICE_EXISTS = 1807,
    /// Symbol not found.
    ADSERR_DEVICE_SYMBOLNOTFOUND = 1808,
    /// Invalid symbol version. This can occur due to an online change. Create a new handle.
    ADSERR_DEVICE_SYMBOLVERSIONINVALID = 1809,
    /// Device (server) is in invalid state.
    ADSERR_DEVICE_INVALIDSTATE = 1810,
    /// AdsTransMode not supported.
    ADSERR_DEVICE_TRANSMODENOTSUPP = 1811,
    /// Notification handle is invalid.
    ADSERR_DEVICE_NOTIFYHNDINVALID = 1812,
    /// Notification client not registered.
    ADSERR_DEVICE_CLIENTUNKNOWN = 1813,
    /// No further handle available.
    ADSERR_DEVICE_NOMOREHDLS = 1814,
    /// Notification size too large.
    ADSERR_DEVICE_INVALIDWATCHSIZE = 1815,
    /// Device not initialized.
    ADSERR_DEVICE_NOTINIT = 1816,
    /// Device has a timeout.
    ADSERR_DEVICE_TIMEOUT = 1817,
    /// Interface query failed.
    ADSERR_DEVICE_NOINTERFACE = 1818,
    /// Wrong interface requested.
    ADSERR_DEVICE_INVALIDINTERFACE = 1819,
    /// Class ID is invalid.
    ADSERR_DEVICE_INVALIDCLSID = 1820,
    /// Object ID is invalid.
    ADSERR_DEVICE_INVALIDOBJID = 1821,
    /// Request pending.
    ADSERR_DEVICE_PENDING = 1822,
    /// Request is aborted.
    ADSERR_DEVICE_ABORTED = 1823,
    /// Signal warning.
    ADSERR_DEVICE_WARNING = 1824,
    /// Invalid array index.
    ADSERR_DEVICE_INVALIDARRAYIDX = 1825,
    /// Symbol not active.
    ADSERR_DEVICE_SYMBOLNOTACTIVE = 1826,
    /// Access denied.
    ADSERR_DEVICE_ACCESSDENIED = 1827,
    /// Missing license.
    ADSERR_DEVICE_LICENSENOTFOUND = 1828,
    /// License expired.
    ADSERR_DEVICE_LICENSEEXPIRED = 1829,
    /// License exceeded.
    ADSERR_DEVICE_LICENSEEXCEEDED = 1830,
    /// Invalid license.
    ADSERR_DEVICE_LICENSEINVALID = 1831,
    /// License problem: System ID is invalid.
    ADSERR_DEVICE_LICENSESYSTEMID = 1832,
    /// License not limited in time.
    ADSERR_DEVICE_LICENSENOTIMELIMIT = 1833,
    /// Licensing problem: time in the future.
    ADSERR_DEVICE_LICENSEFUTUREISSUE = 1834,
    /// License period too long.
    ADSERR_DEVICE_LICENSETIMETOLONG = 1835,
    /// Exception at system startup.
    ADSERR_DEVICE_EXCEPTION = 1836,
    /// License file read twice.
    ADSERR_DEVICE_LICENSEDUPLICATED = 1837,
    /// Invalid signature.
    ADSERR_DEVICE_SIGNATUREINVALID = 1838,
    /// Invalid certificate.
    ADSERR_DEVICE_CERTIFICATEINVALID = 1839,
    /// Public key not known from OEM.
    ADSERR_DEVICE_LICENSEOEMNOTFOUND = 1840,
    /// License not valid for this system ID.
    ADSERR_DEVICE_LICENSERESTRICTED = 1841,
    /// Demo license prohibited.
    ADSERR_DEVICE_LICENSEDEMODENIED = 1842,
    /// Invalid function ID.
    ADSERR_DEVICE_INVALIDFNCID = 1843,
    /// Outside the valid range.
    ADSERR_DEVICE_OUTOFRANGE = 1844,
    /// Invalid alignment.
    ADSERR_DEVICE_INVALIDALIGNMENT = 1845,
    /// Invalid platform level.
    ADSERR_DEVICE_LICENSEPLATFORM = 1846,
    /// Context – forward to passive level.
    ADSERR_DEVICE_FORWARD_PL = 1847,
    /// Context – forward to dispatch level.
    ADSERR_DEVICE_FORWARD_DL = 1848,
    /// Context – forward to real time.
    ADSERR_DEVICE_FORWARD_RT = 1849,
    /// Client error.
    ADSERR_CLIENT_ERROR = 1856,
    /// Service contains an invalid parameter.
    ADSERR_CLIENT_INVALIDPARM = 1857,
    /// Polling list is empty.
    ADSERR_CLIENT_LISTEMPTY = 1858,
    /// Var connection already in use.
    ADSERR_CLIENT_VARUSED = 1859,
    /// The called ID is already in use.
    ADSERR_CLIENT_DUPLINVOKEID = 1860,
    /// Timeout has occurred – the remote terminal is not responding in the specified ADS timeout. The route setting of the remote terminal may be configured incorrectly.
    ADSERR_CLIENT_SYNCTIMEOUT = 1861,
    /// Error in Win32 subsystem.
    ADSERR_CLIENT_W32ERROR = 1862,
    /// Invalid client timeout value.
    ADSERR_CLIENT_TIMEOUTINVALID = 1863,
    /// Port not open.
    ADSERR_CLIENT_PORTNOTOPEN = 1864,
    /// No AMS address.
    ADSERR_CLIENT_NOAMSADDR = 1865,
    /// Internal error in Ads sync.
    ADSERR_CLIENT_SYNCINTERNAL = 1872,
    /// Hash table overflow.
    ADSERR_CLIENT_ADDHASH = 1873,
    /// Key not found in the table.
    ADSERR_CLIENT_REMOVEHASH = 1874,
    /// No symbols in the cache.
    ADSERR_CLIENT_NOMORESYM = 1875,
    /// Invalid response received.
    ADSERR_CLIENT_SYNCRESINVALID = 1876,
    /// Sync Port is locked.
    ADSERR_CLIENT_SYNCPORTLOCKED = 1877,
    /// The request was cancelled.
    ADSERR_CLIENT_REQUESTCANCELLED = 1878,
    /// Internal error in the real-time system.
    RTERR_INTERNAL = 4096,
    /// Timer value is not valid.
    RTERR_BADTIMERPERIODS = 4097,
    /// Task pointer has the invalid value 0 (zero).
    RTERR_INVALIDTASKPTR = 4098,
    /// Stack pointer has the invalid value 0 (zero).
    RTERR_INVALIDSTACKPTR = 4099,
    /// The request task priority is already assigned.
    RTERR_PRIOEXISTS = 4100,
    /// No free TCB (Task Control Block) available. The maximum number of TCBs is 64.
    RTERR_NOMORETCB = 4101,
    /// No free semaphores available. The maximum number of semaphores is 64.
    RTERR_NOMORESEMAS = 4102,
    /// No free space available in the queue. The maximum number of positions in the queue is 64.
    RTERR_NOMOREQUEUES = 4103,
    /// An external synchronization interrupt is already applied.
    RTERR_EXTIRQALREADYDEF = 4109,
    /// No external sync interrupt applied.
    RTERR_EXTIRQNOTDEF = 4110,
    /// Application of the external synchronization interrupt has failed.
    RTERR_EXTIRQINSTALLFAILED = 4111,
    /// Call of a service function in the wrong context
    RTERR_IRQLNOTLESSOREQUAL = 4112,
    /// Intel VT-x extension is not supported.
    RTERR_VMXNOTSUPPORTED = 4119,
    /// Intel VT-x extension is not enabled in the BIOS.
    RTERR_VMXDISABLED = 4120,
    /// Missing function in Intel VT-x extension.
    RTERR_VMXCONTROLSMISSING = 4121,
    /// Activation of Intel VT-x fails.
    RTERR_VMXENABLEFAILS = 4122,
    /// A connection timeout has occurred - error while establishing the connection, because the remote terminal did not respond properly after a certain period of time, or the established connection could not be maintained because the connected host did not respond.
    WSAETIMEDOUT = 10060,
    /// Connection refused - no connection could be established because the target computer has explicitly rejected it. This error usually results from an attempt to connect to a service that is inactive on the external host, that is, a service for which no server application is running.
    WSAECONNREFUSED = 10061,
    /// No route to host - a socket operation referred to an unavailable host.
    WSAEHOSTUNREACH = 10065,
    /// Unknown ads error
    UNKNOWN,
}

impl From<AdsError> for AdsErrorCode {
    fn from(value: AdsError) -> Self {
        AdsErrorCode::try_from(value.n_error).unwrap_or(AdsErrorCode::UNKNOWN)
    }
}
