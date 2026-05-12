/*
 Author: Django Wardlow

  Main file for writing code to control the Rose-Hulman autonomous car. 
  Contains the code that turns inputs from the camera and timer into outputs for the stering and motor

  this is the file the students should work in
*/
#include <Arduino.h>
#include "RHcar.h"
#include "UserVar.h"

const char* car_name = "teamAMSW04-car-ap";


//abstraction for the car hardware
RHcar car;

//the normal speed of the car
UserInt speed("speed");

//the speed of the car when it does not see the line
UserInt slow_speed("no line speed");

//perarmiters for the PID controller
UserFloat Pval("P");
UserFloat Ival("I");
UserFloat Dval("D");

//maximum value for the I term of the PID controller
UserFloat Imax("Imax");

//error as calculated from the tip of the arrow seen by the camera
UserFloat Error1("Error1");

//error as calculated from the base of the arrow seen by the camera
UserFloat Error2("Error2");

//the ratio of Error1 to Error2 to use to get the error value the PID controller will use
UserFloat Errormix("error mix");

//the error value the PID controller uses
UserFloat Error("used error");

//angle speed decrease, controls the amount that the angle of the line seen by the 
// camera relative to the forward direction of the car should reduce the speed of the car
UserFloat ASD("asd");

//The angle of the line seen by the camera relative to the forward direction of the car
UserFloat Angle("Angle");

//the output steering command of the PID controller
UserFloat Out("Out");

//previous error, used to compute D term
float lastError = 0.0;

//total error over time, used to compute the I term
float totalError = 0.0;

RacePhase previousPhase = RacePhase::Stop;


void setup() {

  //initlise the car hardware and web dashboard
  car.init_car(car_name);

  //set default values for user vars
  speed.set(50);

  slow_speed.set(40);

  Pval.set(0.55);
  Ival.set(0.02);
  Dval.set(1);

  Imax.set(20.0);

  Errormix.set(0.5);

  ASD.set(0.1);
 
}

//gets the data from the camera and runs the pid controller, returns the speed the car should go
int driveCar(){

  int power = 0;

  //get data from huskylense (from huskylense example code)
  if (!car.camera.request(1)) {Serial.println(F("Fail to request data from HUSKYLENS, recheck the connection!"));}
  else if(!car.camera.isLearned()) {Serial.println(F("Nothing learned, press learn button on HUSKYLENS to learn one!"));}
  else if(!car.camera.available()) {
    //go to slow speed if we loose the line
    power = slow_speed.get();
  }
  else
  {

    power = speed.get();

    HUSKYLENSResult result = car.camera.read();

    // Calculate the error between the detected line pos and the center of the car
    float error1 = (float)((int32_t)160 - (int32_t)result.xTarget);

    float error2 = (float)((int32_t)160 - (int32_t)result.xOrigin);

    Error1.set(error1);
    
    Error2.set(error2);


    //calculate angle of the line relative to the car
    float adj = result.yOrigin - result.yTarget;

    float opp = result.xOrigin - result.xTarget;

    float angle = std::atan(opp/adj)*57.296;

    Angle.set(angle);

    //reduce motor power based on angle of the line, more angle -> slower movement
    power = power*(1 - (std::abs(angle)/90.0)*ASD.get());

    //compute error value for pid controller based on Error1 and Error2
    float error = error1*Errormix.get() + error2*(1-Errormix.get());

    Error.set(error);

    //calculate P term
    float out = error*Pval.get();

    //calculate D term
    out += Dval.get()*(error - lastError);

    lastError = error;

    //calculate I term
    totalError += error*Ival.get();

    float Ilim = Imax.get();

    //I windup limiter
    if(totalError > Ilim){
      totalError = Ilim;
    }
    else if (totalError < -Ilim){
      totalError = -Ilim;
    }

    out += totalError;

    Out.set(out);
  }

  car.set_stering_angle((Out.get() + 90));

  return power;
  
}

//log a new message every 10 seconds for testing loging
void loop() {

  //get the current time
  int t = millis();

  int power = 0;

  //if the race is running, drive the car
  if(car.RacePhase != RacePhase::Stop){
    power = driveCar();
  }

  //if the car should be moving,, set the power of the car to the power provided by drivecar()
  if(car.RacePhase == RacePhase::First || car.RacePhase == RacePhase::Second){

    car.set_motor_power(power);

  }
  //set the motor to 0 when the car is charging
  else{
      car.set_motor_power(0);

  }

  //log cars current race phase
  if (previousPhase != car.RacePhase){
    previousPhase = car.RacePhase;

    switch (previousPhase) {
      case RacePhase::First :
        car.log("car in first phase of race");
        break;
      case RacePhase::EarlyCharge :
        car.log("car in early charge phase");
        break;
      case RacePhase::Charge :
        car.log("car in first charge phase");
        break;
      case RacePhase::Second :
        car.log("car in second phase of race");
        break;
      case RacePhase::Stop :
        car.log("race is over");
        break;
      default:
        car.log("unknown race phase!!");
    }
  }

  //update the web dashboard
  //must be done frequently (every loop) 
  car.update_dashboard();


  //find how long executing the main loop took
  int time = millis() - t;

  //cam runs at 30 fps so any update rate faster does not make sense
  //dont delay if we spent the entire 33ms frame period executing
  if (time <= 32){
      delay(33 - time);
  }
}
