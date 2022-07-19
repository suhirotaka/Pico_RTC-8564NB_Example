//! Pico With RTC-8564NB Example
//!
//! This makes Raspberry Pi Pico
//! - to read time from RTC-8564NB I2C real time clock module (https://www5.epsondevice.com/en/products/rtc/rtc8564nb.html)
//! - to output time through UART
//! - to enable RTC-8564NB's alarm output

#![no_std]
#![no_main]

use core::fmt::Write as FmtWrite;
use embedded_hal::blocking::i2c::{Operation, Transactional};
use embedded_time::rate::*;
use panic_halt as _;
use rp_pico::{entry, hal, hal::pac, hal::prelude::*};

/// Pico calls this function after startup
#[entry]
fn main() -> ! {
    // Set constants
    const RTC_I2C_ADDRESS: u8 = 0x51u8;

    // Get peripherals
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    // Configure clock
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();
    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().integer());

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins up according to their function on this particular board
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Configure GPIO pins
    let uart_tx_pin = pins.gpio0;
    let uart_rx_pin = pins.gpio1;
    let i2c_sda_pin = pins.gpio2;
    let i2c_scl_pin = pins.gpio3;

    // Configure UART
    let uart_pins = (
        uart_tx_pin.into_mode::<hal::gpio::FunctionUart>(),
        uart_rx_pin.into_mode::<hal::gpio::FunctionUart>(),
    );
    let mut uart = hal::uart::UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            hal::uart::common_configs::_115200_8_N_1,
            clocks.peripheral_clock.into(),
        )
        .unwrap();

    // Configure I2C
    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let mut i2c_pio = pico_rtc_8564nb_example::I2C::new(
        &mut pio,
        i2c_sda_pin,
        i2c_scl_pin,
        sm0,
        100_000.Hz(),
        clocks.system_clock.freq(),
    );

    // Set time to 2022/07/19 18:29:00 (Tue)
    let time_year = &[decimal_to_bcd(22)];
    let time_month = &[decimal_to_bcd(7)];
    let time_day = &[decimal_to_bcd(19)];
    let time_weekday = &[decimal_to_bcd(2)];
    let time_hour = &[decimal_to_bcd(18)];
    let time_minute = &[decimal_to_bcd(29)];
    let time_second = &[decimal_to_bcd(0)];

    // Set alarm to 2022/07/19 18:30:00 (Tue)
    let alarm_day = &[decimal_to_bcd(0x80)];
    let alarm_weekday = &[decimal_to_bcd(0x80)];
    let alarm_hour = &[decimal_to_bcd(0x80)];
    let alarm_minute = &[decimal_to_bcd(30)];

    // Set RTC-8564NB register
    let mut operations = [
        Operation::Write(&[0]), // Set register address to 00
        Operation::Write(&[0]),
        Operation::Write(&[0x02]), // Enable alarm flag output
        // Set time
        Operation::Write(time_second),
        Operation::Write(time_minute),
        Operation::Write(time_hour),
        Operation::Write(time_day),
        Operation::Write(time_weekday),
        Operation::Write(time_month),
        Operation::Write(time_year),
        // Set alarm
        Operation::Write(alarm_minute),
        Operation::Write(alarm_hour),
        Operation::Write(alarm_day),
        Operation::Write(alarm_weekday),
        // Disable timer
        Operation::Write(&[0]),
        Operation::Write(&[0]),
        Operation::Write(&[0]),
    ];
    i2c_pio
        .exec(RTC_I2C_ADDRESS, &mut operations)
        .expect("Failed to set RTC-8564NB register");

    // Loop to read from RTC and print time
    loop {
        let mut _alarm = [0];
        let mut second = [0];
        let mut minute = [0];
        let mut hour = [0];
        let mut day = [0];
        let mut weekday = [0];
        let mut month = [0];
        let mut year = [0];

        let mut operations = [
            // Set register address to 02
            Operation::Write(&[1]),
            // Read alarm flag
            Operation::Read(&mut _alarm),
            // Read time
            Operation::Read(&mut second),
            Operation::Read(&mut minute),
            Operation::Read(&mut hour),
            Operation::Read(&mut day),
            Operation::Read(&mut weekday),
            Operation::Read(&mut month),
            Operation::Read(&mut year),
        ];
        i2c_pio
            .exec(RTC_I2C_ADDRESS, &mut operations)
            .expect("Failed to read RTC-8564NB register");

        print_time(&mut uart, year, month, day, weekday, hour, minute, second);

        delay.delay_ms(5000);
    }
}

// Parse response from RTC-8564NB and print it
fn print_time(
    serial: &mut impl FmtWrite,
    year: [u8; 1],
    month: [u8; 1],
    day: [u8; 1],
    _weekday: [u8; 1],
    hour: [u8; 1],
    minute: [u8; 1],
    second: [u8; 1],
) {
    let _ = writeln!(
        serial,
        "Time: {0: >02}/{1: >02}/{2: >02} {3: >02}:{4: >02}:{5: >02}\r\n",
        bcd_to_decimal(year[0]),
        bcd_to_decimal(month[0] & 0x1fu8),
        bcd_to_decimal(day[0] & 0x3fu8),
        bcd_to_decimal(hour[0] & 0x3fu8),
        bcd_to_decimal(minute[0] & 0x7fu8),
        bcd_to_decimal(second[0] & 0x7fu8),
    );
}

// Convert BCD value to decimal
fn bcd_to_decimal(bcd: u8) -> u8 {
    ((bcd & 0xf0u8) >> 4) * 10 + (bcd & 0x0fu8)
}

// Convert decimal value to BCD
fn decimal_to_bcd(decimal: u8) -> u8 {
    (decimal / 10 << 4) + decimal % 10
}

// End of file
