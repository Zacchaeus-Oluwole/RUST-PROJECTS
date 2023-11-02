use std::{
    thread::sleep,
    time::Duration,
    sync::{Mutex, Arc},
    str::from_utf8
};
use embedded_svc::{
    http::Method,
    io::Write,
    wifi::{ClientConfiguration, Configuration as wifiConfiguration},

};
use anyhow::Result;
use esp_idf_hal::{
    peripherals::Peripherals,
    ledc::{LedcTimerDriver, config::TimerConfig, LedcDriver},
    units::*,
};
use esp_idf_svc::{
    wifi::EspWifi,
    nvs::EspDefaultNvsPartition,
    eventloop::EspSystemEventLoop,
    http::server::{Configuration, EspHttpServer},
};
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;
// use esp_idf_hal::units::*;

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

    let servo_timer = peripherals.ledc.timer1;
    let servo_driver = LedcTimerDriver::new(servo_timer, &TimerConfig::new().frequency(50.Hz().into()).resolution(esp_idf_hal::ledc::Resolution::Bits14)).unwrap();
    let servo = Arc::new(Mutex::new(LedcDriver::new(peripherals.ledc.channel3, servo_driver, peripherals.pins.gpio2).unwrap()));

    let max_duty = servo.lock().unwrap().get_max_duty();

    let max = max_duty/8;
    let min = max_duty/40;

    fn interpolate(angle: u32, max: u32, min: u32) ->u32 {
        angle * (max - min)/ 180 + min
    }

    server.fn_handler("/servo", Method::Post, move |mut request| {
        let mut buf = [0_u8; 6];
        let bytes_read = request.read(&mut buf).unwrap();
        let angle_string = from_utf8(&buf[0..bytes_read]).unwrap();
        let angle = angle_string.parse().unwrap();

        servo.lock().unwrap().set_duty(interpolate(angle, max, min)).unwrap();
        Ok(())
    })?;

     
    loop{
        println!("IP info: {:?} ", wifi_driver.sta_netif().get_ip_info().unwrap());
        sleep(Duration::new(10, 0));
    }
}
