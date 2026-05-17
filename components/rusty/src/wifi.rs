use core::fmt::{self, Write};
use heapless::String;

pub enum UriAuthMode {
    Open,
    Wep,
    Wpa,
}

pub fn uri_from_ssid_and_auth_mode(
    ssid: &str,
    auth_mode: UriAuthMode,
) -> Result<String<256>, fmt::Error> {
    let mut uri = String::new();

    let type_ = match auth_mode {
        UriAuthMode::Open => "T:nopass",
        UriAuthMode::Wep => "T:WEP",
        UriAuthMode::Wpa => "T:WPA",
    };

    write!(&mut uri, "WIFI:{type_};S:{ssid};H:false;;")?;

    Ok(uri)
}
