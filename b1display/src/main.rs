//! B1 Display Module
#![no_std]
#![no_main]
#![allow(clippy::needless_range_loop)]

use bsp::entry;
//use defmt::*;
use defmt_rtt as _;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::spi::SpiDevice;
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};

use rp2040_hal::gpio::{FunctionSioOutput, FunctionSpi, Pin, PullDown, PullNone};
//#[cfg(debug_assertions)]
//use panic_probe as _;
use rp2040_panic_usb_boot as _;

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use st7306::{FpsConfig, HpmFps, LpmFps, PowerMode, ST7306};

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use fl16_inputmodules::lcd_hal as bsp;
//use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    gpio, pac,
    sio::Sio,
    usb,
    watchdog::Watchdog,
    Timer,
};
use fugit::RateExtU32;

// USB Device support
use usb_device::descriptor::lang_id::LangID;
use usb_device::device::StringDescriptors;
use usb_device::{class_prelude::*, prelude::*};

// USB Communications Class Device support
use usbd_serial::{SerialPort, USB_CLASS_CDC};

// Used to demonstrate writing formatted strings
use core::fmt::Write;
use heapless::String;

use fl16_inputmodules::control::*;

/// Wrapper around cortex_m::delay::Delay that implements embedded-hal 1.0's DelayNs
struct Delay(cortex_m::delay::Delay);

impl DelayNs for Delay {
    fn delay_ns(&mut self, ns: u32) {
        // Round up to microseconds
        self.0.delay_us(ns.div_ceil(1000));
    }

    fn delay_us(&mut self, us: u32) {
        self.0.delay_us(us);
    }

    fn delay_ms(&mut self, ms: u32) {
        self.0.delay_ms(ms);
    }
}
use fl16_inputmodules::graphics::*;
use fl16_inputmodules::serialnum::{device_release, get_serialnum};

//                            FRA                - Framwork
//                               KDE             - C1 LED Matrix
//                                  AM           - Atemitech
//                                    00         - Default Configuration
//                                      00000000 - Device Identifier
const DEFAULT_SERIAL: &str = "FRAKDEAM0000000000";

type SpiPinout = (
    Pin<gpio::bank0::Gpio19, FunctionSpi, PullNone>, // TX/MOSI
    Pin<gpio::bank0::Gpio16, FunctionSpi, PullNone>, // RX/MISO
    Pin<gpio::bank0::Gpio18, FunctionSpi, PullNone>, // SCK
);

type SpiBus = rp2040_hal::spi::Spi<rp2040_hal::spi::Enabled, pac::SPI0, SpiPinout, 8>;
type CsPin = Pin<gpio::bank0::Gpio17, FunctionSioOutput, PullDown>;

type B1ST7306 = ST7306<
    ExclusiveDevice<SpiBus, CsPin, NoDelay>,
    Pin<gpio::bank0::Gpio20, FunctionSioOutput, PullDown>, // DC
    Pin<gpio::bank0::Gpio21, FunctionSioOutput, PullDown>, // RST
    25,
    200,
>;

