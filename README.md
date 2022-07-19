# Pico RTC-8564NB Example

This makes Raspberry Pi Pico

- to read time from RTC-8564NB I2C real time clock module (https://.epsondevice.com/en/products/rtc/rtc8564nb.html)
- to output time through UART
- to enable RTC-8564NB's alarm output

### How to Run

For a debug build

```sh
cargo run
```

For a release build

```sh
cargo run --release
```
