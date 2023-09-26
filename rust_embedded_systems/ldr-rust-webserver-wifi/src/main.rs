use anyhow::Result;
use core::str;
use embedded_svc::{http::Method, io::Write};
use esp_idf_hal::{
    adc::config::Config as AdcConfig,
    adc::*,
    peripherals::Peripherals,
    gpio::Gpio4,
    // prelude::*,
};
use esp_idf_svc::{
    wifi::EspWifi, //
    nvs::EspDefaultNvsPartition, // 
    eventloop::EspSystemEventLoop,
    http::server::{Configuration, EspHttpServer},
};
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

// use wifi::wifi;
// use embedded_svc::wifi::{ClientConfiguration, Configuration as wifiConfiguration}; //
use embedded_svc::wifi::{AccessPointConfiguration, Configuration as wifiConfiguration}; //
use esp_idf_sys as _;

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take().unwrap(); //

    // Wifi 
    let mut wifi_driver = EspWifi::new(  //
        peripherals.modem,  //
        sysloop,  //
        Some(nvs)  //
    ).unwrap();   //

    // wifi_driver.set_configuration(&wifiConfiguration::Client(ClientConfiguration{
    wifi_driver.set_configuration(&wifiConfiguration::AccessPoint(AccessPointConfiguration{  //
        ssid: "RustEspWifi".into(),
        password: "RustEsp32".into(),
        ..Default::default()
    })).unwrap();  //

    wifi_driver.start().unwrap();  //
    wifi_driver.connect().unwrap();  //
    while !wifi_driver.is_connected().unwrap(){
        let config = wifi_driver.get_configuration().unwrap();
        println!("Waiting for station {:?} ", config);
    }
    println!("Should be connected now");  //

    // // Initialize ADC for LDR
    // Configure ADC Driver
    let mut adc = AdcDriver::new(peripherals.adc1, &AdcConfig::new()).unwrap();

    // Configure ADC Channel
    let mut adc_pin: esp_idf_hal::adc::AdcChannelDriver<'_, Gpio4, Atten11dB<_>> =
        AdcChannelDriver::new(peripherals.pins.gpio4).unwrap();

    // Create an Arc and Mutex for sharing LDR value between threads
    let ldr_value = Arc::new(Mutex::new(0.0));
    let ldr_value_clone = ldr_value.clone();

    // Create a thread to continuously update the LDR value
    thread::spawn(move || {
        loop {
            // Read the ADC value from the LDR pin.
            let adc_value = adc.read(&mut adc_pin).unwrap();

            // Convert the ADC value to an LDR value based on your calibration.
            // You may need to adjust this conversion based on your LDR and circuit.
            let ldr = adc_to_ldr_value(adc_value as u32);

            // Update the shared LDR value
            let mut ldr_value = ldr_value_clone.lock().unwrap();
            *ldr_value = ldr;

            // Sleep for a while before the next reading
            thread::sleep(Duration::from_millis(1000));
        }
    });

    // Set the HTTP server
    let mut server = EspHttpServer::new(&Configuration::default())?;
    // http://<sta ip>/ handler
    server.fn_handler("/", Method::Get, move |request| {
        let ldr_val = {
            // Read the shared LDR value
            let ldr_value = ldr_value.lock().unwrap();
            *ldr_value
        };

        let html = index_html(ldr_val);
        let mut response = request.into_ok_response()?;
        response.write_all(html.as_bytes())?;
        Ok(())
    })?;


    loop {
        println!("IP info: {:?} ", wifi_driver.sta_netif().get_ip_info().unwrap());
        thread::sleep(Duration::from_millis(2000));
    }
}

// Function to convert ADC value to LDR value.
fn adc_to_ldr_value(adc_value: u32) -> f32 {
    // You need to calibrate this conversion based on your LDR and circuit.
    // It should map the ADC value to a meaningful LDR value.
    // You may need to use a datasheet or experimentation to determine the mapping.
    const MAX_ADC: u32 = 4095; // Maximum ADC value (12-bit ADC)
    const LDR_MAX_VALUE: f32 = 100.0; // Maximum LDR value

    // Perform the conversion
    let ldr_value = (MAX_ADC - adc_value) as f32 / MAX_ADC as f32 * LDR_MAX_VALUE;

    ldr_value
}

fn templated(content: impl AsRef<str>) -> String {
    format!(
        r#"
        <!DOCTYPE html>
        <html>
            <head>
                <meta charset="utf-8">
                <title>esp-rs web server</title>
            </head>
            <body>
                {}
            </body>
        </html>
        "#,
        content.as_ref()
    )
}

fn index_html(ldr_value: f32) -> String {
    templated(format!("LDR Value: {:.2}", ldr_value))
}