#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
pub enum DeviceType {
    Unknown = 0,
    Ethernet = 1,
    Wifi = 2,
    Bluetooth = 5,
    OlpcMesh = 6,
    WiMAX = 7,
    Modem = 8,
    Infiniband = 9,
    Bond = 10,
    Vlan = 11,
    ADSL = 12,
    Bridge = 13,
    Team = 15,
    TUN = 16,
    IpTunnel = 17,
    MACVLAN = 18,
    VXLAN = 19,
    VETH = 20,
    Other(u32), // fallback for unknown types
}

impl From<u32> for DeviceType {
    fn from(val: u32) -> Self {
        match val {
            0 => DeviceType::Unknown,
            1 => DeviceType::Ethernet,
            2 => DeviceType::Wifi,
            5 => DeviceType::Bluetooth,
            6 => DeviceType::OlpcMesh,
            7 => DeviceType::WiMAX,
            8 => DeviceType::Modem,
            9 => DeviceType::Infiniband,
            10 => DeviceType::Bond,
            11 => DeviceType::Vlan,
            12 => DeviceType::ADSL,
            13 => DeviceType::Bridge,
            15 => DeviceType::Team,
            16 => DeviceType::TUN,
            17 => DeviceType::IpTunnel,
            18 => DeviceType::MACVLAN,
            19 => DeviceType::VXLAN,
            20 => DeviceType::VETH,
            other => DeviceType::Other(other),
        }
    }
}
