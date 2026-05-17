#![no_std]

use crate::sys::esp_idf::{self, wifi_auth_mode_t};
use crate::wifi::UriAuthMode;
use core::ffi::{CStr, c_char};
use core::ptr;
use qrcode::{Color, QrCode};

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
/// * `output` must be non-null and valid for writing up to `output_capacity` bytes (including a
///   null terminator) to.
///
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

/// # Safety
///
/// * `ssid` must be a valid raw C string according to [`core::ffi::CStr::from_ptr`].
/// * `pixel_data` must be non-null, properly aligned, and valid for writing up to `pixel_capacity`
///   `u16` values to
/// * `width_and_height` must be non-null, properly aligned, and valid for writing an `usize` value
///   to.
///
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rusty_generate_wifi_qr(
    ssid: *const c_char,
    auth_mode: wifi_auth_mode_t,
    pixel_data: *mut u16,
    pixel_capacity: usize,
    width_and_height: *mut usize,
) -> bool {
    if !ssid.is_null() && !pixel_data.is_null() && !width_and_height.is_null() {
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
        let Ok(code) = QrCode::new(uri) else {
            return false;
        };

        let width = code.width();
        let colors = code.to_colors();

        if let Some(total_pixels) = width.checked_mul(width)
            && total_pixels <= pixel_capacity
        {
            for (index, color) in colors.iter().enumerate() {
                let value = match color {
                    Color::Light => 0xffff,
                    Color::Dark => 0x0000,
                };

                unsafe { pixel_data.add(index).write(value) };
            }

            unsafe { width_and_height.write(width) };
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
