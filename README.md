# ADS Client

An asynchronous, non-blocking ADS client for communication with Beckhoff controller.
This ADS client implementation requires the presence of a [TC1000 ADS router](https://www.beckhoff.com/en-en/products/automation/twincat/tc1xxx-twincat-3-base/tc1000.html) on the system.

## Examples

The ADS client requires the presence of the [tokio](https://tokio.rs/) runtime.
The examples denoted with *_async* are called from a main function denoted with with tokios [#[tokio::main]](https://docs.rs/tokio/latest/tokio/attr.main.html ) macro which causes the provision of the runtime. The examples without *_async* provides the runtime manually.

The provided examples rely on the related TwinCAT 3 project in [TC3_Sample_Project](https://github.com/hANSIc99/ads_client/tree/main/TC3_Sample_Project). The AmsNetId of the target system in the examples must be adapted accordingly.

Build and execute the examples with ```cargo run --example <example-name>```.

Following examples are available:
- [notification](/examples/notification.rs)
- [notification_async](/examples/notification_async.rs)
- [read_state](/examples/read_state.rs)
- [read_state_async](/examples/read_state_async.rs)
- [read_symbol](/examples/read_symbol.rs)
- [read_symbol_async](/examples/read_state_async.rs)
- [write_symbol](/examples/write_symbol.rs)
- [write_symbol_async](/examples/write_control_async.rs)
- [write_control](/examples/write_symbol.rs)
- [write_control_async](/examples/write_control_async.rs)
- [read_device_info](/examples/read_device_info.rs)
- [read_device_info_async](/examples/read_device_info_async.rs)

## Documentation

Build the documentation with:

```bash
cargo doc
```
Afterwards, the documentation can be found under **/target/doc/ads_client/index.html**.
