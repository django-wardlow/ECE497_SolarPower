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
use embassy_sync::blocking_mutex::CriticalSectionMutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Sender;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{DriveMode, Level, Output, OutputConfig};
use esp_hal::i2c::master::I2c;
use esp_hal::interrupt::InterruptHandler;
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::mcpwm::operator::{DeadTimeCfg, LinkedPins, PwmActions, PwmPinConfig};
use esp_hal::mcpwm::timer::{PwmWorkingMode, Timer, TimerClockConfig};
use esp_hal::mcpwm::{self, McPwm, PeripheralClockConfig};
use esp_hal::peripherals::{self, MCPWM0, Peripherals};
use esp_hal::rmt::{LoopMode, PulseCode, Rmt, TxChannelConfig, TxChannelCreator};
use esp_hal::{Blocking, DriverMode, handler, i2c, main, ram};
use esp_hal::time::{Duration, Rate};
use esp_hal::timer::PeriodicTimer;
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;
#[macro_use(block)]
use nb;
use nb::block;
use pid::Pid;

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]


const max_duty: u16 = 180;
const min_duty: u16 = 20;

pub static PS_CMD: critical_section::Mutex<RefCell<PsCmd>> = critical_section::Mutex::new(RefCell::new(PsCmd{v: 0.0, i:0.0, mppt:false}));


#[derive(Default, Clone, Copy)]
pub struct PsData{
    pub data: [[f32; 4]; 16],
}

#[derive(Default, Clone, Copy)]
pub struct PsCmd{
    pub v: f32,
    pub i: f32, 
    pub mppt: bool
}




pub fn run_ps(i2c: I2c<Blocking>, 
    mut mcpwm_timer: Timer<0, MCPWM0>, 
    pwm_clk: PeripheralClockConfig, 
    mut complementary_pwm: LinkedPins<MCPWM0, 0>, 
    mut sender: embassy_sync::zerocopy_channel::Sender<CriticalSectionRawMutex, PsData>){

    let dead_time = 20;

    
    complementary_pwm.set_falling_edge_deadtime(dead_time);
    complementary_pwm.set_rising_edge_deadtime(dead_time);
    complementary_pwm.set_timestamp_a(100);
    complementary_pwm.set_timestamp_b(100);

    mcpwm_timer.start(pwm_clk.timer_clock_with_frequency(199, PwmWorkingMode::Increase, Rate::from_khz(100)).unwrap());


    let mut adc = Ads1x1x::new_ads1015(i2c, ads1x1x::TargetAddr::Gnd);

    adc.set_data_rate(ads1x1x::DataRate12Bit::Sps3300).unwrap();

    adc.set_full_scale_range(ads1x1x::FullScaleRange::Within2_048V).unwrap();

    // let mut cont_adc = adc.into_continuous().unwrap_or_else(|_| panic!("Something went horribly wrong!"));

    // cont_adc.read();

    let mut target_i = 0.0;
    let mut target_v= 0.0;
    let mut mppt_toggle = false;

    let mut pid_v = Pid::new(0.0, 1.0);
    let mut pid_i =  Pid::new(0.0, 1.0);

    enum Direction{
        Negative,
        Positive
    }

    struct Perturbation{
        direction: Direction,
        magnitude: u16,
        initial_power: f32
    }

    let mut perturbation = Perturbation{direction: Direction::Positive, magnitude: 0, initial_power: 0.0};

    pid_v.p(0.05, 1.0);
    pid_v.i(0.01, 1.0);

    pid_i.p(0.5, 20.0);
    pid_i.i(0.05, 1.0);
    println!("started closed loop ctl");

    let mut buf = [[0.0; 4]; 16];
    let mut index = 0;

    //reading these should be done by an interupt form the ADC and then once all 3 are read we should update the PWM instead of a fixed timer
    loop{

        let a0_raw = block!(adc.read(ads1x1x::channel::SingleA0));
        let a1_raw = block!(adc.read(ads1x1x::channel::SingleA1));
        let a2_raw = block!(adc.read(ads1x1x::channel::SingleA2));

        if let Ok(a0) = a0_raw && let Ok(a1) = a1_raw && let Ok(a2) = a2_raw{

            //read values from ADC. apply adc gain, then offset, then circut gain
            let Iout = ((a0 as f32 / 2048 as f32 * 2.048) - 0.073) * 1.52;

            let Vout = ((a1 as f32 / 2048 as f32 * 2.048) - 0.067) * 6.5;

            let Vin = ((a2 as f32 / 2048 as f32 * 2.048) - 0.067) * 10.0;

            critical_section::with(|cs| {
            let d = PS_CMD.borrow_ref(cs);

            target_v = d.v;
            target_i = d.i;

        });
        

        pid_i.setpoint(target_i);

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

        let duty = (pwm_v_out + ff).min(0.95).max(0.0);

        let mut duty_cycle = (duty * 200.0) as u16;


        // Start mppt here
        if mppt_toggle && Iout < (0.98*target_i) {
            let Iin = (Iout * Vout)/Vin;    // input current
            let power_in = Iin*Vin;         // current power
            
            match (&perturbation.direction, power_in > perturbation.initial_power){
                (Direction::Positive, true) => {
                    perturbation.direction = Direction::Positive;
                }
                (Direction::Positive, false) => {
                    perturbation.direction = Direction::Negative;
                }
                (Direction::Negative, true) => {
                    perturbation.direction = Direction::Negative;
                }
                (Direction::Negative, false) => {
                    perturbation.direction = Direction::Positive;
                }
            }

            // add in logic for dynamically adjusting the magntiude of the perturbation
            perturbation.magnitude = 2;
            perturbation.initial_power = power_in;

            // adjust the duty
        }

        //limit duty cycle to sain values
        // duty_cycle = duty_cycle.min(max_duty).max(min_duty);

        complementary_pwm.set_timestamp_a(duty_cycle);
        complementary_pwm.set_timestamp_b(duty_cycle);


        buf[index] = [Vin, Vout, Iout, duty];
        index += 1;

        if index == buf.len(){

            let data = PsData{
                data: buf
            };

            // println!("buf len is {}", sender.len());

            let buf = sender.try_send();

            if let Some(b) = buf{
                *b = data;

                sender.send_done();
            }
            else {
                println!("buf is full");
            }

            index = 0;

        }

        //println!("IL: {:.4}, Vin: {:.3}, Vout: {:.3}, v targ: {:.3}, load r: {:.3}", Iout, Vin, Vout, pid_v.setpoint, load_r);


        }

        else {
            println!("adc i2c issue!!!!!!!!!!!!!!!!!!!!!!1");
        }    

    }    

}

/*
For maximum power point tracking, we want to wait until the control loop sets the duty
to the next value before perturbing and then observing. We'll perturb and observe after
every cycle that the control loop completes. We should probably change this later. For
now we'll just use really simple logic with a fixed perturbation size. 

the mppt works by checking if the mppt toggle is on and also checking if the output
current is within 2% of the target current. This way, if the output current isn't
close to the target current, we'll actually perturb to try to get more current.
*/
