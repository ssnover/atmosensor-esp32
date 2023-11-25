use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::modem::Modem,
    ipv4::{ClientSettings, Configuration as IpConfiguration, Mask, Subnet},
    netif::{EspNetif, NetifConfiguration, NetifStack},
    nvs::{EspNvsPartition, NvsDefault},
    sys::EspError,
    timer::EspTaskTimerService,
    wifi::{AsyncWifi, AuthMethod, ClientConfiguration, Configuration, EspWifi},
};
use futures::executor::block_on;

use crate::options;

pub struct NetworkStack {
    wifi: AsyncWifi<EspWifi<'static>>,
}

impl NetworkStack {
    pub fn configure(
        modem: Modem,
        sys_loop: EspSystemEventLoop,
        nvs: EspNvsPartition<NvsDefault>,
    ) -> Result<NetworkStack, EspError> {
        let timer_service = EspTaskTimerService::new()?;

        let (ssid, pass) = options::access_point_credentials();

        let wifi_cfg = Configuration::Client(ClientConfiguration {
            ssid: ssid.into(),
            bssid: None,
            auth_method: AuthMethod::WPA2Personal,
            password: pass.into(),
            channel: None,
        });
        let mut wifi = EspWifi::new(modem, sys_loop.clone(), Some(nvs))?;
        wifi.set_configuration(&wifi_cfg)?;

        if let (Some(device_ip), Some((gateway_ip, gateway_netmask))) = (
            options::get_device_static_ip_addr(),
            options::get_gateway_info(),
        ) {
            let sta_cfg = EspNetif::new_with_conf(&NetifConfiguration {
                key: "euclid0".into(),
                description: "cfg".into(),
                route_priority: 0,
                ip_configuration: IpConfiguration::Client(
                    esp_idf_svc::ipv4::ClientConfiguration::Fixed(ClientSettings {
                        ip: device_ip,
                        subnet: Subnet {
                            gateway: gateway_ip,
                            mask: Mask(gateway_netmask),
                        },
                        dns: None,
                        secondary_dns: None,
                    }),
                ),
                stack: NetifStack::Sta,
                custom_mac: None,
            })?;
            let mut ap_cfg = NetifConfiguration::wifi_default_router();
            ap_cfg.key = "bla0".into();
            let ap_cfg = EspNetif::new_with_conf(&ap_cfg)?;
            let (_, _) = wifi.swap_netif(sta_cfg, ap_cfg)?;
        }
        let wifi = AsyncWifi::wrap(wifi, sys_loop.clone(), timer_service.clone())?;

        Ok(NetworkStack { wifi })
    }

    pub fn start(&mut self) -> Result<(), EspError> {
        block_on(wait_for_wifi_connect(&mut self.wifi))
    }

    pub fn stop(&mut self) -> Result<(), EspError> {
        block_on(wait_for_wifi_disconnect(&mut self.wifi))
    }
}

async fn wait_for_wifi_connect(wifi: &mut AsyncWifi<EspWifi<'static>>) -> Result<(), EspError> {
    wifi.start().await?;
    wifi.connect().await?;
    wifi.wait_netif_up().await?;
    Ok(())
}

async fn wait_for_wifi_disconnect(wifi: &mut AsyncWifi<EspWifi<'static>>) -> Result<(), EspError> {
    wifi.disconnect().await?;
    wifi.stop().await?;
    Ok(())
}
