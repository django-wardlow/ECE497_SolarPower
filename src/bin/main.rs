#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use core::cell::RefCell;
use core::time;

use critical_section::Mutex;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::interrupt::InterruptHandler;
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::mcpwm::operator::{DeadTimeCfg, LinkedPins, PwmActions, PwmPinConfig};
use esp_hal::mcpwm::timer::{PwmWorkingMode, TimerClockConfig};
use esp_hal::mcpwm::{McPwm, PeripheralClockConfig};
use esp_hal::peripherals::MCPWM0;
use esp_hal::rmt::{LoopMode, PulseCode, Rmt, TxChannelConfig, TxChannelCreator};
use esp_hal::{Blocking, handler, main, ram};
use esp_hal::time::{Duration, Rate};
use esp_hal::timer::PeriodicTimer;
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]

struct pwm_ctl{
    duty_cycle: u16,
    ctl_timer: PeriodicTimer<'static, Blocking>,
    pwm: LinkedPins<'static, MCPWM0<'static>, 0>
}

// static OUT: Mutex<RefCell<Option<Output>>> = Mutex::new(RefCell::new(None));
static PWM: Mutex<RefCell<Option<pwm_ctl>>> = Mutex::new(RefCell::new(None));


#[main]
fn main() -> ! {

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let d2 = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default().with_drive_mode(esp_hal::gpio::DriveMode::PushPull));

    let d4 = Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default().with_drive_mode(esp_hal::gpio::DriveMode::PushPull));
    let d5 = Output::new(peripherals.GPIO5, Level::Low, OutputConfig::default().with_drive_mode(esp_hal::gpio::DriveMode::PushPull));


    let control_period_us = 50000;

    let pwm_period_us = 10;

    let dead_time = 20;

    let pwm_clock = PeripheralClockConfig::with_frequency(Rate::from_mhz(40)).unwrap();

    let mut mcpwm = McPwm::new(peripherals.MCPWM0, pwm_clock);

    mcpwm.operator0.set_timer(&mcpwm.timer0);
    let mut complementary_pwm = mcpwm.operator0.with_linked_pins(d4, PwmPinConfig::UP_ACTIVE_HIGH, d5, PwmPinConfig::UP_ACTIVE_HIGH, DeadTimeCfg::new_ahc());
    complementary_pwm.set_falling_edge_deadtime(dead_time);
    complementary_pwm.set_rising_edge_deadtime(dead_time);
    complementary_pwm.set_timestamp_a(100);
    complementary_pwm.set_timestamp_b(100);

    mcpwm.timer0.start(pwm_clock.timer_clock_with_frequency(199, PwmWorkingMode::Increase, Rate::from_khz(100)).unwrap());

    let mut control_timer = PeriodicTimer::new(TimerGroup::new(peripherals.TIMG0).timer0);

    control_timer.set_interrupt_handler(handler);

    control_timer.start(Duration::from_micros(control_period_us)).unwrap();

    critical_section::with(|cs|{
        control_timer.listen();
        let mut pwm = pwm_ctl{
            duty_cycle: 0,
            ctl_timer: control_timer,
            pwm: complementary_pwm,
        };
        PWM.borrow_ref_mut(cs).replace(pwm);
    });

    println!("started timer");

    loop{

    }    

}

#[handler]
#[ram]
fn handler() {

    critical_section::with(|cs|{
        let mut pwm_ctl_mutex = PWM.borrow_ref_mut(cs);
        let pwm = pwm_ctl_mutex.as_mut().unwrap();

        pwm.duty_cycle += 1;
        pwm.duty_cycle %= 200;

        pwm.pwm.set_timestamp_a(pwm.duty_cycle);
        pwm.pwm.set_timestamp_b(pwm.duty_cycle);

        pwm.ctl_timer.clear_interrupt();
        
    });

}
