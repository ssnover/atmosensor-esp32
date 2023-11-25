use esp_idf_svc::hal::delay::FreeRtos;
use std::{net::Ipv4Addr, str::FromStr};

pub const DEVICE_IP: Option<&str> = option_env!("DEVICE_IP");
pub const GATEWAY_IP: Option<&str> = option_env!("GATEWAY_IP");
pub const GATEWAY_NETMASK: Option<&str> = option_env!("GATEWAY_NETMASK");
pub const AP_SSID: Option<&str> = option_env!("WIFI_SSID");
pub const AP_PASSWORD: Option<&str> = option_env!("WIFI_PASS");

pub fn access_point_credentials() -> (&'static str, &'static str) {
    match (AP_SSID, AP_PASSWORD) {
        (Some(ssid), Some(pass)) => (ssid, pass),
        _ => invalid_cfg_loop(),
    }
}

pub fn get_device_static_ip_addr() -> Option<Ipv4Addr> {
    match DEVICE_IP.map(|ip_addr| Ipv4Addr::from_str(ip_addr)) {
        Some(Ok(addr)) => Some(addr),
        Some(Err(err)) => {
            log::error!("Unable to parse static IP {}: {err}", DEVICE_IP.unwrap());
            None
        }
        None => None,
    }
}

pub fn get_gateway_info() -> Option<(Ipv4Addr, u8)> {
    match (
        GATEWAY_IP.map(|ip_addr| Ipv4Addr::from_str(ip_addr)),
        GATEWAY_NETMASK.map(|nm| u8::from_str(nm)),
    ) {
        (Some(Ok(addr)), Some(Ok(netmask))) => Some((addr, netmask)),
        _ => {
            log::error!(
                "No valid configuration for gateway: IP: {:?}, Netmask: {:?}",
                GATEWAY_IP,
                GATEWAY_NETMASK
            );
            None
        }
    }
}

fn invalid_cfg_loop() -> ! {
    loop {
        FreeRtos::delay_ms(1000);
        log::error!("No valid configuration for the WiFi STA credentials");
    }
}
