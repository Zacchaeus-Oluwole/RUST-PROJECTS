use std::sync::{Arc, Mutex};
use log::*;
use esp_idf_sys::{self as _}; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
// ADC
use esp_idf_hal::adc::config::Config as adcConfig;
use esp_idf_hal::adc::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::gpio::Gpio34;
// WIFI-SERVER
use esp_idf_svc::{
    wifi::EspWifi,
    nvs::EspDefaultNvsPartition,
    eventloop::EspSystemEventLoop,
    http::server::{Configuration, EspHttpServer},
};
use embedded_svc::wifi::{ClientConfiguration, Configuration as wifiConfiguration};
use embedded_svc::{http::Method, io::Write};
// STD
use std::{
    // sync::{Arc, Mutex},
    thread,
    time::Duration,
};

fn main() -> anyhow::Result<()> {
    let peripherals = Peripherals::take().unwrap();

    // Configure ADC Driver
    let mut adc = AdcDriver::new(peripherals.adc1, &adcConfig::new()).unwrap();

    // Configure ADC Channel
    let mut adc_pin: esp_idf_hal::adc::AdcChannelDriver<'_, Gpio34, Atten11dB<_>> =
        AdcChannelDriver::new(peripherals.pins.gpio34).unwrap();

    // Create an Arc and Mutex for sharing LDR value between threads/closures
    let ldr_value = Arc::new(Mutex::new(0.0));
    let ldr_value_clone = ldr_value.clone();
    let _ = ldr_value_clone.lock().unwrap();
    
    // WIFI - WEBSERVER

    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let mut wifi_driver = EspWifi::new(
        peripherals.modem,
        sys_loop,
        Some(nvs)
    ).unwrap();

    wifi_driver.set_configuration(&wifiConfiguration::Client(ClientConfiguration{
        ssid: "mySSID".into(),
        password: "myPASSWORD".into(),
        ..Default::default()
    })).unwrap();

    wifi_driver.start().unwrap();
    wifi_driver.connect().unwrap();
    while !wifi_driver.is_connected().unwrap(){
        let config = wifi_driver.get_configuration().unwrap();
        println!("Waiting for station {:?} ", config);
    }
    println!("Should be connected now");

    // Set the HTTP server
    let mut server = EspHttpServer::new(&Configuration::default())?;
    // http://<sta ip>/ handler
    server.fn_handler("/", Method::Get, |request| {
        let html = ldr_html();
        let mut response = request.into_ok_response()?;
        response.write_all(html.as_bytes())?;
        Ok(())
    })?;

    // http://<sta ip>/ldr handler

    server.fn_handler("/ldr", Method::Get, move |request| {
        // Convert the ADC value to an LDR value based on your calibration.
        // You may need to adjust this conversion based on your LDR and circuit.
        // let ldr_value = adc_to_ldr_value(adc_value as u32);
        let ldr_val = ldr_value_clone.lock().unwrap();
        let ldr_html = *ldr_val;
        let html = ldr_api(ldr_html);
        let mut response = request.into_ok_response()?;
        response.write_all(html.as_bytes())?;
        Ok(())
    })?;

    loop {
        // Get ADC Reading
        // thread::sleep(Duration::from_millis(10));
        thread::sleep(Duration::from_millis(1000));
        println!("IP info: {:?} ", wifi_driver.sta_netif().get_ip_info().unwrap());
        // println!("LDR value: {}", adc.read(&mut adc_pin).unwrap());
        // Read the ADC value from the LDR pin.
        let adc_value = adc.read(&mut adc_pin).unwrap();

        // Convert the ADC value to an LDR value based on your calibration.
        // You may need to adjust this conversion based on your LDR and circuit.
        let ldr = adc_to_ldr_value(adc_value as u32);

        // Update the shared LDR value
        let mut ldr_value_lock = ldr_value.lock().unwrap();
        *ldr_value_lock = ldr;
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

fn ldr_api(ldr_value: f32) -> String {
    format!("{:.2}", ldr_value)
}

fn ldr_html() -> String {
    format!(
        r#"
        <!DOCTYPE html>
        <html>
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>LDR RUST WEBSERVER FOR ESP32</title>
                <style>
                    body {{
                        font-family: Arial, sans-serif;
                        text-align: center;
                    }}

                    #ldr {{
                        font-size: 3rem;
                        margin: 20px;
                    }}
                </style>
            </head>
            <body>
                <h1>LDR's RUST WEB-SERVER ON ESP32</h1>
                <div id="ldr"></div>

                <script>
                    async function getLdrValue() {{
                        var geturl = window.location.href.slice(0, -1) + "/ldr";
                        const ldrElement = document.getElementById('ldr');
                        const ldrValue = await fetch(geturl);
                        const ldr = await ldrValue.text();
                        const ldrString = `LDR Value : ${{ldr}}`;
                        ldrElement.textContent = ldrString;
                    }}

                    // Update the clock every second
                    setInterval(getLdrValue, 1000);

                    // Initial call to set the clock
                    getLdrValue();
                </script>
            </body>
        </html>
        "#
    )
}
