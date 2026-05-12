#include "esp32-hal-gpio.h"
/*
 Author: Django Wardlow

  Contains the code that implements the hardware abstractions for the capacitor bank, INA192, pwm outputs, and camera on the Rose-Hulman car

  it is unlikely that students should modify this file
*/

#include <sys/_types.h>
#include "esp32-hal.h"
#include <cstdint>
#include <Wire.h>
#include "HardwareSerial.h"
#include "RHcar.h"
#include <Arduino.h>
#include <cmath>


RHcar::RHcar(){
  this->energy = 0;
  this->last_charge = 0;
  this->initialChargeEnergy = 0;
  this->chargeEnergy = 0;
  this->RacePhase = RacePhase::Stop;
}

///initlise the car hardware
void RHcar::init_car(const char* name){

  //set up pins used for charging
  pinMode(balance_pin, OUTPUT);
  digitalWrite(balance_pin, LOW);

  pinMode(chage_indicator, OUTPUT);
  digitalWrite(chage_indicator, LOW);

  pinMode(charge_pin, INPUT_PULLUP);

  //start serial for debugging
  Serial.begin(115200);

  this->race_time = 0;

  this->dashboard.init_dashboard(name, [&](){this->race_start();}, [&](){this->race_man_charge();}, [&](){this->race_stop();});

  //init pwm out for servo and esc
  ledcAttach(servo_pin, 50, 12);
  ledcAttach(drive_pin, 50, 12);

  //set outputs to nominal values
  set_pwm_out(servo_pin, 90);
  set_pwm_out(drive_pin, 0);

  //set up ADC measuring cap voltages
  this->ads1015.begin(0x48);
  this->ads1015.setGain(ADS1015_CONFIG_PGA_1);
  this->ads1015.setSampleRate(ADS1015_CONFIG_RATE_1600HZ);
  this->ads1015.useConversionReady(true);

  // init AI cmera
  Wire.begin();

  while (!this->camera.begin(Wire))
  {
      Serial.println(F("camera init failed!"));
      Serial.println(F("1.Please recheck the \"Protocol Type\" in HUSKYLENS (General Settings>>Protol Type>>I2C)"));
      Serial.println(F("2.Please recheck the connection."));
      delay(100);
  }

  this->camera.writeAlgorithm(ALGORITHM_LINE_TRACKING); //Switch the algorithm to line tracking.

  //init current sensor
  while (!ina219.begin()) {
    Serial.println("Failed to connect to INA219 chip");
  }

  //update time for energy calc
  this->last_dashboard_update_time = millis();
  this->last_energy_time = millis();


  Serial.println("car initlisation complete!");
}

//reads the cap voltages from the ADC
void RHcar::read_adc(int charging){

  int16_t adc0, adc1, adc2, adc3;

  //only read extra voltages if we are charging as each voltage takes ~10ms to read
  if(charging){
    adc0 = this->ads1015.getSingleEndedSigned(0);
    adc1 = this->ads1015.getSingleEndedSigned(1);
    adc2 = this->ads1015.getSingleEndedSigned(2);

    this->adc_voltages[0] = (adc0*2)*1;
    this->adc_voltages[1] = (adc1*2)*2;
    this->adc_voltages[2] = (adc2*2)*3;
  }

  adc3 = this->ads1015.getSingleEndedSigned(3);

  this->adc_voltages[3] = (adc3*2)*4;

  // if (charging) {
  //   Serial.println("charging!");
  // }
  // else{
  //   Serial.println("not charging!");
  // }

  // Serial.print("V0: "); Serial.println(this->adc_voltages[0]);
  // Serial.print("V1: "); Serial.println(this->adc_voltages[1]);
  // Serial.print("V2: "); Serial.println(this->adc_voltages[2]);
  // Serial.print("V3: "); Serial.println(this->adc_voltages[3]);

  // Serial.println(" ");

  // Serial.print("C0: "); Serial.println(this->adc_voltages[0]);
  // Serial.print("C1: "); Serial.println(this->adc_voltages[1] - this->adc_voltages[0]);
  // Serial.print("C2: "); Serial.println(this->adc_voltages[2] - this->adc_voltages[1]);
  // Serial.print("C3: "); Serial.println(this->adc_voltages[3] - this->adc_voltages[2]);

}

//controls the relays that balance the caps
void RHcar::charge(){

  switch (this->charge_phase) {
    case ChargePhase::Normal:
      {

        //compute the voltage of each of the 4 caps
        int16_t c1 = this->adc_voltages[0];
        int16_t c2 = this->adc_voltages[1] - this->adc_voltages[0];
        int16_t c3 = this->adc_voltages[2] - this->adc_voltages[1];
        int16_t c4 = this->adc_voltages[3] - this->adc_voltages[2];

        //find the min and max cap voltage
        int16_t max = std::max(c1, std::max(c2, std::max(c3, c4)));
        int16_t min = std::min(c1, std::min(c2, std::min(c3, c4)));

        int16_t dif = max-min;

        this->nomal_timer -= 1;

        //balance if caps are to out of balance
        if((dif > early_blanace_threshold) && (this->nomal_timer <= 0)){
          this->balance_timer = balance_time;
          this->charge_phase = ChargePhase::EarlyBalance;
        }
        //switch to final charge mode if close to done charging
        else if(max >= final_charge_threshold){
          this->charge_phase = ChargePhase::FinalBlanace;
        }
      }
    break;

    case ChargePhase::EarlyBalance:
      {
        //switch caps to parallel
        digitalWrite(balance_pin, HIGH);

        int c = this->adc_voltages[0];

        this->balance_timer -= 1;

        //stop balancing if timer expired
        if(this -> balance_timer <= 0){
          digitalWrite(balance_pin, LOW);
          this->nomal_timer = normal_time;
          this->charge_phase = ChargePhase::Normal;
        }

        //switch to final charge mode if above threshold voltage
        else if(c >= final_charge_threshold){
          this->charge_phase = ChargePhase::FinalBlanace;
        }
      }
    break;

    case ChargePhase::FinalBlanace:
      {
        //switch caps to parallel
        digitalWrite(balance_pin, HIGH);

        int c = this->adc_voltages[0];

        //switch to done when we reach the fully charged voltage
        if(c >= done_charge_threshold){
          digitalWrite(balance_pin, LOW);
          this->charge_phase = ChargePhase::Done;
        }
        else if (c <= exit_final_charge_threshold) {
          digitalWrite(balance_pin, LOW);
          this->charge_phase = ChargePhase::Normal;
        }
      }
    break;

    case ChargePhase::Done:
      //switch caps to series
      digitalWrite(balance_pin, LOW);

      //turn on blue led to indicate charging is done
      digitalWrite(chage_indicator, HIGH);

    break;

    //in the case that we are in a unknown state, switch caps to series and go to normal charge mode
    default:
      digitalWrite(balance_pin, LOW);
      this->charge_phase = ChargePhase::Normal;
  }

}

