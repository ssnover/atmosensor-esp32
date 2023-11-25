use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{modem::Modem, peripherals::Peripherals},
    ipv4::{ClientSettings, Configuration as IpConfiguration, Ipv4Addr, Mask, Subnet},
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

        let sta_cfg = EspNetif::new_with_conf(&NetifConfiguration {
            key: "euclid0".into(),
            description: "cfg".into(),
            route_priority: 0,
            ip_configuration: IpConfiguration::Client(
                esp_idf_svc::ipv4::ClientConfiguration::Fixed(ClientSettings {
                    ip: Ipv4Addr::new(192, 168, 5, 151),
                    subnet: Subnet {
                        gateway: Ipv4Addr::new(192, 168, 4, 1),
                        mask: Mask(22),
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
        let wifi_cfg = Configuration::Client(ClientConfiguration {
            ssid: ssid.into(),
            bssid: None,
            auth_method: AuthMethod::WPA2Personal,
            password: pass.into(),
            channel: None,
        });
        let mut wifi = EspWifi::new(modem, sys_loop.clone(), Some(nvs))?;
        wifi.set_configuration(&wifi_cfg)?;
        let (_, _) = wifi.swap_netif(sta_cfg, ap_cfg)?;
        println!("Connecting");
        let mut wifi = AsyncWifi::wrap(wifi, sys_loop.clone(), timer_service.clone())?;

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
