//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use bsp::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::OutputPin;
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
use rp_pico::hal::Timer;

use midi_convert::midi_types::{Note, Value7};

use bsp::hal::{
    clocks::init_clocks_and_plls,
    pac,
    sio::Sio,
    watchdog::Watchdog,
    usb::UsbBus
};

// Import necessary types from the usb-device crate
use usb_device::bus::UsbBusAllocator;

mod midi_device;
mod drum;

use drum::DrumController;

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let _core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    
    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    
    let mut led_pin = pins.gpio13.into_push_pull_output();
    
    // Load usb bus
    let usb_bus = UsbBusAllocator::new(UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));
    
        
    let device = midi_device::UsbMidiController::new(&usb_bus);
    
    let mut drum = DrumController::new(&timer, device);
    
    // Set pad to note and define as active
    drum.assign(0, Note::A4);
    drum.pad(0).len = 300;
    
    let mut next_toggle = timer.get_counter().ticks() + 500_000;
    let mut led_on = false;
    
    loop {
        // TODO read adcs
        
        // Poll the USB device and MIDI class
        drum.poll();

        let now = timer.get_counter().ticks();
        
        // Heartbeat
        if now >= next_toggle {
            next_toggle += 500_000; // Schedule next toggle in 500 ms
            if led_on {
                led_pin.set_low().unwrap();
            } else {
                info!("on!");
                led_pin.set_high().unwrap();
                // TEMP: trigger automatically
                drum.trigger(0, Value7::from(100));
            }

            led_on = !led_on;
        }
    }
}

// End of file
