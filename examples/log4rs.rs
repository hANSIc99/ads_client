use ads_client::{Client, AdsTimeout, Result};
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
    Ok(())
}