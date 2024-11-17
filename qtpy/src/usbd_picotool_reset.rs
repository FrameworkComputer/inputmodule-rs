//! From https://github.com/ithinuel/usbd-picotool-reset
//! UsbClass implementation for the picotool reset feature.
//!
//! ## Note
//!
//! For picotool to recognize your device, your device must be using Raspberry Pi's vendor ID (`0x2e8a`)
//! and one of the product ID. You can check [picotool's sources](https://github.com/raspberrypi/picotool/blob/master/picoboot_connection/picoboot_connection.c#L23-L27)
//! for an exhaustive list.

#![forbid(missing_docs)]
// #![no_std]

use core::marker::PhantomData;
use usb_device::class_prelude::{InterfaceNumber, StringIndex, UsbBus, UsbBusAllocator};
//use usb_device::LangID;
use usb_device::descriptor::lang_id;

// Vendor specific class
const CLASS_VENDOR_SPECIFIC: u8 = 0xFF;
// cf: https://github.com/raspberrypi/pico-sdk/blob/f396d05f8252d4670d4ea05c8b7ac938ef0cd381/src/common/pico_usb_reset_interface/include/pico/usb_reset_interface.h#L17
const RESET_INTERFACE_SUBCLASS: u8 = 0x00;
const RESET_INTERFACE_PROTOCOL: u8 = 0x01;
const RESET_REQUEST_BOOTSEL: u8 = 0x01;
//const RESET_REQUEST_FLASH: u8 = 0x02;

/// Defines which feature of the bootloader are made available after reset.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DisableInterface {
    /// Both Mass Storage and Pico boot are enabled.
    None,
    /// Disables Mass Storage leaving only PicoBoot.
    DisableMassStorage,
    /// Disables PicoBoot leaving only Mass Storage.
    DisablePicoBoot,
}
impl DisableInterface {
    const fn into(self) -> u32 {
        match self {
            DisableInterface::None => 0,
            DisableInterface::DisableMassStorage => 1,
            DisableInterface::DisablePicoBoot => 2,
        }
    }
}

/// Allows to customize the configuration of the UsbClass.
pub trait Config {
    /// Configuration for which interface to enable/disable after reset.
    const INTERFACE_DISABLE: DisableInterface;
    /// Configuration for which pin to show mass storage activity after reset.
    const BOOTSEL_ACTIVITY_LED: Option<usize>;
}

/// Default configuration for PicoTool class.
///
/// This lets both interface enabled after reset and does not display mass storage activity on any
/// LED.
pub enum DefaultConfig {}
impl Config for DefaultConfig {
    const INTERFACE_DISABLE: DisableInterface = DisableInterface::None;

    const BOOTSEL_ACTIVITY_LED: Option<usize> = None;
}

/// UsbClass implementation for Picotool's reset feature.
pub struct PicoToolReset<'a, B: UsbBus, C: Config = DefaultConfig> {
    intf: InterfaceNumber,
    str_idx: StringIndex,
    _bus: PhantomData<&'a B>,
    _cnf: PhantomData<C>,
}
impl<'a, B: UsbBus, C: Config> PicoToolReset<'a, B, C> {
    /// Creates a new instance of PicoToolReset.
    pub fn new(alloc: &'a UsbBusAllocator<B>) -> PicoToolReset<'a, B, C> {
        Self {
            intf: alloc.interface(),
            str_idx: alloc.string(),
            _bus: PhantomData,
            _cnf: PhantomData,
        }
    }
}

impl<B: UsbBus, C: Config> usb_device::class::UsbClass<B> for PicoToolReset<'_, B, C> {
    fn get_configuration_descriptors(
        &self,
        writer: &mut usb_device::descriptor::DescriptorWriter,
    ) -> usb_device::Result<()> {
        writer.interface_alt(
            self.intf,
            0,
            CLASS_VENDOR_SPECIFIC,
            RESET_INTERFACE_SUBCLASS,
            RESET_INTERFACE_PROTOCOL,
            Some(self.str_idx),
        )
    }

    fn get_string(&self, index: StringIndex, _lang_id: u16) -> Option<&str> {
        (index == self.str_idx).then_some("Reset")
    }

    fn control_out(&mut self, xfer: usb_device::class_prelude::ControlOut<B>) {
        let req = xfer.request();
        if !(req.request_type == usb_device::control::RequestType::Class
            && req.recipient == usb_device::control::Recipient::Interface
            && req.index == u8::from(self.intf) as u16)
        {
            return;
        }

        match req.request {
            RESET_REQUEST_BOOTSEL => {
                let mut gpio_mask = C::BOOTSEL_ACTIVITY_LED.map(|led| 1 << led).unwrap_or(0);
                if req.value & 0x100 != 0 {
                    gpio_mask = 1 << (req.value >> 9);
                }
                rp2040_hal::rom_data::reset_to_usb_boot(
                    gpio_mask,
                    u32::from(req.value & 0x7F) | C::INTERFACE_DISABLE.into(),
                );
                // no-need to accept/reject, we'll reset the device anyway
                unreachable!()
            }
            //RESET_REQUEST_FLASH => todo!(),
            _ => {
                // we are not expecting any other USB OUT requests
                let _ = xfer.reject();
            }
        }
    }

    fn control_in(&mut self, xfer: usb_device::class_prelude::ControlIn<B>) {
        let req = xfer.request();
        if !(req.request_type == usb_device::control::RequestType::Class
            && req.recipient == usb_device::control::Recipient::Interface
            && req.index == u8::from(self.intf) as u16)
        {
            return;
        }
        // we are not expecting any USB IN requests
        let _ = xfer.reject();
    }
}
