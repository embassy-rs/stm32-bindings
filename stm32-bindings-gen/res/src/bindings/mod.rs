#[cfg(feature = "wba_wpan_mac")]
pub mod wba_wpan_mac;
#[cfg(feature = "wba_wpan_mac")]
pub use self::wba_wpan_mac as mac_802_15_4;
#[cfg(feature = "wba_wpan_mac")]
pub use self::wba_wpan_mac as wpan_wba;

#[cfg(feature = "wba_wpan_ble")]
pub mod ble_stack;
#[cfg(feature = "wba_wpan_ble")]
pub use self::ble_stack as ble;
#[cfg(feature = "wba_wpan_ble")]
pub use self::ble_stack as ble_wba;
