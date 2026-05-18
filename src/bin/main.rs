#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use congo_ps::psu_ctl::{PsCmd, PsData};
use defmt::info;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::CriticalSectionMutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::zerocopy_channel::{self, Channel};
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::rng::Rng;
use esp_hal::system::{CpuControl, Stack};
use esp_hal::timer::timg::TimerGroup;
use esp_println as _;

use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::i2c::master::I2c;
use esp_hal::time::{Rate};
use esp_hal::mcpwm::{self, McPwm, PeripheralClockConfig};
use esp_hal::mcpwm::operator::{DeadTimeCfg, LinkedPins, PwmActions, PwmPinConfig};


use esp_backtrace as _;



use congo_ps as lib;


extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

static mut APP_CORE_STACK: Stack<4096> = Stack::new();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.0.0

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[unsafe(link_section = ".dram2_uninit")] size: 98767);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    info!("Embassy initialized!");

    let mut ch_buf = [PsData::default(); 1024];

    let mut ps_data: Channel<CriticalSectionRawMutex, PsData> = zerocopy_channel::Channel::new(&mut ch_buf);

    let (ps_data_tx, ps_data_rx) = ps_data.split();

    let radio_init = &*lib::mk_static!(
        esp_radio::Controller<'static>,
        esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller")
    );
    let rng = Rng::new();

    let stack = lib::wifi::start_wifi(radio_init, peripherals.WIFI, rng, &spawner).await;

    let web_app = lib::web::WebApp::default();
    for id in 0..lib::web::WEB_TASK_POOL_SIZE {
        spawner.must_spawn(lib::web::web_task(
            id,
            stack,
            web_app.router,
            web_app.config,
        ));
    }

    let mut cpu = CpuControl::new(peripherals.CPU_CTRL);

    let d4 = Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default().with_drive_mode(esp_hal::gpio::DriveMode::PushPull));
    let d5 = Output::new(peripherals.GPIO5, Level::Low, OutputConfig::default().with_drive_mode(esp_hal::gpio::DriveMode::PushPull));

    let i2c = I2c::new(peripherals.I2C0, esp_hal::i2c::master::Config::default().with_frequency(Rate::from_khz(400))).unwrap().with_sda(peripherals.GPIO22).with_scl(peripherals.GPIO21);

    let pwm_clock = PeripheralClockConfig::with_frequency(Rate::from_mhz(40)).unwrap();

    let mut mcpwm = McPwm::new(peripherals.MCPWM0, pwm_clock);

    mcpwm.operator0.set_timer(&mcpwm.timer0);

    let mctimer = mcpwm.timer0;

    let cpwm = mcpwm.operator0.with_linked_pins(d4, PwmPinConfig::UP_ACTIVE_HIGH, d5, PwmPinConfig::UP_ACTIVE_HIGH, DeadTimeCfg::new_ahc());


    let second_core = cpu.start_app_core(unsafe { &mut *core::ptr::addr_of_mut!(APP_CORE_STACK) }, move || {lib::psu_ctl::run_ps(i2c, mctimer, pwm_clock, cpwm, ps_data_tx);});

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}
