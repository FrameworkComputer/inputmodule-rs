//! USB reset interface used by picotool to enter the RP2040 bootloader.
//!
//! Adapted from https://github.com/ithinuel/usbd-picotool-reset (v0.3.0).

// MIT License
//
// Copyright (c) 2023 rp-rs organization
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use core::marker::PhantomData;
use usb_device::{class_prelude::*, control, LangID};

const RESET_REQUEST_BOOTSEL: u8 = 0x01;

pub struct PicoToolReset<B: UsbBus> {
    interface: InterfaceNumber,
    string: StringIndex,
    _bus: PhantomData<B>,
}

impl<B: UsbBus> PicoToolReset<B> {
    pub fn new(allocator: &UsbBusAllocator<B>) -> Self {
        Self {
            interface: allocator.interface(),
            string: allocator.string(),
            _bus: PhantomData,
        }
    }

    fn is_reset_interface(&self, request: &control::Request) -> bool {
        request.request_type == control::RequestType::Class
            && request.recipient == control::Recipient::Interface
            && request.index == u16::from(u8::from(self.interface))
    }
}

impl<B: UsbBus> UsbClass<B> for PicoToolReset<B> {
    fn get_configuration_descriptors(
        &self,
        writer: &mut DescriptorWriter,
    ) -> usb_device::Result<()> {
        // Pico SDK reset interface: vendor class, subclass 0, protocol 1.
        writer.interface_alt(self.interface, 0, 0xff, 0, 1, Some(self.string))
    }

    fn get_string(&self, index: StringIndex, _lang_id: LangID) -> Option<&str> {
        (index == self.string).then_some("Reset")
    }

    fn control_out(&mut self, transfer: ControlOut<B>) {
        let request = transfer.request();
        if !self.is_reset_interface(request) {
            return;
        }
        if request.request != RESET_REQUEST_BOOTSEL {
            let _ = transfer.reject();
            return;
        }

        // Bit 8 enables the activity LED; bits 9..13 select its GPIO.
        let gpio_mask = if request.value & 0x100 != 0 {
            1 << (request.value >> 9)
        } else {
            0
        };
        // Forward the bootloader interface flags supplied by picotool.
        rp2040_hal::rom_data::reset_to_usb_boot(gpio_mask, u32::from(request.value & 0x7f));
        unreachable!();
    }

    fn control_in(&mut self, transfer: ControlIn<B>) {
        if self.is_reset_interface(transfer.request()) {
            let _ = transfer.reject();
        }
    }
}
