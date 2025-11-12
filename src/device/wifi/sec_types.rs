#![allow(clippy::bad_bit_mask)]
use bitflags::bitflags;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiSecurity {
    Open,
    Wep,
    Wpa,
    Wpa2,
    Enterprise,
    Unknown,
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
