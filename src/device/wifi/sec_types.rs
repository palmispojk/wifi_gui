#![allow(clippy::bad_bit_mask)]
use bitflags::bitflags;
use std::str::FromStr;
use zbus::zvariant::{Dict, Signature, Value};

bitflags! {
    /// access point flags for wifi
    pub struct NM80211ApFlags: u32 {
        const NONE    = 0x00000000;
        const PRIVACY = 0x00000001;
    }
}

bitflags! {
    /// flags for security in wifi access points
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
    pub fn to_nm_dict<'a>(&self, password: Option<&'a str>) -> Dict<'a, 'a> {
        fn make_dict<'a>(
            key_mgmt: &'a str,
            pwd_key: &'a str,
            password: Option<&'a str>,
        ) -> Dict<'a, 'a> {
            let sig = Signature::from_str("v").expect("Signature for v should be possible.");
            let mut dict = Dict::new(&Signature::from_str("s").unwrap(), &sig);
            dict.add("key-mgmt", Value::from(key_mgmt))
                .expect("Adding mgmt key to inner dict should not throw error!");
            if let Some(pwd) = password {
                dict.add(pwd_key, Value::from(pwd))
                    .expect("adding password key should not throw error when adding to dict!");
            }
            dict
        }
        match self {
            WifiSecurity::Open | WifiSecurity::Unknown => Dict::new(
                &Signature::from_str("s").unwrap(),
                &Signature::from_str("a{sv}").unwrap(),
            ),

            WifiSecurity::Wep => make_dict("none", "wep-key0", password),

            WifiSecurity::Wpa | WifiSecurity::Wpa2 => make_dict("wpa-psk", "psk", password),

            WifiSecurity::Enterprise => make_dict("wpa-eap", "password", password),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zbus::zvariant::{Dict, Value};

    fn dict_keys(dict: &Dict<'_, '_>) -> Vec<String> {
        dict.iter()
            .map(|(k, _)| k.to_string().trim_matches('"').to_string())
            .collect()
    }

    fn dict_get_string(dict: &Dict, key: &str) -> String {
        dict.get::<&str, Value>(&key)
            .unwrap_or_else(|_| panic!("Missing key: {}", key))
            .unwrap()
            .clone()
            .try_into()
            .unwrap_or_else(|_| panic!("Value for {} is not a string", key))
    }

    #[test]
    fn test_open_security_empty_dict() {
        let dict = WifiSecurity::Open.to_nm_dict(None);

        assert_eq!(dict.iter().count(), 0);
    }

    #[test]
    fn test_unknown_security_empty_dict() {
        let dict = WifiSecurity::Unknown.to_nm_dict(None);

        assert_eq!(dict.iter().count(), 0);
    }

    #[test]
    fn test_wep_correct_dict() {
        let dict = WifiSecurity::Wep.to_nm_dict(Some("test"));

        let keys = dict_keys(&dict);

        assert_eq!(keys.len(), 2);

        assert!(keys.contains(&"key-mgmt".into()));
        assert!(keys.contains(&"wep-key0".into()));

        assert_eq!(dict_get_string(&dict, "key-mgmt"), "none");

        assert_eq!(dict_get_string(&dict, "wep-key0"), "test");
    }

    #[test]
    fn tets_wpa_correct_dict() {
        let dict = WifiSecurity::Wpa.to_nm_dict(Some("test"));

        let keys = dict_keys(&dict);
        assert_eq!(keys.len(), 2);

        assert!(keys.contains(&"key-mgmt".into()));
        assert!(keys.contains(&"psk".into()));

        assert_eq!(dict_get_string(&dict, "key-mgmt"), "wpa-psk");
        assert_eq!(dict_get_string(&dict, "psk"), "test");
    }

    #[test]
    fn test_wpa2_correct_dict() {
        let dict = WifiSecurity::Wpa2.to_nm_dict(Some("test"));

        let keys = dict_keys(&dict);
        assert_eq!(keys.len(), 2);

        assert!(keys.contains(&"key-mgmt".into()));
        assert!(keys.contains(&"psk".into()));

        assert_eq!(dict_get_string(&dict, "key-mgmt"), "wpa-psk");
        assert_eq!(dict_get_string(&dict, "psk"), "test");
    }

    #[test]
    fn test_enterprise_correct_dict() {
        let dict = WifiSecurity::Enterprise.to_nm_dict(Some("test"));

        let keys = dict_keys(&dict);
        assert_eq!(keys.len(), 2);

        assert!(keys.contains(&"key-mgmt".into()));
        assert!(keys.contains(&"password".into()));

        assert_eq!(dict_get_string(&dict, "key-mgmt"), "wpa-eap");
        assert_eq!(dict_get_string(&dict, "password"), "test");
    }
}
