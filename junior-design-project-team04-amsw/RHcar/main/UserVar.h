/*
 Author: Django Wardlow

  defined the variables that can be changed from the Rose-Hulman car dashboard

  it is unlikely that students should modify this file
*/
#ifndef UserVar_h

#define UserVar_h

// an integer that that can be read and written to from the web dashboard
class UserInt{

  public:
    UserInt(const char* name);
    
    void set(int value);
    int get();

    const char* name;
    

  private:
    int value;
};

// a float that that can be read and written to from the web dashboard
class UserFloat{

  public:
    UserFloat(const char* name);
    
    void set(float value);
    float get();

    const char* name;

  private:
    
    float value;
};

//have up to 16 ints and 16 floats
extern UserInt* UserInts[16];
extern int LastUserInt;
extern UserFloat* UserFloats[16];
extern int LastUserFloat;


#endif