void RHcar::update_dashboard(){

  //check if the charger is plugged in
  int charging = digitalRead(charge_pin);

  //if the charger goes from unplugged to plugged, store the voltage to use for charge energy calculations
  if(this->last_charge != charging){
    if(charging){
      this->initialChargeEnergy = pow(this->adc_voltages[3]/1000.0, 2)*30*0.5;
    }

    this->last_charge = charging;
  }

  //compute the energy we have gained form charging
  if(charging){
    float finalenergy = pow(this->adc_voltages[3]/1000.0, 2)*30*0.5;
    this->chargeEnergy = finalenergy - this->initialChargeEnergy;
  }

  //read the cap voltag(s)
  this->read_adc(charging);

  //if we are charging, run balancing logic
  //otherwise, reset charge logic and put caps in series
  if(charging){
    this->charge();
  }
  else{
    this->balance_timer = 0;
    this->nomal_timer = normal_time;
    this->charge_phase = ChargePhase::Normal;
    digitalWrite(chage_indicator, LOW);
    digitalWrite(balance_pin, LOW);
  }

  //get current power draw from INA sensor
  float w = this->get_watts();

  //numerical integration of power to get energy
  unsigned long time = millis();
  float deltaTime = (time-(this->last_energy_time))/1000.0;

  //accumulate energy used only while car is running
  if(this->RacePhase != RacePhase::Stop){
      this->energy += w*deltaTime;
  }

  this->last_energy_time = time;

  //update race data every 100 ms
  if((time - this->last_dashboard_update_time) > 100){
    this->last_dashboard_update_time = time;

    this->race_time = this->dashboard.update_race_data();

    //update rache phase based on race timer
    if(this->race_time > 90+45+45){
      this->RacePhase = RacePhase::Stop;
    }
    else if(this->race_time > 90+45 && this->RacePhase != RacePhase::Stop){
      this->RacePhase = RacePhase::Second;
    }
    else if (this->race_time > 90 && this->RacePhase != RacePhase::Stop){
      this->RacePhase = RacePhase::Charge;
    }

  }

  //update power data on the dashboard
  this->dashboard.update_power(this->adc_voltages[3]/1000.0, this->get_current(), w, this->energy, this->chargeEnergy, this->get_energy_remaining());


  //update user vars
  this->dashboard.update_user_vars();
  
}

void RHcar::race_start(){
  this->energy = 0;
  this->RacePhase = RacePhase::First;
  this->race_time = 0;
  this->chargeEnergy = 0;
}

void RHcar::race_man_charge(){
  // Serial.println("race manual charge!");
  this->RacePhase = RacePhase::EarlyCharge;
}

void RHcar::race_stop(){
    this->RacePhase = RacePhase::Stop;
}

int RHcar::get_race_time(){
    return this->race_time;
}

///gets the ammount of current the car is drawing in milliamps
float RHcar::get_current(){
  return this->ina219.getCurrent_mA();
}

///gets the voltage seen by the car
float RHcar::get_voltage(){
  return this->adc_voltages[3]/1000.0;
}

///gets the power in milliwatts that the car is using
float RHcar::get_watts(){
  return this->ina219.getPower_mW();
}

//gets the energy used during the race in jules
float RHcar::get_energy_used(){
  return this->energy;
}

//gets the approximant usable energy remaining in the capacitors
float RHcar::get_energy_remaining(){
  //calculate energy in caps - energy stored at 5V as the DCDC shuts down at ~5v
  return (pow(this->adc_voltages[3]/1000.0, 2)*30*0.5) - 375.0;
}

//sets the drive motor power to the car. 0 is off and 180 is full power
void RHcar::set_motor_power(int pwr){
  this->set_pwm_out(drive_pin, pwr);
}

//sets the car stering angle. 0 is full left, 90 is center, and 180 is full right
void RHcar::set_stering_angle(int pwr){
  this->set_pwm_out(servo_pin, pwr);
}

///sets the pwm output for a given pin, used to controll the motor and servo
void RHcar::set_pwm_out(uint8_t pin, int angle){
  angle = constrain(angle, 0, 180);

  uint32_t duty = min_duty + (uint8_t)(((float)angle/180.0)*(float)min_duty);
  
  ledcWrite(pin, duty);

}

///logs a message to the web dashboard
void RHcar::log(String msg){
  this->dashboard.log(msg);
}
