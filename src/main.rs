//! I2C test with SSD1306
//!
//! Folowing pins are used:
//! SDA     GPIO5
//! SCL     GPIO6
//!
//! Depending on your target and the board you are using you have to change the pins.
//!
//! For this example you need to hook up an SSD1306 I2C display.
//! The display will flash black and white.

use esp_idf_hal::delay::{FreeRtos, BLOCK};
use esp_idf_hal::i2c::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal as esp_idf_hal;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use std::io::Write;
use std::net::TcpStream;

use crate::networking::NetworkStack;

const SSD1306_ADDRESS: u8 = 0x3c;

mod networking;
mod options;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();

    println!("Rust main entered");

    let peripherals = Peripherals::take()?;
    // let i2c = peripherals.i2c0;
    // let sda = peripherals.pins.gpio5;
    // let scl = peripherals.pins.gpio6;

    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let mut net_stack = NetworkStack::configure(peripherals.modem, sys_loop.clone(), nvs)?;

    net_stack.start()?;
    println!("Connected");

    let mut stream = TcpStream::connect("192.168.4.68:5433")?;
    stream.write("hello world\r\n".as_bytes())?;

    println!("Starting I2C SSD1306 test");

    //let config = I2cConfig::new().baudrate(100.kHz().into());
    //let mut i2c = I2cDriver::new(i2c, sda, scl, &config)?;

    let mut counter = 1u32;
    loop {
        // we are sleeping here to make sure the watchdog isn't triggered
        FreeRtos::delay_ms(500);
        //i2c.write(SSD1306_ADDRESS, &[0, 0xa6], BLOCK)?;
        FreeRtos::delay_ms(500);
        //i2c.write(SSD1306_ADDRESS, &[0, 0xa7], BLOCK)?;
        stream.write(format!("count {counter}\r\n").as_bytes())?;
        counter += 1;
    }
}
