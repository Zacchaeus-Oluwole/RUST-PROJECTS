use log::*;
use esp_idf_sys::{self as _}; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
// ADC
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::gpio::*;
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
use std::sync::{Arc, Mutex};

fn main() -> anyhow::Result<()> {

    let peripherals = Peripherals::take().unwrap();

    // Create led object as GPIO4 output pin
    let mut led = PinDriver::output(peripherals.pins.gpio4).expect("Error to set pin Output");

    // Create an Arc and Mutex for sharing LDR value between threads/closures
    let value = Arc::new(Mutex::new(0));

    // Clone the value for the "/on" closure
    let value_on = Arc::clone(&value);

    // Clone the value for the "/off" closure
    let value_off = Arc::clone(&value);
    
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
        let html = light_html();
        let mut response = request.into_ok_response()?;
        response.write_all(html.as_bytes())?;
        Ok(())
    })?;

    // http://<sta ip>/on handler
    server.fn_handler("/on", Method::Get, move |request| {
        // Update the shared value
        let mut value_lock = value_on.lock().unwrap();
        *value_lock = 1;

        let mut response = request.into_ok_response()?;
        response.write_all("ON".as_bytes())?;
        Ok(())
    })?;

    server.fn_handler("/off", Method::Get, move |request| {
        let mut value_lock = value_off.lock().unwrap();
        *value_lock = 0;
        let mut response = request.into_ok_response()?;
        response.write_all("OFF".as_bytes())?;
        Ok(())
    })?;

    loop {
        thread::sleep(Duration::from_millis(1000));
        println!("IP info: {:?} ", wifi_driver.sta_netif().get_ip_info().unwrap());

        let val = value.lock().unwrap();
        let state_val = *val;

        if state_val == 0 {
            led.set_low()?;
        } else {
            led.set_high()?;
        }

    }
}

fn light_html() -> String {
    format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <link rel="icon" href="data:,">
        <style>
            html {{
            font-family: Helvetica;
            display: inline-block;
            margin: 0px auto;
            text-align: center;
            }}

            .button {{
            background-color: #4CAF50;
            border: none;
            color: white;
            padding: 16px 40px;
            text-decoration: none;
            font-size: 30px;
            margin: 2px;
            cursor: pointer;
            }}
        </style>
        </head>
        <body>
        <h1>Embedded Rust ESP32 Web Server</h1>
        <p>GPIO 4 - State <span id="output4State">%output4State%</span></p>
        <button class="button" onclick="toggleState('output4State')">Toggle GPIO 4</button>

        <script>
            function toggleState(elementId) {{
            var onurl = window.location.href.slice(0, -1) + "/on";
            var offurl = window.location.href.slice(0, -1) + "/off";
            var currentState = document.getElementById(elementId).innerText;
            var newState = currentState === 'ON' ? 'OFF' : 'ON';
            document.getElementById(elementId).innerText = newState;
            if(newState === "ON"){{
                fetch(onurl);
            }}else{{
                fetch(offurl);
            }}
            }}
        </script>
        </body>
        </html>
        "#
    )
}
