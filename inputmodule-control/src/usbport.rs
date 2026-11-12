//! Where a serial device is plugged in, to put several LED matrices in a stable order.
//!
//! COM port numbers and /dev/ttyACM* names depend on enumeration order, and not
//! every module reports a serial number. But the USB port of each input deck slot
//! is fixed, and on the Framework 16 a lower hub port is further left.

/// USB location of a serial device, ending in root port and hub port,
/// like [.., 4, 2] for root port 4, hub port 2.
/// None if it can't be found out on this platform.
pub fn location(port_name: &str) -> Option<Vec<u32>> {
    imp::location(port_name)
}

/// Framework 16 input deck slots from left to right, as (root port, hub port).
/// The left two are behind one hub, the right two behind another.
const FW16_SLOTS: [(u32, u32); 4] = [(4, 2), (4, 3), (3, 2), (3, 3)];

/// Position from the left of a module in the Framework 16 input deck
pub fn fw16_position(location: &[u32]) -> Option<usize> {
    let [.., root, port] = location else {
        return None;
    };
    FW16_SLOTS.iter().position(|&slot| slot == (*root, *port))
}

/// Position from the left of a serial device in the Framework 16 input deck
pub fn fw16_position_of(port_name: &str) -> Option<usize> {
    fw16_position(&location(port_name)?)
}

/// Numbers in a Linux USB device name, like [1, 4, 2] for "1-4.2"
#[allow(dead_code)]
fn numbers(s: &str) -> Vec<u32> {
    s.split(|c: char| !c.is_ascii_digit())
        .filter(|n| !n.is_empty())
        .filter_map(|n| n.parse().ok())
        .collect()
}

/// Hex numbers in a Windows location path, like [0, 0x803, 0, 0, 4, 2] for
/// "PCIROOT(0)#PCI(0803)#PCI(0000)#USBROOT(0)#USB(4)#USB(2)"
#[allow(dead_code)]
fn path_numbers(path: &str) -> Option<Vec<u32>> {
    path.split('#')
        // Interface of a composite device, not part of the port path
        .filter(|seg| !seg.starts_with("USBMI("))
        .map(|seg| {
            let n = seg.split_once('(')?.1.strip_suffix(')')?;
            u32::from_str_radix(n, 16).ok()
        })
        .collect()
}

#[cfg(target_os = "linux")]
mod imp {
    /// /sys/class/tty/ttyACM0/device links to the USB interface, like
    /// .../usb1/1-4/1-4.2/1-4.2:1.0, whose parent is the USB device at 1-4.2.
    pub fn location(port_name: &str) -> Option<Vec<u32>> {
        let name = port_name.rsplit('/').next()?;
        let dev = std::fs::canonicalize(format!("/sys/class/tty/{}/device", name)).ok()?;
        let usb_dev = dev.parent()?.file_name()?.to_str()?;
        Some(super::numbers(usb_dev))
    }
}

#[cfg(windows)]
mod imp {
    use std::ptr;
    use windows_sys::Win32::Devices::DeviceAndDriverInstallation::{
        SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo, SetupDiGetClassDevsW,
        SetupDiGetDeviceRegistryPropertyW, SetupDiOpenDevRegKey, DICS_FLAG_GLOBAL, DIGCF_PRESENT,
        DIREG_DEV, GUID_DEVCLASS_PORTS, SPDRP_LOCATION_PATHS, SP_DEVINFO_DATA,
    };
    use windows_sys::Win32::System::Registry::{RegCloseKey, RegQueryValueExW, KEY_READ};

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(Some(0)).collect()
    }

    fn from_wide(buf: &[u16]) -> String {
        let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        String::from_utf16_lossy(&buf[..end])
    }

    /// Goes through the present devices of the Ports class to find the one with
    /// this COM name and reads its location, like "Port_#0002.Hub_#0008".
    pub fn location(port_name: &str) -> Option<Vec<u32>> {
        unsafe {
            let set = SetupDiGetClassDevsW(&GUID_DEVCLASS_PORTS, ptr::null(), 0, DIGCF_PRESENT);
            if set == -1 {
                return None;
            }
            let mut result = None;
            let mut index = 0;
            loop {
                let mut info: SP_DEVINFO_DATA = std::mem::zeroed();
                info.cbSize = std::mem::size_of::<SP_DEVINFO_DATA>() as u32;
                if SetupDiEnumDeviceInfo(set, index, &mut info) == 0 {
                    break;
                }
                index += 1;

                let key = SetupDiOpenDevRegKey(set, &info, DICS_FLAG_GLOBAL, 0, DIREG_DEV, KEY_READ);
                if key == -1 {
                    continue;
                }
                let mut name = [0u16; 64];
                let mut size = std::mem::size_of_val(&name) as u32;
                let found = RegQueryValueExW(
                    key,
                    wide("PortName").as_ptr(),
                    ptr::null(),
                    ptr::null_mut(),
                    name.as_mut_ptr() as *mut u8,
                    &mut size,
                ) == 0
                    && from_wide(&name).eq_ignore_ascii_case(port_name);
                RegCloseKey(key);
                if !found {
                    continue;
                }

                // Like PCIROOT(0)#PCI(0803)#PCI(0000)#USBROOT(0)#USB(4)#USB(2), which is
                // fixed by the hardware, unlike the hub number Windows assigns
                let mut loc = [0u16; 512];
                if SetupDiGetDeviceRegistryPropertyW(
                    set,
                    &info,
                    SPDRP_LOCATION_PATHS,
                    ptr::null_mut(),
                    loc.as_mut_ptr() as *mut u8,
                    std::mem::size_of_val(&loc) as u32,
                    ptr::null_mut(),
                ) != 0
                {
                    // First of the list of paths is the PCI one
                    result = super::path_numbers(&from_wide(&loc));
                }
                break;
            }
            SetupDiDestroyDeviceInfoList(set);
            result
        }
    }
}

#[cfg(not(any(target_os = "linux", windows)))]
mod imp {
    pub fn location(_port_name: &str) -> Option<Vec<u32>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{fw16_position, numbers, path_numbers};

    #[test]
    fn fw16_slots() {
        let win = |p| path_numbers(p).unwrap();
        let left = win("PCIROOT(0)#PCI(0803)#PCI(0000)#USBROOT(0)#USB(4)#USB(2)");
        let right = win("PCIROOT(0)#PCI(0803)#PCI(0000)#USBROOT(0)#USB(3)#USB(3)");
        assert_eq!(fw16_position(&left), Some(0));
        assert_eq!(fw16_position(&right), Some(3));
        assert_eq!(fw16_position(&numbers("1-4.3")), Some(1));
        assert_eq!(fw16_position(&numbers("1-3.2")), Some(2));
        // Keyboard interface, not a slot for an LED matrix in this position
        let kbd = win("PCIROOT(0)#PCI(0803)#PCI(0000)#USBROOT(0)#USB(3)#USB(1)#USBMI(0)");
        assert_eq!(fw16_position(&kbd), None);
        assert_eq!(fw16_position(&[]), None);
    }

    #[test]
    fn location_numbers() {
        assert_eq!(numbers("1-4.2"), vec![1, 4, 2]);
        assert_eq!(numbers(""), Vec::<u32>::new());
        assert_eq!(
            path_numbers("PCIROOT(0)#PCI(0803)#PCI(0000)#USBROOT(0)#USB(4)#USB(2)"),
            Some(vec![0, 0x803, 0, 0, 4, 2])
        );
        assert_eq!(path_numbers("ACPI(_SB_)"), None);
    }
}
