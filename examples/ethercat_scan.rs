
use std::fmt;
use ads_client::{Client, AdsTimeout, Result};

type EtherCATSlaveState = std::result::Result<EcState, EcSlaveError>;


#[derive(Debug)]
#[derive(PartialEq)]
enum EcState {
    Init            = 0x0001,
    PreOp           = 0x0002,
    Boot            = 0x0003,
    SafeOp          = 0x0004,
    Op              = 0x0008
}

impl From<u8> for EcState {
    fn from(x: u8) -> Self {
        match x {
            0x01 => EcState::Init,
            0x02 => EcState::PreOp,
            0x03 => EcState::Boot,
            0x04 => EcState::SafeOp,
            0x08 => EcState::Op,
            _ => panic!("Invalid value: {x}")
        }
    }
}


#[derive(Debug)]
#[derive(PartialEq)]
enum EcErrState {
    Ok              = 0x00,
    Err             = 0x10,
    VprsErr         = 0x20,
    InitErr         = 0x40,
    Disabled        = 0x80
}

impl From<u8> for EcErrState {
    fn from(x: u8) -> Self {
        match x {
            0x00 => EcErrState::Ok,
            0x10 => EcErrState::Err,
            0x20 => EcErrState::VprsErr,
            0x40 => EcErrState::InitErr,
            0x80 => EcErrState::Disabled,
            _ => panic!("Invalid value: {x}")
        }
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
enum EcLinkState {
    Ok              = 0x0000,
    NotPresent      = 0x0100,
    LinkError       = 0x0200,
    MissLink        = 0x0400,
    UnexpectedLink  = 0x0800,
}

impl From<u8> for EcLinkState {
    fn from(x: u8) -> Self {
        let word: u16 = (x as u16) << 8;

        match word {
            0x0000 => EcLinkState::Ok,
            0x0100 => EcLinkState::NotPresent,
            0x0200 => EcLinkState::LinkError,
            0x0400 => EcLinkState::MissLink,
            0x0800 => EcLinkState::UnexpectedLink,
            _ => panic!("Invalid value: {x}")
        }
    }
    
}

#[derive(Debug)]
#[derive(PartialEq)]
enum EcLinkPort {
    None            = 0x0000,
    ComPortA        = 0x1000,
    ComPortB        = 0x2000,
    ComPortC        = 0x4000,
    ComPortD        = 0x8000
}

impl From<u8> for EcLinkPort {
    fn from(x: u8) -> Self {
        let word: u16 = (x as u16) << 8;

        match word {
            0x0000 => EcLinkPort::None,
            0x1000 => EcLinkPort::ComPortA,
            0x2000 => EcLinkPort::ComPortB,
            0x4000 => EcLinkPort::ComPortC,
            0x8000 => EcLinkPort::ComPortD,
            _ => panic!("Invalid value: {x}")
        }
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
struct EcSlaveError {
    ec_state        : EcState,
    ec_err_state    : EcErrState,
    link_state      : EcLinkState,
    link_port       : EcLinkPort
}




#[derive(Debug)]
struct EtherCATSlave {
    state : EtherCATSlaveState,
}

impl EtherCATSlave {
    //pub fn new(value: [u8; 2] ) -> Self {
    pub async fn new(addr :&str) -> Result<Self> {
        
        let ads_client = Client::new(addr, 0xFFFF, AdsTimeout::DefaultTimeout).await?;
        let mut ec_state_raw : [u8; 2] = [0; 2];
        let rd_result = ads_client.read(0x00000009, 1002, &mut ec_state_raw).await?;

        // println!("value[0] : {:?}", value[0]); // 0x00
        // println!("value[1] : {:?}", value[1]); // 0x08

        let ec_state = EcState::from(ec_state_raw[0] & 0x0F);
        println!("EcStateMachine state {:?}", ec_state);

        let ec_err_state = EcErrState::from(ec_state_raw[0] & 0xF0);
        println!("EcErrorState {:?}", ec_err_state);

        let link_state = EcLinkState::from(ec_state_raw[1] & 0x0F);
        println!("EcLinkState {:?}", link_state);

        let link_port = EcLinkPort::from(ec_state_raw[1] & 0xF0);
        println!("EcLinkPort {:?}", link_port);

        if ec_err_state != EcErrState::Ok || link_state != EcLinkState::Ok {     
            Ok(EtherCATSlave {
                state : Err(EcSlaveError {
                    ec_state : ec_state,
                    ec_err_state : ec_err_state,
                    link_state : link_state,
                    link_port : link_port
                })
            })
        } else {
            Ok(EtherCATSlave {
                state : EtherCATSlaveState::Ok(ec_state)
            })
        }
    }
}



#[tokio::main]
async fn main() -> Result<()> {

    let ec_slave = EtherCATSlave::new("5.80.201.232.2.1").await?;
    println!("ec_slave : {:?}", ec_slave.state); // 0x08
    let x = 3;

    Ok(())
}

// fn f() -> Result<(), anyhow::Error> {
//     g().map_err(|e| e.context(format!("at {}:{}:{}", file!(), line!(), column!())))?;
//     Err(anyhow::anyhow!("some other error"))
// }

// fn g() -> Result<(), anyhow::Error> {
//     Err(anyhow::anyhow!("oh noes"))
// }

// fn main() -> Result<()>{
//     let rt = Runtime::new().unwrap();
//     let ads_client = rt.block_on(Client::new("5.80.201.232.1.1", 10000, AdsTimeout::DefaultTimeout)).unwrap();

//     let ads_state = rt.block_on(ads_client.read_state())?;
//     //println!("State: {:?}", ads_client.read_state().unwrap());
//     // match rt.block_on(ads_client.read_state()) {
//     //     Ok(state) => println!("State: {:?}", state),
//     //     Err(err) => println!("Error: {}", err.to_string())
//     // }
//     Ok(())
// }