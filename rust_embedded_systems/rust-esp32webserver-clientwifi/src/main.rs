use std::{
    thread::sleep,
    time::Duration,
};
use embedded_svc::{http::Method, io::Write};
use anyhow::Result;
use esp_idf_hal::peripherals::Peripherals;

use esp_idf_svc::{
    wifi::EspWifi,
    nvs::EspDefaultNvsPartition,
    eventloop::EspSystemEventLoop,
    http::server::{Configuration, EspHttpServer},
};

use embedded_svc::wifi::{ClientConfiguration, Configuration as wifiConfiguration};

use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    info!("Hello, world!");

    let peripherals = Peripherals::take().unwrap();
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
        let html = index_html();
        let mut response = request.into_ok_response()?;
        response.write_all(html.as_bytes())?;
        Ok(())
    })?;

     
    loop{
        println!("IP info: {:?} ", wifi_driver.sta_netif().get_ip_info().unwrap());
        sleep(Duration::new(10, 0));
    }
}


fn index_html() -> String {
    format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>esp32 clock app webserver</title>
            <style>
                body {{
                    font-family: Arial, sans-serif;
                    text-align: center;
                    margin: 0;
                    padding: 0;
                    display: flex;
                    flex-direction: row;
                    height: 100vh;
                }}

                #left-section {{
                    flex: 1;
                    background-color: green;
                    color: white;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                }}

                #middle-section {{
                    flex: 1;
                    background-color: white;
                    color: green;     
                    align-items: center;
                    justify-content: center;
                }}

                #right-section {{
                    flex: 1;
                    background-color: green;
                    color: white;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                }}

                #clock {{
                    font-size: 3rem;
                }}
                #mw {{
                    flex: 1;
                    background-color: white;
                    color: green;     
                    align-items: center;
                    justify-content: center;
                }}
            </style>
        </head>
        <body>
            <div id="left-section">
                <h1></h1>
            </div>
            <div id="middle-section">
                <h1>Clock App Web Server on ESP32</h1>
                <div id="clock"></div>
                <div id="mw"><h1>Happy Independence To Nigeria</h1></div>
                <div id="mw"><h1>By Zacchaeus Oluwole</h1></div>
            </div>
            <div id="right-section">
                <h1></h1>
            </div>

            <script>
                function updateClock() {{
                    const clockElement = document.getElementById('clock');
                    const now = new Date();
                    const hours = now.getHours().toString().padStart(2, '0');
                    const minutes = now.getMinutes().toString().padStart(2, '0');
                    const seconds = now.getSeconds().toString().padStart(2, '0');

                    const timeString = `${{hours}}:${{minutes}}:${{seconds}}`;
                    clockElement.textContent = timeString;
                }}

                // Update the clock every second
                setInterval(updateClock, 1000);

                // Initial call to set the clock
                updateClock();
            </script>
        </body>
        </html>
        "#
    )
}