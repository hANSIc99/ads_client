use ads_client::{Client, AdsTimeout, Result, AdsNotificationAttrib, AdsTransMode};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use bytes::{Bytes, BytesMut, BufMut};
use log::LevelFilter;
use log4rs::filter::threshold::ThresholdFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Logger, Root};

#[tokio::main]
async fn main() -> Result<()> {


    // Define appender "stderr"
    let stderr = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d:<35.35} - {l} - {f}:{L}- {m}{n}")))
        .build();
    
    // Define appender "logfile"
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {l} - {f}:{L}- {m}{n}")))
        .build("log/requests.log")
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(log::LevelFilter::Info)))
                .build("stderr", Box::new(stderr)),
        )
        .build(
            Root::builder()
                .appender("logfile")
                .appender("stderr")
                .build(LevelFilter::Trace),
        )
        .unwrap();


    let _handle = log4rs::init_config(config).unwrap();

    let ads_client = Client::new("5.80.201.232.1.1", 851, AdsTimeout::DefaultTimeout).await?;

    //read_symbol(&ads_client).await;
    device_notofication(&ads_client).await;

    Ok(())
}

async fn device_notofication(ads_client: &Client){
    // Get symbol handle
    let mut var_hdl_a : [u8; 4] = [0; 4];
    let mut var_hdl_b : [u8; 4] = [0; 4];
    let mut var_hdl_c : [u8; 4] = [0; 4];

    let symbol_a = b"MAIN.n_cnt_a";
    let symbol_b = b"MAIN.n_cnt_b";
    let symbol_c = b"MAIN.n_cnt_c";

    let buf_n_cnt_a = Arc::new(Mutex::new( BytesMut::with_capacity(2)));
    let buf_n_cnt_b = Arc::new(Mutex::new( BytesMut::with_capacity(2)));
    let buf_n_cnt_c = Arc::new(Mutex::new( BytesMut::with_capacity(2)));


    // Get handle for  MAIN.n_cnt_a (1/s)
    if let Err(err) = ads_client.read_write(0xF003, 0, &mut var_hdl_a, symbol_a).await{
        eprintln!("Error: {}", err.to_string());
        panic!();
    }

    let var_hdl_a = u32::from_ne_bytes(var_hdl_a.try_into().unwrap());


    // Get handle for  MAIN.n_cnt_b (10/s)
    if let Err(err) = ads_client.read_write(0xF003, 0, &mut var_hdl_b, symbol_b).await{
        eprintln!("Error: {}", err.to_string());
        panic!();
    }

    let var_hdl_b = u32::from_ne_bytes(var_hdl_b.try_into().unwrap());

    // Get handle for  MAIN.n_cnt_c (1000/s)
    if let Err(err) = ads_client.read_write(0xF003, 0, &mut var_hdl_c, symbol_c).await{
        eprintln!("Error: {}", err.to_string());
        panic!();
    }

    let var_hdl_c = u32::from_ne_bytes(var_hdl_c.try_into().unwrap());



    if var_hdl_a != 0 && var_hdl_b != 0 && var_hdl_c != 0 {
        println!("Got handles!");
        println!("Handle n_cnt_a: {}", var_hdl_a);
        println!("Handle n_cnt_b: {}", var_hdl_b);
        println!("Handle n_cnt_c: {}", var_hdl_c);

        // Register Device Notification for n_cnt_a
        let mut not_hdl_a : u32 = 0;

        let ads_notification_attrib_a = AdsNotificationAttrib {
            cb_length   : 2, // UINT
            trans_mode  : AdsTransMode::OnChange, // trigger notification when value changed
            max_delay   : 0, // send notification asap
            cycle_time  : 0  // check for value change each cycle
        };

        match ads_client.add_device_notification(   0xF005,
                                                    var_hdl_a,
                                                    &ads_notification_attrib_a,
                                                    &mut not_hdl_a,
                                                    notification_a,
                                                    Some(&buf_n_cnt_a)).await
        {
            Ok(_)     => {
                println!("Waiting for notifications on n_cnt_a!");
            },
            Err(err) => println!("Error: {}", err.to_string())
        }



        // // Register Device Notification for n_cnt_b
        let mut not_hdl_b : u32 = 0;

        let ads_notification_attrib_b = AdsNotificationAttrib {
            cb_length   : 2, // UINT
            trans_mode  : AdsTransMode::OnChange,
            max_delay   : 500, // sumup notifications each 500ms
            cycle_time  : 0 // check for value change each cycle
        };
        
        match ads_client.add_device_notification(   0xF005,
                                                    var_hdl_b,
                                                    &ads_notification_attrib_b,
                                                    &mut not_hdl_b,
                                                    notification_b,
                                                    Some(&buf_n_cnt_b)).await
        {
            Ok(_)     => {
                println!("Waiting for notifications on n_cnt_b!");
            },
            Err(err) => println!("Error: {}", err.to_string())
        }

        // // Register Device Notification for n_cnt_c
        let mut not_hdl_c : u32 = 0;

        let ads_notification_attrib_c = AdsNotificationAttrib {
            cb_length   : 2, // UINT
            trans_mode  : AdsTransMode::OnChange,
            max_delay   : 100, // sumup notifications each 100ms
            cycle_time  : 0 // check for value change each cycle
        };
        
        match ads_client.add_device_notification(   0xF005,
                                                    var_hdl_c,
                                                    &ads_notification_attrib_c,
                                                    &mut not_hdl_c,
                                                    notification_c,
                                                    Some(&buf_n_cnt_c)).await
        {
            Ok(_)     => {
                println!("Waiting for notifications on n_cnt_c!");
            },
            Err(err) => println!("Error: {}", err.to_string())
        }


        thread::sleep(Duration::from_secs(5));

        match ads_client.delete_device_notification(not_hdl_a).await {
            Ok(_)     => {
                println!("Notification for n_cnt_a deleted.");
            },
            Err(err) => println!("Error: {}", err.to_string())
        }

        match ads_client.delete_device_notification(not_hdl_b).await {
            Ok(_)     => {
                println!("Notification for n_cnt_b deleted.");
            },
            Err(err) => println!("Error: {}", err.to_string())
        }

        match ads_client.delete_device_notification(not_hdl_c).await {
            Ok(_)     => {
                println!("Notification for n_cnt_c deleted.");
            },
            Err(err) => println!("Error: {}", err.to_string())
        }

        let b_n_cnt_a : Bytes;
        let b_n_cnt_b : Bytes;
        let b_n_cnt_c : Bytes;
        { // LOCK
            let lock_n_cnt_a = buf_n_cnt_a.lock().expect("Threading error");
            b_n_cnt_a = lock_n_cnt_a.clone().freeze();
        } // UNLOCK
        { // LOCK
            let lock_n_cnt_b = buf_n_cnt_b.lock().expect("Threading error");
            b_n_cnt_b = lock_n_cnt_b.clone().freeze();
        } // UNLOCK
        { // LOCK
            let lock_n_cnt_c = buf_n_cnt_c.lock().expect("Threading error");
            b_n_cnt_c = lock_n_cnt_c.clone().freeze();
        } // UNLOCK

        // TODO: len prüfen
        let n_cnt_a = u16::from_ne_bytes(b_n_cnt_a[0..2].try_into().expect("Failed to prase data"));
        println!("Final value n_cnt_a: {}", n_cnt_a);

        let n_cnt_b = u16::from_ne_bytes(b_n_cnt_b[0..2].try_into().expect("Failed to prase data"));
        println!("Final value n_cnt_b: {}", n_cnt_b);

        let n_cnt_c = u16::from_ne_bytes(b_n_cnt_c[0..2].try_into().expect("Failed to prase data"));
        println!("Final value n_cnt_c: {}", n_cnt_c);
    }
}


