#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use core::cell::{Cell, RefCell};
use core::time;

use ads1x1x::Ads1x1x;
use critical_section::Mutex;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::i2c::master::I2c;
use esp_hal::interrupt::InterruptHandler;
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::mcpwm::operator::{DeadTimeCfg, LinkedPins, PwmActions, PwmPinConfig};
use esp_hal::mcpwm::timer::{PwmWorkingMode, TimerClockConfig};
use esp_hal::mcpwm::{McPwm, PeripheralClockConfig};
use esp_hal::peripherals::MCPWM0;
use esp_hal::rmt::{LoopMode, PulseCode, Rmt, TxChannelConfig, TxChannelCreator};
use esp_hal::{Blocking, handler, i2c, main, ram};
use esp_hal::time::{Duration, Rate};
use esp_hal::timer::PeriodicTimer;
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;
#[macro_use(block)]
use nb;
use nb::block;
use pid::Pid;


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
    pwm: LinkedPins<'static, MCPWM0<'static>, 0>,
    pid_v: Pid<f32>,
    pid_i: Pid<f32>,
    count: usize,
}

struct adc_data{
    // adc: 
    Vin: f32,
    Vout: f32,
    Iout: f32,
}

// static OUT: Mutex<RefCell<Option<Output>>> = Mutex::new(RefCell::new(None));
static CTRL_DATA: Mutex<RefCell<Option<pwm_ctl>>> = Mutex::new(RefCell::new(None));

const max_duty: u16 = 180;
const min_duty: u16 = 20;

const target_output: f32 = 5.0;

static ADC_DATA: Mutex<RefCell<adc_data>> = Mutex::new(RefCell::new(adc_data { Vin: 0.0, Vout: 0.0, Iout: 0.0 }));



#[main]
fn main() -> ! {

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let d2 = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default().with_drive_mode(esp_hal::gpio::DriveMode::PushPull));

    let d4 = Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default().with_drive_mode(esp_hal::gpio::DriveMode::PushPull));
    let d5 = Output::new(peripherals.GPIO5, Level::Low, OutputConfig::default().with_drive_mode(esp_hal::gpio::DriveMode::PushPull));

    let i2c = I2c::new(peripherals.I2C0, i2c::master::Config::default().with_frequency(Rate::from_khz(400))).unwrap().with_sda(peripherals.GPIO22).with_scl(peripherals.GPIO21);

    let mut adc = Ads1x1x::new_ads1015(i2c, ads1x1x::TargetAddr::Gnd);

    adc.set_data_rate(ads1x1x::DataRate12Bit::Sps3300).unwrap();

    adc.set_full_scale_range(ads1x1x::FullScaleRange::Within2_048V).unwrap();

    // let mut cont_adc = adc.into_continuous().unwrap_or_else(|_| panic!("Something went horribly wrong!"));

    // cont_adc.read();


    let control_period_us = 5000;

    let pwm_period_us = 10;

    let dead_time = 20;

    let target_v = 5.0;
    let target_i = 0.5;

    let pwm_clock = PeripheralClockConfig::with_frequency(Rate::from_mhz(40)).unwrap();

    let mut mcpwm = McPwm::new(peripherals.MCPWM0, pwm_clock);

    mcpwm.operator0.set_timer(&mcpwm.timer0);
    let mut complementary_pwm = mcpwm.operator0.with_linked_pins(d4, PwmPinConfig::UP_ACTIVE_HIGH, d5, PwmPinConfig::UP_ACTIVE_HIGH, DeadTimeCfg::new_ahc());
    complementary_pwm.set_falling_edge_deadtime(dead_time);
    complementary_pwm.set_rising_edge_deadtime(dead_time);
    complementary_pwm.set_timestamp_a(100);
    complementary_pwm.set_timestamp_b(100);

    mcpwm.timer0.start(pwm_clock.timer_clock_with_frequency(199, PwmWorkingMode::Increase, Rate::from_khz(100)).unwrap());

    let mut pid_v = Pid::new(target_v, 1.0);
    let mut pid_i =  Pid::new(target_i, 1.0);

    pid_v.p(0.05, 1.0);
    pid_v.i(0.01, 1.0);

    pid_i.p(1.0, 20.0);
    pid_i.i(0.05, 1.0);
    println!("started timer");


    //reading these should be done by an interupt form the ADC and then once all 3 are read we should update the PWM instead of a fixed timer
    loop{

        //read values from ADC. apply adc gain, then offset, then circut gain
        let Iout = ((block!(adc.read(ads1x1x::channel::SingleA0)).unwrap() as f32 / 2048 as f32 * 2.048) - 0.073) * 1.52;

        let Vout = ((block!(adc.read(ads1x1x::channel::SingleA1)).unwrap() as f32 / 2048 as f32 * 2.048) - 0.067) * 6.5;

        let Vin = ((block!(adc.read(ads1x1x::channel::SingleA2)).unwrap() as f32 / 2048 as f32 * 2.048) - 0.067) * 10.0;

        //compute load thevnin resistance v/i = r
        let load_r = Vout/Iout;

        let thev_v = load_r*target_i;

        let current_ctl_v;

        //hack because when measurements are to low thevnin resistance becomes unreliable
        if Iout > 0.05 {

            current_ctl_v = thev_v + pid_i.next_control_output(Iout).output;

        }
        else{
            current_ctl_v = target_v;
        }


        pid_v.setpoint(current_ctl_v.min(target_v).max(0.0));
        

        let pwm_v_out = pid_v.next_control_output(Vout).output;

        //compute feed forward term based in input voltage
        let ff = pid_v.setpoint / Vin;

        let mut duty_cycle = ((pwm_v_out + ff) * 200.0) as u16;

        //limit duty cycle to sain values
        duty_cycle = duty_cycle.min(max_duty).max(min_duty);

        complementary_pwm.set_timestamp_a(duty_cycle);
        complementary_pwm.set_timestamp_b(duty_cycle);


        println!("IL: {:.4}, Vin: {:.3}, Vout: {:.3}, v targ: {:.3}, load r: {:.3}", Iout, Vin, Vout, pid_v.setpoint, load_r);

    }    

}


