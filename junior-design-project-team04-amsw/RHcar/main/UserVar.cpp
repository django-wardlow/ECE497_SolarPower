/*
 Author: Django Wardlow

 implements the backend for the user variables for the Rose-Hulman car

 it is unlikely that students should modify this fil

*/

#include "UserVar.h"

//array of all ints and floats so that the dashboard can access them
UserInt* UserInts[16];
UserFloat* UserFloats[16];

//last used index in the arrays
int LastUserInt = -1;
int LastUserFloat = -1;


UserInt::UserInt(const char* name){
  this->name = name;
  this->value = 0;

  //add to array of all ints
  LastUserInt++;

  UserInts[LastUserInt] = this;
}

void UserInt::set(int val){
  this->value = val;
}

int UserInt::get(){
  return this->value;
}

UserFloat::UserFloat(const char* name){
  this->name = name;
  this->value = 0.0;

  //add to array of all floats
  LastUserFloat++;

  UserFloats[LastUserFloat] = this;
}

void UserFloat::set(float val){
  this->value = val;
}

float UserFloat::get(){
  return this->value;
}