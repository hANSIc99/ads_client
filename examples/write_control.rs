use std::{thread, time::Duration};
use ads_client::{Client, AdsTimeout, StateInfo, AdsState};
use tokio::runtime::Runtime;

fn main() {
    let rt = Runtime::new().unwrap();
    let ads_client = rt.block_on(Client::new("5.80.201.232.1.1", 10000, AdsTimeout::DefaultTimeout)).unwrap();

    // Set target system to config mode
    let new_state_config = StateInfo {ads_state : AdsState::Reconfig, device_state : 0 };

    match rt.block_on(ads_client.write_control(&new_state_config, None)) {
        Ok(_) => println!("State change to {:?} successfull", new_state_config.ads_state),
        Err(err) => println!("Error: {}", err.to_string())
    }

    thread::sleep(Duration::from_secs(5));

    // Set target to run mode
    let new_state_run = StateInfo {ads_state : AdsState::Reset, device_state : 0 };

    match rt.block_on(ads_client.write_control(&new_state_run, None)) {
        Ok(_) => println!("State change to {:?} successfull", new_state_run.ads_state),
        Err(err) => println!("Error: {}", err.to_string())
    }
}