#include <cstdint>
/*
 Author: Django Wardlow

RHcar provides an abstraction layer that makes using the Rose-Hulman car simpler to program

It provides a simple API for getting data from the AI camera, steering and driving the car, and reading the power

It also provides a web UI that allows for monitoring and control of the car

it is unlikely that students should modify this file

*/
#ifndef RHcar_h

#define RHcar_h

#include "Dashboard.h"

#include <Wire.h>
#include <Adafruit_INA219.h>
#include "HUSKYLENS.h"
#include <SparkFun_ADS1015_Arduino_Library.h>

//pins for controlling motor and servo, defined by car hardware
const uint8_t servo_pin = 32;
const uint8_t drive_pin = 33;

//used to genorate the PWM signals for the motor and servo
const uint32_t min_duty = 204;

const int charge_pin = 27;

const int balance_pin = 13;

const int chage_indicator = 2;

//the difference in cap voltage that should trigger a balance before charging is nearing completion
const int early_blanace_threshold = 500;

const int balance_time = 60;

const int normal_time = 15;

const int final_charge_threshold = 2850;

const int exit_final_charge_threshold = final_charge_threshold - early_blanace_threshold;

const int done_charge_threshold = 2980;


//defines the phases of the race
enum RacePhase{
  Stop = 0,
  First = 1, 
  EarlyCharge = 3,
  Charge = 4, 
  Second = 5,
};

enum ChargePhase{
  Normal = 0,
  EarlyBalance = 1,
  FinalBlanace = 2,
  Done = 3,
};

class RHcar{

  public:
    RHcar();
    void init_car(const char* name);
    
    void log(String);
    void update_dashboard();

    float get_voltage();
    float get_current();
    float get_watts();
    float get_energy_used();
    float get_energy_remaining();

    void set_motor_power(int pwr);
    void set_stering_angle(int angle);

    int get_race_time();

    HUSKYLENS camera;

    RacePhase RacePhase;

  private:
    Dashboard dashboard;

    //current sensor
    Adafruit_INA219 ina219;

    //ADC
    ADS1015 ads1015;

    float energy;

    float chargeEnergy;

    float initialChargeEnergy;

    int last_charge;

    unsigned long last_energy_time;

    unsigned long last_dashboard_update_time;

    int race_time;

    int balance_timer;

    int nomal_timer;

    ChargePhase charge_phase;

    int16_t adc_voltages[4];

    void set_pwm_out(uint8_t pin, int angle);
    void race_start();
    void race_man_charge();
    void race_stop();

    void read_adc(int charging);

    void charge();

};


#endif
