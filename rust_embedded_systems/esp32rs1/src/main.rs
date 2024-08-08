use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::*;
use esp_idf_hal::peripherals::Peripherals;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    info!("Hello, world!");

    // Initialize all required peripherals
    let peripherals = Peripherals::take().unwrap();

    // Create led object as GPIO4 output pin
    let mut led = PinDriver::output(peripherals.pins.gpio4).expect("Error to set pin Output");

    // Infinite loop where we are constantly turning ON and OFF the LED every 50ms
    loop{
        led.set_high().expect("Error: Unable to set pin high");

        // We are sleeping here to make sure the watchdog isn't triggered
        FreeRtos::delay_ms(1000);
        
        led.set_low().expect("Error: Unable to set pin low");
        FreeRtos::delay_ms(1000);
    }
}
