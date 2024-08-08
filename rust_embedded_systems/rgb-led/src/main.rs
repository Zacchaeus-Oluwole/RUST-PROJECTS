
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::*;
use esp_idf_hal::peripherals::Peripherals;

fn main(){
    
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Author: Zacchaeus Oluwole!");
    let peripherals = Peripherals::take().unwrap();
    let mut r_pin = PinDriver::output(peripherals.pins.gpio2).expect("Error: Unable to set pin(r) gpio2 Output");
    let mut g_pin = PinDriver::output(peripherals.pins.gpio4).expect("Error: Unable to set pin(g) gpio4 Output");
    let mut b_pin = PinDriver::output(peripherals.pins.gpio5).expect("Error: Unable to set pin(b) gpio5 Output");

    loop{
        r_pin.set_high().expect("Error: Unable to set pin r high");
        FreeRtos::delay_ms(1000);
        r_pin.set_low().expect("Error: Unable to set pin r low");
        g_pin.set_high().expect("Error: Unable to set pin g high");
        FreeRtos::delay_ms(1000);
        g_pin.set_low().expect("Error: Unable to set pin g low");
        b_pin.set_high().expect("Error: Unable to set pin b high");
        FreeRtos::delay_ms(1000);
        b_pin.set_low().expect("Error: Unable to set pin b low");
    }
}