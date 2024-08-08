use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::*;
use esp_idf_hal::peripherals::Peripherals;
use log::*;
use esp_idf_svc::systime::EspSystemTime;

use std::sync::{Arc, Mutex};
// use log::*;
use esp_idf_sys::{self as _}; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

// WIFI-SERVER
use esp_idf_svc::{
    wifi::EspWifi,
    nvs::EspDefaultNvsPartition,
    eventloop::EspSystemEventLoop,
    http::server::{Configuration, EspHttpServer},
};
use embedded_svc::wifi::{ClientConfiguration, Configuration as wifiConfiguration};
use embedded_svc::{http::Method, io::Write};

fn main() -> anyhow::Result<()> {
    // ...
    // Initialize logging and necessary peripherals
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    info!("Hello, world!");

    let peripherals = Peripherals::take().unwrap();

    // Configure pins for trigger and echo
    let mut trigger_pin = PinDriver::output(peripherals.pins.gpio4).expect("Error configuring trigger pin");
    let echo_pin = PinDriver::input(peripherals.pins.gpio5).expect("Error configuring echo pin");

    // Create an Arc and Mutex for sharing Distance value between threads/closures
    let distance_value = Arc::new(Mutex::new(0.0));
    let distance_value_clone = distance_value.clone();
    let _unused = drop(distance_value_clone.lock().unwrap());

    // WIFI
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
        let html = distance_html();
        let mut response = request.into_ok_response()?;
        response.write_all(html.as_bytes())?;
        Ok(())
    })?;

    // http://<sta ip>/distance handler
    server.fn_handler("/distance", Method::Get, move |request| {
        let distance_val = distance_value_clone.lock().unwrap();
        let dist_value = *distance_val;
        let html = distance_api(dist_value);
        let mut response = request.into_ok_response()?;
        response.write_all(html.as_bytes())?;
        Ok(())
    })?;

    loop {
        // Wait for a brief moment before taking the next measurement
        FreeRtos::delay_ms(1000);
        println!("IP info: {:?} ", wifi_driver.sta_netif().get_ip_info().unwrap());
  
        // Send a 10us pulse to the trigger pin to start the measurement
        trigger_pin.set_high().expect("Error: Unable to set trigger pin high");
        FreeRtos::delay_us(10);
        trigger_pin.set_low().expect("Error: Unable to set trigger pin low");
  
        // Wait for the echo pin to go high (start of pulse)
        while !echo_pin.is_high() {}
  
        // Measure the duration of the echo pulse (in microseconds)
        let start_time = EspSystemTime {}.now().as_micros();
        while echo_pin.is_high() {}
        let end_time = EspSystemTime {}.now().as_micros();
  
        // Calculate the duration of the echo pulse in microseconds
        let pulse_duration = end_time - start_time;
  
        // Calculate the distance based on the speed of sound (approximately 343 m/s)
        // Distance in centimeters: duration * speed_of_sound / 2 (since the signal goes to the object and back)
        let distance_cm = (pulse_duration as f32 * 0.0343) / 2.0;
  
        // Update the shared distance value
        let mut distance_value_lock = distance_value.lock().unwrap();
        *distance_value_lock = distance_cm;
    }


    fn distance_api(dist_value: f32) -> String {
        format!("{:.2}", dist_value)
    }
    
    fn distance_html() -> String {
        format!(
            r#"
            <!DOCTYPE html>
            <html>
                <head>
                    <meta charset="UTF-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <title>HC-SR04 WEBSERVER WITH RUST ON ESP32</title>
                    <style>
                        body {{
                            font-family: Arial, sans-serif;
                            text-align: center;
                        }}
    
                        #dist {{
                            font-size: 3rem;
                            margin: 20px;
                        }}
                    </style>
                </head>
                <body>
                    <h1>HC-SR04's RUST WEB-SERVER ON ESP32</h1>
                    <div id="dist"></div>
    
                    <script>
                        async function getDistanceValue() {{
                            var geturl = window.location.href.slice(0, -1) + "/distance";
                            const distanceElement = document.getElementById('dist');
                            const distanceValue = await fetch(geturl);
                            const distance = await distanceValue.text();
                            const distanceString = `Distance Value : ${{distance}} cm`;
                            distanceElement.textContent = distanceString;
                        }}
    
                        // Update the distance every second
                        setInterval(getDistanceValue, 1000);
    
                        // Initial call to set the distance
                        getDistanceValue();
                    </script>
                </body>
            </html>
            "#
        )
    }

}