# Pico RTC-8564NB Example

This makes Raspberry Pi Pico

- reading time from [RTC-8564NB I2C real time clock module](https://www5.epsondevice.com/en/products/rtc/rtc8564nb.html)
- printing time through UART

### Usage

For a debug build

```sh
cargo run
```

For a release build

```sh
cargo run --release
```

### Circuit Diagram

![pico_i2c_uart_ブレッドボード](https://user-images.githubusercontent.com/618577/180661044-1de50956-70c6-4aba-acfa-bcf3c7cb9672.png)