const DEBUG: bool = false;
const SCRNS_DELTA: i32 = 5;
const WIDTH: i32 = 300;
const HEIGHT: i32 = 400;
const SIZE: Size = Size::new(WIDTH as u32, HEIGHT as u32);

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    let clocks = init_clocks_and_plls(
        bsp::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = Delay(cortex_m::delay::Delay::new(
        core.SYST,
        clocks.system_clock.freq().to_Hz(),
    ));

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Create timer before USB bus since USB bus moves clocks.usb_clock
    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    // Set up the USB driver
    let usb_bus = UsbBusAllocator::new(usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    // Set up the USB Communications Class Device driver
    let mut serial = SerialPort::new(&usb_bus);

    let serialnum = if let Some(serialnum) = get_serialnum() {
        serialnum.serialnum
    } else {
        DEFAULT_SERIAL
    };

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x32ac, 0x0021))
        .strings(&[StringDescriptors::new(LangID::EN_US)
            .manufacturer("Framework Computer Inc")
            .product("B1 Display")
            .serial_number(serialnum)])
        .unwrap()
        .max_power(500) // TODO: Check how much
        .unwrap()
        .device_release(device_release()) // TODO: Assign dynamically based on crate version
        .device_class(USB_CLASS_CDC)
        .build();

    // Display SPI pins - order is (TX/MOSI, RX/MISO, SCK)
    // Reconfigure pins to PullNone before setting SPI function
    let spi_mosi = pins.sda.reconfigure::<FunctionSpi, PullNone>();
    let spi_miso = pins.miso.reconfigure::<FunctionSpi, PullNone>();
    let spi_sclk = pins.scl.reconfigure::<FunctionSpi, PullNone>();
    let spi = bsp::hal::Spi::<_, _, _, 8>::new(pac.SPI0, (spi_mosi, spi_miso, spi_sclk));
    // Display control pins
    let dc = pins.dc.into_push_pull_output();
    let cs = pins.cs.into_push_pull_output();
    let rst = pins.rstb.into_push_pull_output();

    let spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        16_000_000u32.Hz(),
        embedded_hal::spi::MODE_0,
    );

    // Wrap SPI bus with ExclusiveDevice to get SpiDevice trait
    // ExclusiveDevice manages CS for us
    let spi_device = ExclusiveDevice::new_no_delay(spi, cs).unwrap();

    let mut state = B1DIsplayState {
        sleeping: SimpleSleepState::Awake,
        screen_inverted: false,
        screen_on: true,
        screensaver: Some(ScreenSaverState::default()),
        power_mode: PowerMode::Lpm,
        fps_config: FpsConfig {
            hpm: HpmFps::ThirtyTwo,
            lpm: LpmFps::Two,
        },
        animation_period: 1_000_000, // 1000ms = 1Hz
    };

    const INVERTED: bool = false;
    const AUTO_PWRDOWN: bool = true;
    const TE_ENABLE: bool = true;
    const COL_START: u16 = 0x12;
    const ROW_START: u16 = 0x00;
    let mut disp: B1ST7306 = ST7306::new(
        spi_device,
        dc,
        rst,
        INVERTED,
        AUTO_PWRDOWN,
        TE_ENABLE,
        state.fps_config,
        WIDTH as u16,
        HEIGHT as u16,
        COL_START,
        ROW_START,
    );
    disp.init(&mut delay).unwrap();

    // Clear display, might have garbage in display memory
    // TODO: Seems broken
    //disp.clear(Rgb565::WHITE).unwrap();
    Rectangle::new(Point::new(0, 0), SIZE)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
        .draw(&mut disp)
        .unwrap();

    let logo_rect = draw_logo(&mut disp, Point::new(LOGO_OFFSET_X, LOGO_OFFSET_Y)).unwrap();
    if DEBUG {
        Rectangle::new(Point::new(10, 10), Size::new(10, 10))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
            .draw(&mut disp)
            .unwrap();
        Rectangle::new(Point::new(20, 20), Size::new(10, 10))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
            .draw(&mut disp)
            .unwrap();
        Rectangle::new(Point::new(30, 30), Size::new(10, 10))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
            .draw(&mut disp)
            .unwrap();
        Rectangle::new(Point::new(40, 40), Size::new(10, 10))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
            .draw(&mut disp)
            .unwrap();
        Rectangle::new(Point::new(50, 50), Size::new(10, 10))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
            .draw(&mut disp)
            .unwrap();
        draw_text(
            &mut disp,
            "Framework",
            Point::new(LOGO_OFFSET_X, LOGO_OFFSET_Y + logo_rect.size.height as i32),
        )
        .unwrap();
    }
    disp.flush().unwrap();

    let mut sleep = pins.sleep.into_pull_down_input();

    let mut prev_timer = timer.get_counter().ticks();
    let mut ticks = 0;

    let mut logo_pos = Point::new(LOGO_OFFSET_X, LOGO_OFFSET_Y);

    loop {
        // Go to sleep if the host is sleeping
        let host_sleeping = sleep.is_low().unwrap();
        handle_sleep(host_sleeping, &mut state, &mut delay, &mut disp);

        // Handle period display updates. Don't do it too often
        if timer.get_counter().ticks() > prev_timer + state.animation_period {
            prev_timer = timer.get_counter().ticks();

            if let Some(ref mut screensaver) = state.screensaver {
                let seconds = ticks / (1_000_000 / state.animation_period);
                #[allow(clippy::modulo_one)]
                let second_decimals = ticks % (1_000_000 / state.animation_period);
                Rectangle::new(Point::new(0, 0), Size::new(300, 50))
                    .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
                    .draw(&mut disp)
                    .unwrap();
                let mut text: String<32> = String::new();
                write!(
                    &mut text,
                    "{:>4} Ticks ({:>4}.{} s)",
                    ticks, seconds, second_decimals
                )
                .unwrap();
                // Uncomment to draw the ticks on the screen
                //draw_text(
                //    &mut disp,
                //    &text,
                //    Point::new(0, 0),
                //).unwrap();
                ticks += 1;

                logo_pos = {
                    let (x, y) = (logo_pos.x, logo_pos.y);
                    let w = logo_rect.size.width as i32;
                    let h = logo_rect.size.height as i32;

                    // Bounce off the walls
                    if x <= 0 || x + w >= WIDTH {
                        screensaver.rightwards *= -1;
                    }
                    if y <= 0 || y + h >= HEIGHT {
                        screensaver.downwards *= -1;
                    }

                    Point::new(
                        x + screensaver.rightwards * SCRNS_DELTA,
                        y + screensaver.downwards * SCRNS_DELTA,
                    )
                };
                // Draw a border around the new logo, to clear previously drawn adjacent logos
                let style = PrimitiveStyleBuilder::new()
                    .stroke_color(Rgb565::WHITE)
                    .stroke_width(2 * SCRNS_DELTA as u32)
                    .build();
                Rectangle::new(
                    logo_pos - Point::new(SCRNS_DELTA, SCRNS_DELTA),
                    logo_rect.size + Size::new(2 * SCRNS_DELTA as u32, 2 * SCRNS_DELTA as u32),
                )
                .into_styled(style)
                .draw(&mut disp)
                .unwrap();
                draw_logo(&mut disp, logo_pos).unwrap();
                disp.flush().unwrap();
            }
        }

        // Check for new data
        if usb_dev.poll(&mut [&mut serial]) {
            let mut buf = [0u8; 64];
            match serial.read(&mut buf) {
                Err(_e) => {
                    // Do nothing
                }
                Ok(0) => {
                    // Do nothing
                }
                Ok(count) => {
                    match (parse_command(count, &buf), &state.sleeping) {
                        (Some(Command::Sleep(go_sleeping)), _) => {
                            handle_sleep(go_sleeping, &mut state, &mut delay, &mut disp);
                        }
                        (Some(c @ Command::BootloaderReset), _)
                        | (Some(c @ Command::IsSleeping), _) => {
                            if let Some(response) =
                                handle_command(&c, &mut state, logo_rect, &mut disp, &mut delay)
                            {
                                let _ = serial.write(&response);
                            };
                        }
                        (Some(command), SimpleSleepState::Awake) => {
                            // While sleeping no command is handled, except waking up
                            if let Some(response) = handle_command(
                                &command, &mut state, logo_rect, &mut disp, &mut delay,
                            ) {
                                let _ = serial.write(&response);
                            };
                            // Must write AFTER writing response, otherwise the
                            // client interprets this debug message as the response
                            //let mut text: String<64> = String::new();
                            //write!(
                            //    &mut text,
                            //    "Handled command {}:{}:{}:{}\r\n",
                            //    buf[0], buf[1], buf[2], buf[3]
                            //)
                            //.unwrap();
                            //let _ = serial.write(text.as_bytes());
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn handle_sleep<SPI, DC, RST, DELAY, const COLS: usize, const ROWS: usize>(
    go_sleeping: bool,
    state: &mut B1DIsplayState,
    delay: &mut DELAY,
    disp: &mut ST7306<SPI, DC, RST, COLS, ROWS>,
) where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayNs,
{
    match (state.sleeping.clone(), go_sleeping) {
        (SimpleSleepState::Awake, false) => (),
        (SimpleSleepState::Awake, true) => {
            state.sleeping = SimpleSleepState::Sleeping;

            // Turn off display
            //disp.on_off(false).unwrap();
            disp.sleep_in(delay).unwrap();

            // TODO: Power Display controller down

            // TODO: Set up SLEEP# pin as interrupt and wfi
            //cortex_m::asm::wfi();
        }
        (SimpleSleepState::Sleeping, true) => (),
        (SimpleSleepState::Sleeping, false) => {
            // Restore back grid before sleeping
            state.sleeping = SimpleSleepState::Awake;

            // Turn display back on
            //disp.on_off(true).unwrap();
            disp.sleep_out(delay).unwrap();
            // Sleep-in has to go into HPM first, so we'll be in HPM after wake-up as well
            if state.power_mode == PowerMode::Lpm {
                disp.switch_mode(delay, PowerMode::Lpm).unwrap();
            }

            // Turn screensaver on when resuming from sleep
            // TODO Subject to change, but currently I want to avoid burn-in by default
            state.screensaver = Some(ScreenSaverState::default());

            // TODO: Power display controller back on
        }
    }
}
