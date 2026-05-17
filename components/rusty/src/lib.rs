#![no_std]

use crate::sys::esp_idf::{self, wifi_auth_mode_t};
use crate::wifi::UriAuthMode;
use core::ffi::{CStr, c_char};
use core::ptr;

mod wifi;

#[cfg(not(any(unix, windows)))]
mod rt;

/// cbindgen:ignore
mod sys;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TryFromWifiAuthModeError;

/// Converts an ESP-IDF `wifi_auth_mode_t` into a checked `UriAuthMode`.
pub fn try_uri_auth_mode_from_wifi_auth_mode(
    auth_mode: wifi_auth_mode_t,
) -> Result<UriAuthMode, TryFromWifiAuthModeError> {
    match auth_mode {
        esp_idf::wifi_auth_mode_t_WIFI_AUTH_OPEN => Ok(UriAuthMode::Open),
        esp_idf::wifi_auth_mode_t_WIFI_AUTH_WEP => Ok(UriAuthMode::Wep),
        esp_idf::wifi_auth_mode_t_WIFI_AUTH_WPA_PSK
        | esp_idf::wifi_auth_mode_t_WIFI_AUTH_WPA2_PSK
        | esp_idf::wifi_auth_mode_t_WIFI_AUTH_WPA_WPA2_PSK
        | esp_idf::wifi_auth_mode_t_WIFI_AUTH_WPA3_PSK
        | esp_idf::wifi_auth_mode_t_WIFI_AUTH_WPA2_WPA3_PSK
        | esp_idf::wifi_auth_mode_t_WIFI_AUTH_WPA3_ENT_192 => Ok(UriAuthMode::Wpa),
        // Despite not having checked for all defined variants of `wifi_auth_mode_t`, the value
        // might also be an invalid state.
        _ => Err(TryFromWifiAuthModeError),
    }
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_add(left: u64, right: u64) -> u64 {
    add(left, right)
}

/// # Safety
///
/// * `ssid` must be a valid raw C string according to [`core::ffi::CStr::from_ptr`].
/// * `output` must be non-null an valid for writing up to `output_capacity` bytes (including a null
///   termiantor) to.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rusty_generate_wifi_uri(
    ssid: *const c_char,
    auth_mode: wifi_auth_mode_t,
    output: *mut c_char,
    output_capacity: usize,
) -> bool {
    if !ssid.is_null() && !output.is_null() {
        // SAFETY: We checked that `ssid` is not null and placed the requirement of a valid raw C
        // string on the parameter itself.
        let ssid = unsafe { CStr::from_ptr(ssid) };
        let Ok(ssid) = ssid.to_str() else {
            return false;
        };
        let Ok(auth_mode) = try_uri_auth_mode_from_wifi_auth_mode(auth_mode) else {
            return false;
        };
        let Ok(uri) = wifi::uri_from_ssid_and_auth_mode(ssid, auth_mode) else {
            return false;
        };

        let output_bytes = uri.as_str().as_bytes();
        let output_len = output_bytes.len();

        if output_len < output_capacity {
            // SAFETY: We checked that `output` is non-null and placed the requirement of valid
            // memory for `output_capacity` bytes at the parameter.
            unsafe {
                output.write_bytes(0, output_capacity);
                ptr::copy(output_bytes.as_ptr(), output, output_len);
            }
            true
        } else {
            false
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
