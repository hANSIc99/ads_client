

This ADS client implementation requires the presence of a (TC1000 ADS router)[https://www.beckhoff.com/en-en/products/automation/twincat/tc1xxx-twincat-3-base/tc1000.html] on the system.

The ads_client is implemented asynchronous, thus it requires the tokio runtime in order run.

x
Run 

cargo doc

cargo build --example notification