fn notification_a(_handle: u32, _timestamp: u64, payload: Bytes, user_data: Option<Arc<Mutex<BytesMut>>>){
    let n_cnt_a = u16::from_ne_bytes(payload[..].try_into().expect("Failed to parse data"));
    println!("Notification Event!, n_cnt_a: {}", n_cnt_a);

    // Process userdata if available
    if user_data.is_some() {
        let user_data = user_data.unwrap();

        { // LOCK
            let mut user_data = user_data.lock().expect("Threading error");
            user_data.clear();
            user_data.put(&payload[..]);
        } // UNLOCK
    }
}

fn notification_b(_handle: u32, _timestamp: u64, payload: Bytes, user_data: Option<Arc<Mutex<BytesMut>>>){
    let n_cnt_b = u16::from_ne_bytes(payload[..].try_into().expect("failed to parse data"));
    println!("Notification Event!, n_cnt_b: {}", n_cnt_b);

    // Process userdata if available
    if user_data.is_some() {
        let user_data = user_data.unwrap();

        { // LOCK
            let mut user_data = user_data.lock().expect("Threading error");
            user_data.clear();
            user_data.put(&payload[..]);
        } // UNLOCK
    }
}

fn notification_c(_handle: u32, _timestamp: u64, payload: Bytes, user_data: Option<Arc<Mutex<BytesMut>>>){
    let n_cnt_c = u16::from_ne_bytes(payload[..].try_into().expect("failed to parse data"));
    if n_cnt_c % 100 == 0 {
        println!("Notification Event!, n_cnt_c: {}", n_cnt_c);
    }
    

    // Process userdata if available
    if user_data.is_some() {
        let user_data = user_data.unwrap();

        { // LOCK
            let mut user_data = user_data.lock().expect("Threading error");
            user_data.clear();
            user_data.put(&payload[..]);
        } // UNLOCK
    }
}

async fn read_symbol(ads_client: &Client){
        // Get symbol handle
        let mut hdl : [u8; 4] = [0; 4];
        let symbol = b"MAIN.n_cnt_a";
    
        if let Err(err) = ads_client.read_write(0xF003, 0, &mut hdl, symbol).await{
            println!("Error: {}", err.to_string());
        }
    
        let n_hdl = u32::from_ne_bytes(hdl.try_into().unwrap());
    
        if n_hdl != 0 {
            println!("Got handle!");
    
            let mut plc_n_cnt_a : [u8; 2] = [0; 2];
            
    
            let read_hdl = ads_client.read(0xF005, n_hdl, &mut plc_n_cnt_a).await;
    
            match read_hdl {
                Ok(_bytes_read)     => {
                    let n_cnt_a = u16::from_ne_bytes(plc_n_cnt_a.try_into().unwrap());
                    println!("MAIN.n_cnt_a: {}", n_cnt_a);
                },
                Err(err) => println!("Read failed: {}", err.to_string())
            }
        }
}