#![allow(clippy::bad_bit_mask)]
use bitflags::bitflags;
use std::collections::HashMap;
use uuid;
use zbus::zvariant::Value;

bitflags! {
    /// Access point flags for wifi
    pub struct NM80211ApFlags: u32 {
        const NONE    = 0x00000000;
        const PRIVACY = 0x00000001;
    }
}

bitflags! {
    /// Flags for security in wifi access points
    pub struct NM80211ApSecFlags: u32 {
        const NONE             = 0x0000_0000;
        const PAIR_WEP40       = 0x0000_0001;
        const PAIR_WEP104      = 0x0000_0002;
        const PAIR_TKIP        = 0x0000_0004;
        const PAIR_CCMP        = 0x0000_0008;
        const GROUP_WEP40      = 0x0000_0010;
        const GROUP_WEP104     = 0x0000_0020;
        const GROUP_TKIP       = 0x0000_0040;
        const GROUP_CCMP       = 0x0000_0080;
        const KEY_MGMT_PSK     = 0x0000_0100;
        const KEY_MGMT_802_1X  = 0x0000_0200;
    }
}

impl From<NM80211ApSecFlags> for WifiSecurity {
    /// Implements `from` for the wifi security from the bitflags to a more readable enum
    fn from(flags: NM80211ApSecFlags) -> Self {
        if flags.is_empty() {
            WifiSecurity::Open
        } else if flags.intersects(
            NM80211ApSecFlags::PAIR_WEP40
                | NM80211ApSecFlags::PAIR_WEP104
                | NM80211ApSecFlags::GROUP_WEP40
                | NM80211ApSecFlags::GROUP_WEP104,
        ) {
            WifiSecurity::Wep
        } else if flags.intersects(NM80211ApSecFlags::KEY_MGMT_802_1X) {
            WifiSecurity::Enterprise
        } else if flags.intersects(NM80211ApSecFlags::KEY_MGMT_PSK) {
            if flags.contains(NM80211ApSecFlags::PAIR_CCMP) {
                WifiSecurity::Wpa2
            } else {
                WifiSecurity::Wpa
            }
        } else {
            WifiSecurity::Unknown
        }
    }
}

/// Enum for doing security flags for wifi access points more readable and easier to handle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiSecurity {
    Open,
    Wep,
    Wpa,
    Wpa2,
    Enterprise,
    Unknown,
}

impl WifiSecurity {
    fn nm_params(&self) -> (&'static str, &'static str) {
        match self {
            WifiSecurity::Wpa | WifiSecurity::Wpa2 => ("wpa-psk", "psk"),
            WifiSecurity::Wep => ("none", "wep-key0"),
            WifiSecurity::Enterprise => ("wpa-eap", "password"),
            _ => ("none", ""),
        }
    }

    pub fn build_connection_settings<'a>(
        &self,
        ssid: &[u8],
        password: Option<&'a str>,
    ) -> HashMap<&str, HashMap<&str, Value<'a>>> {
        let mut settings = HashMap::new();
        let (key_mgmt, pwd_key) = self.nm_params();

        let mut conn = HashMap::new();
        conn.insert("type", "802-11-wireless".into());
        conn.insert("id", String::from_utf8_lossy(ssid).into_owned().into());

        conn.insert(
            "uuid",
            uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, ssid)
                .to_string()
                .into(),
        );
        settings.insert("connection", conn);

        let mut wifi = HashMap::new();
        wifi.insert("ssid", ssid.to_vec().into());
        settings.insert("802-11-wireless", wifi);

        if *self != WifiSecurity::Open && *self != WifiSecurity::Unknown {
            let mut sec = HashMap::new();
            sec.insert("key-mgmt", key_mgmt.into());
            if let Some(p) = password {
                sec.insert(pwd_key, p.into());
            }
            settings.insert("802-11-wireless-security", sec);
        }

        settings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zbus::zvariant::Value;

    fn get_val<'a>(
        settings: &'a HashMap<&str, HashMap<&str, Value<'static>>>,
        section: &str,
        key: &str,
    ) -> &'a Value<'static> {
        settings
            .get(section)
            .and_then(|s| s.get(key))
            .unwrap_or_else(|| panic!("Missing {}.{}", section, key))
    }

    #[test]
    fn test_stable_uuid_generation() {
        let ssid = b"TestNetwork";
        let settings1 = WifiSecurity::Wpa2.build_connection_settings(ssid, Some("pass"));
        let settings2 = WifiSecurity::Wpa2.build_connection_settings(ssid, Some("pass"));

        let uuid1: String = get_val(&settings1, "connection", "uuid")
            .try_into()
            .unwrap();
        let uuid2: String = get_val(&settings2, "connection", "uuid")
            .try_into()
            .unwrap();

        // Testing the "Better Solution": Same SSID must always produce the same UUID
        assert_eq!(
            uuid1, uuid2,
            "UUIDs should be deterministic for the same SSID"
        );
    }

    #[test]
    fn test_opens_security_omits_security_section() {
        let ssid = b"TestOpenNetwork";
        let settings = WifiSecurity::Open.build_connection_settings(ssid, None);

        assert!(settings.contains_key("connection"));
        assert!(settings.contains_key("802-11-wireless"));
        assert!(!settings.contains_key("802-11-wireless-security"));
    }

    #[test]
    fn test_wpa2_correct_nested_structure() {
        let ssid = b"TestNetwork";
        let settings = WifiSecurity::Wpa2.build_connection_settings(ssid, Some("password"));

        let conn_type: String = get_val(&settings, "connection", "type")
            .try_into()
            .expect("Should be string value");
        assert_eq!(conn_type, "802-11-wireless");

        let ssid_val: Vec<u8> = get_val(&settings, "802-11-wireless", "ssid")
            .clone()
            .downcast::<Vec<u8>>()
            .expect("Should be a Vec<u8>");
        assert_eq!(ssid_val, ssid.to_vec());

        let mgmt: String = get_val(&settings, "802-11-wireless-security", "key-mgmt")
            .try_into()
            .expect("Should be a String");
        let psk: String = get_val(&settings, "802-11-wireless-security", "psk")
            .try_into()
            .expect("Should be a String");

        assert_eq!(mgmt, "wpa-psk");
        assert_eq!(psk, "password");
    }

    #[test]
    fn test_enterprise_uses_correct_keys() {
        let ssid = b"TestNetwork";
        let settings = WifiSecurity::Enterprise.build_connection_settings(ssid, Some("password"));

        let mgmt: String = get_val(&settings, "802-11-wireless-security", "key-mgmt")
            .try_into()
            .expect("Should be able to do as String");
        let pwd: String = get_val(&settings, "802-11-wireless-security", "password")
            .try_into()
            .expect("Should be able to do as String");

        assert_eq!(mgmt, "wpa-eap");
        assert_eq!(pwd, "password");
    }
}
