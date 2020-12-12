
// TO Understand render see the example in Fundamentals: minimal/render.cpp




#include <Bela.h>
#include <libraries/Midi/Midi.h>
#include <stdlib.h>
#include <cmath>


#include <chrono>



// For calling URLs ///
#include <stdio.h>
#include <string.h>
#include <fstream>
///////////////////////



#include <iostream>





// ...

using namespace std::chrono;
milliseconds ms = duration_cast< milliseconds >(
    system_clock::now().time_since_epoch()
);

/////////////


//#define BitClear(x, bitPosition)\
//    ((x) &= ~(1 << (bitPosition)))


/////////////////////////////////////////////////


/*
 ____  _____ _        _    
| __ )| ____| |      / \   
|  _ \|  _| | |     / _ \  
| |_) | |___| |___ / ___ \ 
|____/|_____|_____/_/   \_\
The platform for ultra-low latency audio and sensor processing
http://bela.io
A project of the Augmented Instruments Laboratory within the
Centre for Digital Music at Queen Mary University of London.
http://www.eecs.qmul.ac.uk/~andrewm
(c) 2016 Augmented Instruments Laboratory: Andrew McPherson,
  Astrid Bin, Liam Donovan, Christian Heinrichs, Robert Jack,
  Giulio Moro, Laurel Pardue, Victor Zappi. All rights reserved.
The Bela software is distributed under the GNU Lesser General Public License
(LGPL 3.0), available here: https://www.gnu.org/licenses/lgpl-3.0.txt
*/
#include <Bela.h>
#include <stdlib.h> //random
#include <math.h> //sinf
#include <time.h> //time
#include <libraries/OscillatorBank/OscillatorBank.h>
const float kMinimumFrequency = 20.0f;
const float kMaximumFrequency = 8000.0f;
int gSampleCount;               // Sample counter for indicating when to update frequencies
float gNewMinFrequency;
float gNewMaxFrequency;
// Task for handling the update of the frequencies using the analog inputs
AuxiliaryTask gFrequencyUpdateTask;
// These settings are carried over from main.cpp
// Setting global variables is an alternative approach
// to passing a structure to userData in setup()
int gNumOscillators = 2; // was 500
int gWavetableLength = 1024;
void recalculate_frequencies(void*);
OscillatorBank osc;


int gAudioChannelNum; // number of audio channels to iterate over
int gAnalogChannelNum; // number of analog channels to iterate over


const int SEQUENCE_OUT_PIN = 5;



// Set the analog channels to read from
//int gSensorInputFrequency = 0;
const int CLOCK_INPUT_ANALOG_IN_PIN = 0;



// Salt Pinouts salt pinouts are here: https://github.com/BelaPlatform/Bela/wiki/Salt

// T2 (Trigger 2) is Physical Channel / Pin 14 

// T1 in is	digital channel 15
const int CLOCK_INPUT_DIGITAL_PIN = 15;


// T1 out	digital channel 0
// T2 out	digital channel 5

const int CLOCK_OUTPUT_DIGITAL_PIN = 0;



// CV I/O 1-8	analog channel 0-7
const int SEQUENCE_CV_IN_PIN = 0;


////////////////////////////////////////////////

// Setup pins
const uint8_t teensy_led_pin = 13;
const uint8_t audio1OutPin = 22; 
const uint8_t euroshield_button_pin = 2;

const uint8_t euroshield_led_pin_count = 4;
const uint8_t euroshieldLedPins[euroshield_led_pin_count] = { 6, 5, 4, 3 }; // { 3, 4, 5, 6 }; 

// This the the pin for the upper pot on the Euroshield
const uint8_t upper_pot_pin = 20;
// This the the pin for the upper pot on the Euroshield
const uint8_t lower_pot_pin = 21;


const uint8_t BRIGHT_0 = 0;
const uint8_t BRIGHT_1 = 10;
const uint8_t BRIGHT_2 = 20;
const uint8_t BRIGHT_3 = 75;
const uint8_t BRIGHT_4 = 100;
const uint8_t BRIGHT_5 = 255;

// Use zero based index for sequencer. i.e. step_count for the first step is 0.
const uint8_t FIRST_STEP = 0;
const uint8_t MAX_STEP = 15;

const uint8_t MIN_SEQUENCE_LENGTH_IN_STEPS = 1; // ONE INDEXED
const uint8_t MAX_SEQUENCE_LENGTH_IN_STEPS = 16; // ONE INDEXED

///////////////////////

const int CV_WAVEFORM_B_FREQUENCY_RAW_MAX_INPUT = 1023;


const uint8_t MIDI_NOTE_ON = 1;
const uint8_t MIDI_NOTE_OFF = 0;


////////////////////////////////////////////////////
// Actual pot values
unsigned int upper_input_raw; // TODO Make t type.
unsigned int lower_input_raw;


// Create 4 virtual pots out of two pots and a button.
// To handle the case when 1) Pot is fully counter clockwise 2) We press the button 3) Move the pot fully clockwise 4) Release the button.
// We introduce the concept that a virtual pot can be "engaged" or not so we can catchup with its stored value only when the pot gets back to that position.
//bool upper_pot_high_engaged = true;
unsigned int binary_sequence_input_raw;
unsigned int binary_sequence_input = 20;
unsigned int binary_sequence_input_last;
unsigned int binary_sequence_input_at_button_change;

//bool lower_pot_high_engaged = true;
unsigned int lfo_a_frequency_input_raw;
unsigned int lfo_a_frequency_input = 20;
unsigned int lfo_a_frequency_input_last;
unsigned int lfo_a_frequency_input_at_button_change;

//bool upper_pot_low_engaged = true;
unsigned int sequence_length_input_raw;
unsigned int sequence_length_input = 20;
unsigned int sequence_length_input_last;
unsigned int sequence_length_input_at_button_change;

//bool lower_pot_low_engaged = true;
unsigned int lfo_b_frequency_input_raw;
unsigned int lfo_b_frequency_input = 20;
unsigned int lfo_b_frequency_input_last;
unsigned int lfo_b_frequency_input_at_button_change;


int new_digital_clock_in_state;
int current_digital_clock_in_state;
float analog_clock_in_level;
float right_peak_level;

float external_modulator_object_level;


////////////////////////////////////////////////////


////////////////////////////////////////////////////
// Musical parameters that the user can tweak.

uint8_t sequence_length_in_steps_raw;


// The Primary GATE sequence pattern // Needs to be upto 16 bits. Maybe more later.
unsigned int binary_sequence_result;
unsigned int gray_code_sequence;
unsigned int the_sequence;
unsigned int last_binary_sequence_result; // So we can detect changes

// Sequence Length (and default)
uint8_t sequence_length_in_steps = 8; 


bool do_tick = true;

// Used to control when/how we change sequence length 
uint8_t new_sequence_length_in_ticks; 

// Just counts 0 to 5 within each step
uint8_t ticks_after_step;

// Jitter Reduction: Used to flatten out glitches from the analog pots. 
// Actually we like the glitches - it makes the sequencer more interesting


uint8_t NO_JITTER_REDUCTION = 0;

uint8_t jitter_reduction = NO_JITTER_REDUCTION;


uint8_t FUZZINESS_AMOUNT = 100;

// LFO
unsigned int cv_waveform_a_frequency_raw;
float cv_waveform_a_frequency;

  bool reset_cv_lfo_at_FIRST_STEP = false;

// Amplitude of the LFO
unsigned int cv_waveform_a_amplitude_raw;
float cv_waveform_a_amplitude;


bool analog_clock_in_state = LOW;

bool midi_clock_detected = LOW;
bool sequence_is_running = LOW;

////////////////////////////////////
// Store extra data about the note (velocity, "exactly" when in a step etc)
// Note name (number) and step information is stored in the array below.         
class NoteInfo
{
 public:
   uint8_t velocity = 0 ;
   uint8_t tick_count_in_sequence = 0;
   uint8_t is_active = 0;
};
/////////

////////
// For each sequence step / midi note number  / on-or-off we store a NoteInfo (which defines a bit more info)
// Arrays are ZERO INDEXED but here we define the SIZE of each DIMENSION of the Array.)
// This way we can easily access a step and the notes there.
// [step][midi_note][on-or-off]
// [step] will store a digit between 0 and 15 to represent the step of the sequence.
// [midi_note] will store between 0 and 127
// [on-or-off] will store either 1 for MIDI_NOTE_ON or 0 for MIDI_NOTE_OFF
NoteInfo channel_a_midi_note_events[MAX_STEP+1][128][2]; 
////////




// "Ghost notes" are created to cancel out a note-off in channel_a_midi_note_events that is created when during the note off of low velocity notes.
class GhostNote
{
 public:
   uint8_t tick_count_in_sequence = 0;
   uint8_t is_active = 0;
};

GhostNote channel_a_ghost_events[128];

////////////////////////////////////////
// Bit Constants for bit wise operations 


 
uint8_t sequence_bits_8_through_1 = 128 + 64 + 32 + 16 + 8 + 4 + 2 + 1;

uint8_t bits_2_1 = 2 + 1; // CV lfo shape
// how long the CV pulse will last for in terms of ticks
uint8_t cv_waveform_b_frequency_bits_4_3_2_1 = 8 + 4 + 2 + 1; 

uint8_t cv_waveform_a_amplitude_bits_8_7_6_5 = 128 + 64 + 32 + 16;
///////////////////////////////////////////////////////////////////    





///////////////////////////
// Values set via the Pots (and later maybe audio inputs).
unsigned int cv_waveform_b_frequency_raw = 0;
float cv_waveform_b_frequency = 0;
float cv_waveform_b_amplitude;
float cv_waveform_b_amplitude_delta;
unsigned int cv_waveform_b_shape;


// Timing 
// count the ticks (24 per quarter note / crotchet since the last crotchet or start)

struct Timing
{
    uint8_t tick_count_in_sequence = 0;  
    int tick_count_since_start = 0; 
};

// Timing is controlled by the loop. Only the loop should update it.
Timing loop_timing;

// Count of the main pulse i.e. sixteenth notes or eigth notes 
uint8_t step_count;

// Helper functions that operate on global variables. Yae!  

void SetTickCountInSequence(uint8_t value){
  loop_timing.tick_count_in_sequence = value;
}

void SetTotalTickCount(int value){
  loop_timing.tick_count_since_start = value;
}

void ResetSequenceCounters(){
  SetTickCountInSequence(0);
  step_count = FIRST_STEP; 
  //rt_printf(String("ResetSequenceCounters Done. sequence_length_in_steps is ") + sequence_length_in_steps + String(" step_count is now: ") + step_count);
}


uint8_t StepCountSanity(uint8_t step_count_){
  uint8_t step_count_fixed;
  
  if (step_count_ > MAX_STEP){
    rt_printf("**** ERROR step_count_ > MAX_STEP i.e. %f \n" , step_count_ );
    step_count_fixed = MAX_STEP;
  } else if (step_count_ < FIRST_STEP){
    rt_printf("**** ERROR step_count_ > FIRST_STEP i.e. %f \n", step_count_ );
    step_count_fixed = FIRST_STEP;
  } else {
    step_count_fixed = step_count_;
  }
  return step_count_fixed;
}




uint8_t IncrementStepCount(){
  step_count = StepCountSanity(step_count + 1);

  rt_printf("IncrementStepCount. sequence_length_in_steps is: %f step_count is now: %f ", sequence_length_in_steps, step_count);
  return StepCountSanity(step_count);
}



float gInterval = 0.5;
float gSecondsElapsed = 0;
int gCount = 0;


// void pass_string_2(const std::string&){
// 	 if(gCount % 100000 == 0) {
// 			rt_printf("%s",s); 
	
// 			rt_printf("Debug Print gCount %d \n",gCount);
// 	 }	
// }



void pass_string(char s[])	 
{  
	 if(gCount % 100000 == 0) {
			rt_printf("%s",s); 
	 }
} 



void debugPrint(char a, char b, char c, int x, int y, int z ){
	
    if(gCount % 100000 == 0) {
    	//rt_printf("OK \n");
    	
    //	rt_printf("Debug Print %s%s%s : %s %s %s",a,b,c,x,y,z);
		rt_printf("Debug Print gCount %d \n",gCount);	
	}
	


} 


void printStatus(){

    // We don't want to print every time else we overload the CPU
    gCount++;
	
    if(gCount % 1000 == 0) {
      rt_printf("================ \n");
	  rt_printf("printStatus says gCount is: %d \n",gCount);


      rt_printf("binary_sequence_input_raw is: %d \n", binary_sequence_input_raw);
	  rt_printf("binary_sequence_input is: %d \n", binary_sequence_input);
      rt_printf("sequence_length_input is: %d \n", sequence_length_input);
      rt_printf("lfo_a_frequency_input is: %d \n", lfo_a_frequency_input);
      rt_printf("lfo_b_frequency_input is: %d \n", lfo_b_frequency_input);

      rt_printf("analog_clock_in_state is: %d \n", analog_clock_in_state);

      rt_printf("current_digital_clock_in_state is: %d \n", current_digital_clock_in_state);
      rt_printf("new_digital_clock_in_state is: %d \n", new_digital_clock_in_state);


     // rt_printf("loop_timing.tick_count_in_sequence is: %d \n", loop_timing.tick_count_in_sequence);
     // rt_printf("loop_timing.tick_count_since_start is: %d \n", loop_timing.tick_count_since_start);

      rt_printf("binary_sequence_result is: %d \n", binary_sequence_result);


      rt_printf("the_sequence is: %d \n", the_sequence);

      rt_printf("sequence_is_running is: %d \n", sequence_is_running);


      rt_printf("midi_clock_detected is: %d \n", midi_clock_detected);





      rt_printf("================ \n");

        char url[1000] = "https://apisandbox.openbankproject.com/obp/v4.0.0/adapter";

        // char url[1000] = "https://apisandbox.openbankproject.com/obp/v4.0.0/root";


//char result[1000]


    std::fstream fs;
    fs.open(url);


//char c = fs.get();
//rt_printf("-%c", c);

// while ( getline (fs,line) ){
//       //use line here
//       rt_printf("-%c", line);
//     }

  // while (fs.good()) {
  //   rt_printf("-%c", c);
  //   c = fs.get();
  // }



// Thanks to: https://gehrcke.de/2011/06/reading-files-in-c-using-ifstream-dealing-correctly-with-badbit-failbit-eofbit-and-perror/

std::string line;
std::string error_opening;
std::ifstream f (url);
if (!f.is_open())
	//std::cout << perror("Cannot open file");
	//std::error >> error_opening;
	
	std::cerr << "Could not open file...\n";
	std::cerr << url << std::endl;
	std::cerr << strerror(errno) << std::endl;
	
	
	//std::cerr << "LARA";
//	std::cerr >> error_opening;
	
//	stderr << strerror(errno) << std::endl;
	
	//error_opening = perror("lalala");	
    //rt_printf("error opening %s", &error_opening);
    
    //rt_printf("error opening %s", stderr);
    
    
    // perror("error while opening file");
while(getline(f, line)) {
    rt_printf("-%c", &line);
    //process(&line);
    }
if (f.bad())
    rt_printf("error reading ");
    //perror("error while reading file");





    fs.close();



//Serial.print("\t");
   //Serial.print(the_sequence, BIN);
   //Serial.println();



      
	}
	


} 




void GateHigh(BelaContext *context){
  //rt_printf("Gate HIGH at tick_count_since_start: %d ", loop_timing.tick_count_since_start);
  
  for(unsigned int n = 0; n < context->digitalFrames; ++n){
			digitalWrite(context, n, SEQUENCE_OUT_PIN, HIGH);
	}
}

void GateLow(BelaContext *context){
  //rt_printf("Gate LOW");
  
  for(unsigned int n = 0; n < context->digitalFrames; ++n){
			digitalWrite(context, n, SEQUENCE_OUT_PIN, LOW);
	}
}

bool RampIsPositive(){
	// TODO BELA
	return false;
  //if (cv_waveform_b_amplitude_delta > 0)
  //{
  //  return true;
  //} 
  //else 
  //{
  //  return false;
  //}
  
}

// Kind of syncs the CV 
void SyncAndResetCv(){
  //Serial.println(String("CV Pulse On") );
   
  
  
  // TODO set bela oscilator
  //cv_waveform_a_object.phase(90); // Sine wave has maximum at 90 degrees
  
    // Used to modulate CV. This signal is multiplied by cv_waveform 

  // Allow the amplitude to fall to zero before we lift it back up. (if it indeed gets to zero)
  
  if (RampIsPositive()){
    if (cv_waveform_b_amplitude == 1) {
      cv_waveform_b_amplitude = 0;
      rt_printf("SyncAndResetCv : 0");
      
      // TODO BELA SetWaveformBObjectAmplitude ();
    }

  } else {
    if (cv_waveform_b_amplitude == 0) {
      cv_waveform_b_amplitude = 1;

       rt_printf("SyncAndResetCv : 1");
      
      // TODO BELA SetWaveformBObjectAmplitude ();
    }
  }
}



// Return bth bit of number from https://stackoverflow.com/questions/2249731/how-do-i-get-bit-by-bit-data-from-an-integer-value-in-c
uint8_t ReadBit (int number, int b ){
	(number & ( 1 << b )) >> b;
}






/////////////////////////////////////////////////////////////
// These are the possible beats of the sequence
void OnStep(BelaContext *context){

  

  rt_printf("OnStep: %d \n", step_count);

  if (step_count > MAX_STEP) {
    rt_printf("----------------------------------------------------------------------------\n");  
    rt_printf("------------------ ERROR! step_count is: %s --- ERROR ---\n", + step_count);
    rt_printf("----------------------------------------------------------------------------\n");    
  }

  

    if (step_count == FIRST_STEP) {
      SyncAndResetCv();
    } else {
      // TODO BELA ChangeCvWaveformBAmplitude();
    }
  
  
  step_count = StepCountSanity(step_count);
  
  
  uint8_t play_note = (the_sequence & ( 1 << step_count )) >> step_count;  
  
  // Why does the line below trigger "Xenomai/cobalt: watchdog triggered" whereas the same logic in this function does not?
  //uint8_t play_note = ReadBit(the_sequence, step_count);
  
   if (play_note){
     rt_printf("****************** play \n");
    GateHigh(context); 
   } else {
    GateLow(context);
     rt_printf("not play \n");
   }

   
      
}



// These are ticks which are not steps - so in between possible beats.
void OnNotStep(BelaContext *context){
  //rt_printf("NOT step_countIn is: ") + step_countIn  ); 
  // TODO not sure how this worked before. function name? ChangeCvWaveformBAmplitude(); 
  GateLow(context);
  
}






milliseconds last_clock_pulse=milliseconds();








// Midi clock and start / stop related
//MIDI_CREATE_INSTANCE(HardwareSerial, Serial1, MIDI);






//////////////////////////////
// Bela Specific
Midi midi;

//const char* gMidiPort0 = "hw:1,0,0";


// To find midi ports Bela can see, type "amidi -l" in the Bela command line.

const char* gMidiPort0 = "hw:0,0";


float gFreq;
float gPhaseIncrement = 0;
bool gIsNoteOn = 0;
int gVelocity = 0;
float gSamplingPeriod = 0;
//int gSampleCount = 44100; // how often to send out a control change


float gPhase;
float gInverseSampleRate;
int gAudioFramesPerAnalogFrame = 0;





/*
 * This callback is called every time a new input Midi message is available
 *
 * Note that this is called in a different thread than the audio processing one.
 *
 */
void midiMessageCallback(MidiChannelMessage message, void* arg){
	if(arg != NULL){
		rt_printf("Message from midi port %s ", (const char*) arg);
	}
	message.prettyPrint();
	if(message.getType() == kmmNoteOn){
		gFreq = powf(2, (message.getDataByte(0) - 69) / 12.f) * 440.f;
		gVelocity = message.getDataByte(1);
		gPhaseIncrement = 2.f * (float)M_PI * gFreq * gSamplingPeriod;
		gIsNoteOn = gVelocity > 0;
		rt_printf("v0:%f, ph: %6.5f, gVelocity: %d\n", gFreq, gPhaseIncrement, gVelocity);
	} else {


// Midi clock  (decimal 248, hex 0xF8)


		int byte0 = message.getDataByte(0);
		int byte1 = message.getDataByte(1);
    int type = message.getType();

		rt_printf("type: %d byte0: %d  byte1 : %d \n", type, byte0, byte1);


    do_tick = true;

    //OnTick();


  }




  
}



//////////////


// That will clear the nth bit of number. You must invert the bit string with the bitwise NOT operator (~), then AND it.
int BitClear (unsigned int number, unsigned int n) {
number &= ~(1UL << n);
}






// Here follows some used and abused code:


/////////////////////////////////////////////
/////////////////////////////////////////////
// https://playground.arduino.cc/Main/Fscale/
// Floating Point Autoscale Function V0.1
// Paul Badger 2007
// Modified from code by Greg Shakar
float fscale( float originalMin, float originalMax, float newBegin, float
newEnd, float inputValue, float curve){

  float OriginalRange = 0;
  float new_range = 0;
  float zeroRefCurVal = 0;
  float normalizedCurVal = 0;
  float rangedValue = 0;
  bool invFlag = 0;


  // condition curve parameter
  // limit range

  if (curve > 10) curve = 10;
  if (curve < -10) curve = -10;

  curve = (curve * -.1) ; // - invert and scale - this seems more intuitive - postive numbers give more weight to high end on output
  curve = pow(10, curve); // convert linear scale into lograthimic exponent for other pow function

  /*
   Serial.println(curve * 100, DEC);   // multply by 100 to preserve resolution  
   Serial.println();
   */

  // Check for out of range inputValues
  if (inputValue < originalMin) {
    inputValue = originalMin;
  }
  if (inputValue > originalMax) {
    inputValue = originalMax;
  }

  // Zero Refference the values
  OriginalRange = originalMax - originalMin;

  if (newEnd > newBegin){
    new_range = newEnd - newBegin;
  }
  else
  {
    new_range = newBegin - newEnd;
    invFlag = 1;
  }

  zeroRefCurVal = inputValue - originalMin;
  normalizedCurVal  =  zeroRefCurVal / OriginalRange;   // normalize to 0 - 1 float

  /*
  Serial.print(OriginalRange, DEC);  
   Serial.print("   ");  
   Serial.print(new_range, DEC);  
   Serial.print("   ");  
   Serial.println(zeroRefCurVal, DEC);  
   Serial.println();  
   */

  // Check for originalMin > originalMax  - the math for all other cases i.e. negative numbers seems to work out fine
  if (originalMin > originalMax ) {
    return 0;
  }

  if (invFlag == 0){
    rangedValue =  (pow(normalizedCurVal, curve) * new_range) + newBegin;

  }
  else     // invert the ranges
  {  
    rangedValue =  newBegin - (pow(normalizedCurVal, curve) * new_range);
  }

  return rangedValue;
}
///////////////////////////////////////////////////////////////


///////////////////////////////////////////////////////////////
// From https://gist.github.com/shirish47/d21b896570a8fccbd9c3
unsigned int Binary2Gray(unsigned int data)
 {
   unsigned int n_data=(data>>1);
   n_data=(data ^ n_data);
   
  return n_data;
 }
///////////////////////////////////////////////////////////////


// linear conversion https://stackoverflow.com/questions/929103/convert-a-number-range-to-another-range-maintaining-ratio

float linearScale( float original_range_min, float original_range_max, float new_range_min, float new_range_max, float original_value){

float original_range = (original_range_max - original_range_min);

//Serial.println(String("linearScale original_range ") + original_range );

float new_range = (new_range_max - new_range_min);  
//Serial.println(String("linearScale new_range ") + new_range );

float new_value = (((original_value - original_range_min) * new_range) / original_range) + new_range_min;

//Serial.println(String("linearScale new_value ") + new_value );

return new_value;
}




void Led1Level(uint8_t level){
 // TODO! analogWrite(euroshieldLedPins[0], level);
}

void Led2Level(uint8_t level){
 // TODO! analogWrite(euroshieldLedPins[1], level);
}

void Led3Level(uint8_t level){
 // TODO! analogWrite(euroshieldLedPins[2], level);
}

void Led4Level(uint8_t level){
  //Serial.println(String("****** Led4Level level ") + level);
 // TODO!  analogWrite(euroshieldLedPins[3], level);
}

void Led4Digital(bool state){
 // TODO! digitalWrite(euroshieldLedPins[3], state);
}







//// end used and abused code //////////////////////////////////////////////












void ChangeCvWaveformBAmplitude(){
  
  // change by an amount (might go up or down)
  cv_waveform_b_amplitude += cv_waveform_b_amplitude_delta;

  // TODO do something with bela
  // SetWaveformBObjectAmplitude ();
 
}

void SetWaveformBObjectAmplitude (){
  
   // reset if its out of bounds
  if (cv_waveform_b_amplitude < 0 ) {
    cv_waveform_b_amplitude = 0;
    }

  if (cv_waveform_b_amplitude > 1 ) {
    cv_waveform_b_amplitude = 1;
    }


  // set it.
  // TODO set bela oscailator
  //cv_waveform_b_object.amplitude(cv_waveform_b_amplitude, 10); // setting-b-amplitude
  //Serial.println(String("cv_waveform_b_object.amplitude was set to: ") + cv_waveform_b_amplitude);
}






void CvStop(){
  rt_printf("CvStop");
  
  // TODO
  //cv_waveform_b_amplitude = 0;
  //cv_waveform_b_object.amplitude(cv_waveform_b_amplitude, 10);
}



void clockShowHigh(){
  //Serial.println(String("Clock Show HIGH ") );
  //analogWrite(teensy_led_pin, BRIGHT_4);   // set the LED on
  
  // TODO BELA SHOW CLOCK OR SO 
  
  //digitalWrite(teensy_led_pin, HIGH);   // set the LED on
}

void clockShowLow(){
  //Serial.println(String("Clock Show LOW") );
  //analogWrite(teensy_led_pin, BRIGHT_0);
  
   // TODO BELA SHOW CLOCK OR SO 
  //digitalWrite(teensy_led_pin, LOW);   // set the LED off
}



// Each time we start the sequencer we want to start from the same conditions.
void InitSequencer(BelaContext *context){
  GateLow(context);
  CvStop();
  loop_timing.tick_count_since_start = 0;
  ResetSequenceCounters();
}

void StartSequencer(BelaContext *context){
  //rt_printf("Start Sequencer ");
  
  //debugPrint('A', 'B', 'C', 1,2,3);
  
  //char a[200] = "simon \n";
  //pass_string(a);
  
  InitSequencer(context);
  sequence_is_running = HIGH;
}

void StopSequencer(BelaContext *context){
  //rt_printf("Stop Sequencer ");      
  InitSequencer(context);
  sequence_is_running = LOW;        
}

float analogRead(int pin){
	// TODO
	1;
}


// void DebugPrint(char m[]){
//   rt_printf("%s", m);
// } 


int GetValue(int raw, int last, int jitter_reduction){
int value; 

 int diff = abs(raw - last);

  // because the value seems to woble, only take the even value for less jitter.
  if (diff >= jitter_reduction){
    value = raw;
    //Serial.println(String("GetValue says use RAW value because diff is ") + diff );
  } else {
    value = last;
    //Serial.println(String("GetValue says use LAST value because diff is ") + diff );
  }
return value;
}

bool IsCrossing(int value_1, int value_2, int fuzzyness){
  // Return true if the two values are close
  if (abs(value_1 - value_2) <= fuzzyness){
    return true;
  } else {
    return false;
  }
}


void PlayMidi(){
  // rt_printf("midi_note  ") + i + String(" value is ") + channel_a_midi_note_events[step_count][i]  );



  for (uint8_t n = 0; n <= 127; n++) {
    //rt_printf("** OnStep ") + step_count + String(" Note ") + n +  String(" ON value is ") + channel_a_midi_note_events[step_count][n][1]);
    
    // READ MIDI MIDI_DATA
    if (channel_a_midi_note_events[StepCountSanity(step_count)][n][1].is_active == 1) { 
           // The note could be on one of 6 ticks in the sequence
           if (channel_a_midi_note_events[StepCountSanity(step_count)][n][1].tick_count_in_sequence == loop_timing.tick_count_in_sequence){
             // rt_printf("Step:Ticks ") + step_count + String(":") + ticks_after_step + String(" Found and will send Note ON for ") + n );
             // TODO SEND BELA MIDI MIDI.sendNoteOn(n, channel_a_midi_note_events[StepCountSanity(step_count)][n][1].velocity, 1);
           }
    } 

    // READ MIDI MIDI_DATA
    if (channel_a_midi_note_events[StepCountSanity(step_count)][n][0].is_active == 1) {
       if (channel_a_midi_note_events[StepCountSanity(step_count)][n][0].tick_count_in_sequence == loop_timing.tick_count_in_sequence){ 
           // rt_printf("Step:Ticks ") + step_count + String(":") + ticks_after_step +  String(" Found and will send Note OFF for ") + n );
          // TODO SEND BELA MIDI  MIDI.sendNoteOff(n, 0, 1);
       }
    }
} // End midi note loop

}




void AdvanceSequenceChronology(){
  
  // This function advances or resets the sequence powered by the clock.

  // But first check / set the desired sequence length

  //rt_printf("Hello from AdvanceSequenceChronology ");

    //Serial.println(String("sequence_length_in_steps_raw is: ") + sequence_length_in_steps_raw  );
  // Reverse because we want fully clockwise to be short so we get 1's if sequence is 1.
  sequence_length_in_steps = 16 - sequence_length_in_steps_raw;

  //rt_printf("sequence_length_in_steps is: %d ", sequence_length_in_steps  );

  if (sequence_length_in_steps < MIN_SEQUENCE_LENGTH_IN_STEPS){
    rt_printf("**** ERROR with sequence_length_in_steps it WAS: %d but setting it to: %d ", sequence_length_in_steps, MIN_SEQUENCE_LENGTH_IN_STEPS );
    sequence_length_in_steps = MIN_SEQUENCE_LENGTH_IN_STEPS; 
    
  }
  
  if (sequence_length_in_steps > MAX_SEQUENCE_LENGTH_IN_STEPS){
    sequence_length_in_steps = MAX_SEQUENCE_LENGTH_IN_STEPS; 
    rt_printf("**** ERROR with sequence_length_in_steps but it is NOW: %d ", sequence_length_in_steps  );
  }

  new_sequence_length_in_ticks = (sequence_length_in_steps) * 6;
  //Serial.println(String("sequence_length_in_steps is: ") + sequence_length_in_steps  ); 
  //Serial.println(String("new_sequence_length_in_ticks is: ") + new_sequence_length_in_ticks  );  

  // Always advance the ticks SINCE START
  SetTotalTickCount(loop_timing.tick_count_since_start += 1);




  // Midi provides 24 PPQ (pulses per quarter note) (crotchet). 
  // 
  // We want to "advance" our sequencer every (24/2)/2 = 6 pulses aka ticks. (every semi-quaver / "sixteenth" even if we have 8 of them in a sequence)

   // (
   // We have 24 ticks per beat 
   // crotchet * 1 = 24 (4 semiquavers)
   // crotchet * 2 = 48 (8 semiquavers)
   // crotchet * 4 = 96 (16 semiauqvers)
   // )

  // Advance the tick_count as long as we're not at the end of the sequence
  // tick_count_in_sequence is zero indexed
  // new_sequence_length_in_ticks is one indexed
  // 

  // If we're at the end of the sequence
  if (
    (loop_timing.tick_count_in_sequence + 1 == new_sequence_length_in_ticks )

  // or we past the end and we're at new beat  
  ||
  (loop_timing.tick_count_in_sequence + 1  >= new_sequence_length_in_ticks 
      && 
      // loop_timing.tick_count_since_start % new_sequence_length_in_ticks == 0 
      // If somehow we overshot (because pot was being turned whilst sequence running), only 
      loop_timing.tick_count_since_start % 6 == 0 
  )
  // or we're past 16 beats worth of ticks. (this could happen if the sequence length gets changed during run-time)
  || 
  loop_timing.tick_count_in_sequence >= 16 * 6
  ) { // Reset
    ResetSequenceCounters();
  } else {
    SetTickCountInSequence(loop_timing.tick_count_in_sequence += 1); // Else increment.
  }

  // Update Step Count (this could also be a function but probably makes sense to store it)
  // An integer operation - we just want the quotient.
  step_count = loop_timing.tick_count_in_sequence / 6;

  // Just to show the tick progress  
  ticks_after_step = loop_timing.tick_count_in_sequence % 6;

 //Serial.println(String("step_count is ") + step_count  + String(" ticks_after_step is ") + ticks_after_step  ); 

  
}


////


//////// SequenceSettings (Everytime we get a midi clock pulse) ////////////////////////////
// This is called from the main loop() function on every Midi Clock message.
// It contains things that we want to check / happen every tick..
int SequenceSettings(){
  // Note we set tick_count_in_sequence to 0 following stop and start midi messages.
  // The midi clock standard sends 24 ticks per crochet. (quarter note).

 int called_on_step = 0; // not currently used


  ////////////////////////////////////////////////////////////////
  // Read button state
  int button_1_state = 1; // TODO digitalRead(euroshield_button_pin); // Pressed = LOW, Normal = HIGH
  //rt_printf("button_1_state is: ") + button_1_state);


  // state-change-3
  
  int button_1_has_changed = 0; // TODO Button1HasChanged(button_1_state);
  //rt_printf("button_1_has_changed is: ") + button_1_has_changed);




  ////////////////////////////////////////////
  // Get the Pot positions. 
  // We will later assign the values dependant on the push button state
  upper_input_raw = analogRead(upper_pot_pin);
  


  //lower_input_raw = analogRead(lower_pot_pin);
  //rt_printf("*****lower_input_raw *** is: %s ", lower_input_raw  );


//  if ((button_1_state == HIGH) & IsCrossing(binary_sequence_input, upper_input_raw, FUZZINESS_AMOUNT)) {
    binary_sequence_input = binary_sequence_input_raw; // GetValue(binary_sequence_input_raw, binary_sequence_input, jitter_reduction);
    //rt_printf("**** NEW value for binary_sequence_input is: %s ", binary_sequence_input  );
    
  //} else {
  //  rt_printf("NO new value for binary_sequence_input . Sticking at: %s", binary_sequence_input  );
  //}
  
  //if ((button_1_state == LOW) & IsCrossing(sequence_length_input, upper_input_raw, FUZZINESS_AMOUNT)) {   
    sequence_length_input = GetValue(upper_input_raw, sequence_length_input, jitter_reduction);
    //rt_printf("**** NEW value for sequence_length_input is: %s ", sequence_length_input  );
  //} else {
  //  rt_printf("NO new value for sequence_length_input . Sticking at: %s ", sequence_length_input  );
  //}
  
  //if ((button_1_state == HIGH) & IsCrossing(lfo_a_frequency_input, lower_input_raw, FUZZINESS_AMOUNT)) {    
    lfo_a_frequency_input = GetValue(lower_input_raw, lfo_a_frequency_input, jitter_reduction);
    //rt_printf("**** NEW value for lfo_a_frequency_input is: %s ", lfo_a_frequency_input  );  
  //} else {
   // rt_printf("NO new value for lfo_a_frequency_input . Sticking at: %s", lfo_a_frequency_input  );
  //}
  
  
  //if ((button_1_state == LOW) & IsCrossing(lfo_b_frequency_input, lower_input_raw, FUZZINESS_AMOUNT)) {   
    lfo_b_frequency_input = GetValue(lower_input_raw, lfo_b_frequency_input, jitter_reduction);
    //rt_printf("**** NEW value for lfo_b_frequency_input is: %s", lfo_b_frequency_input  );
  //} else {
   // rt_printf("NO new value for lfo_b_frequency_input . Sticking at: %s", lfo_b_frequency_input  );
 // }


//rt_printf("**** binary_sequence_input is now: ") + binary_sequence_input  ); 
//rt_printf("**** sequence_length_input is now: ") + sequence_length_input  ); 
//rt_printf("**** lfo_a_frequency_input is now: ") + lfo_a_frequency_input  );
//rt_printf("**** lfo_b_frequency_input is now: ") + lfo_b_frequency_input  ); 



  

        right_peak_level = 1.0; // TODO peak_R.read() * 1.0;
        //rt_printf("right_peak_level: ") + right_peak_level );  


     external_modulator_object_level = right_peak_level;

// TODO!    external_modulator_object.amplitude(1 - external_modulator_object_level, 10);


//////////////////////////////////////////////

//amp_1_object.gain(1.0);



//////////////////////////////////////////
// Assign values to change the sequencer.
///////////////////////////////////

   // 8 bit sequence - 8 Least Significant Bits
   last_binary_sequence_result = binary_sequence_result;

 //  binary_sequence = (binary_sequence_input & sequence_bits_8_through_1) + 1;

   // If we have 8 bits, use the range up to 255

   
   uint8_t binary_sequence_lower_limit = 1;  // Setting to 1 means we never get 0 i.e. a blank sequence especially when we change seq length
   // TODO Could probably use a smaller type 
   unsigned int binary_sequence_upper_limit; 


//binary_sequence_upper_limit = pow(sequence_length_in_steps, 2);

// REMEMBER, sequence_length_in_steps is ONE indexed (from 1 up to 16) 
// For a 3 step sequence we want to cover all the possibilities of a 3 step sequence which is (2^3) - 1 = 7
// i.e. all bits on of a 3 step sequence is 111 = 7 decimal 
// or (2^sequence_length_in_steps) - 1
binary_sequence_upper_limit = pow(2, sequence_length_in_steps) - 1; 

   //rt_printf("binary_sequence_upper_limit is: ") + binary_sequence_upper_limit  );
    


  // Button is in Normal state (not pressed) (HIGH) (button_1_state == HIGH)
   // ***UPPER Pot HIGH Button*** //////////
  // Generally the lowest value from the pot we get is 2 or 3 
  // setting-1
  binary_sequence_result = fscale( 1, 1023, binary_sequence_lower_limit, binary_sequence_upper_limit, binary_sequence_input, 0);

   


   if (binary_sequence_result != last_binary_sequence_result){
    //rt_printf("binary_sequence has changed **"));
   }


   //rt_printf("binary_sequence_result is: ") + binary_sequence_result  );
   //Serial.print("\t");
   //Serial.print(binary_sequence_result, BIN);
   //Serial.println();

   gray_code_sequence = Binary2Gray(binary_sequence_result);
   //rt_printf("gray_code_sequence is: ") + gray_code_sequence  );
   //Serial.print("\t");
   //Serial.print(gray_code_sequence, BIN);
   //Serial.println();



    the_sequence = gray_code_sequence;

    the_sequence = BitClear(the_sequence, sequence_length_in_steps -1); // sequence_length_in_steps is 1 based index. bitClear is zero based index.

    the_sequence = ~ the_sequence; // Invert

   
    // So pot fully counter clockwise is 1 on the first beat 
    if (binary_sequence_result == 1){
      the_sequence = 1;
    }


    
    

   //rt_printf("the_sequence is: %s ", the_sequence  );
   //Serial.print("\t");
   //Serial.print(the_sequence, BIN);
   //Serial.println();


   
  //rt_printf("right_peak_level is: ") + right_peak_level  );

 

// Sequence length raw
// ***UPPER pot LOW value***
 sequence_length_in_steps_raw = fscale( 15, 1023, 0, 15, sequence_length_input, 0);   ;
 // rt_printf("sequence_length_in_steps is: ") + sequence_length_in_steps  );
   
   //((sequence_length_input & sequence_length_in_steps_bits_8_7_6) >> 5) + 1; // We want a range 1 - 8
   

  // Highlight the first step 
  if (step_count == FIRST_STEP) {

    // If the sequence length is 8 (very predictable), make it shine!
    if (sequence_length_in_steps == 8){
      Led2Level(BRIGHT_5);
      //Led4Digital(true);
    } else {
      Led2Level(BRIGHT_2);
      //Led2Level(fscale( FIRST_STEP, sequence_length_in_steps, 0, BRIGHT_1, sequence_length_in_steps, 0));
      //Led4Digital(false);
    }
  
  } else {
      // Else off.
      Led2Level(BRIGHT_0);

  }

// continuous indication of length
    if (sequence_length_in_steps == 16){
      Led3Level(BRIGHT_2);     
    } else if (sequence_length_in_steps == 8){
      Led3Level(BRIGHT_5);
    } else if (sequence_length_in_steps == 4){
      Led3Level(BRIGHT_4);
    } else {
      Led3Level(BRIGHT_0);
    }


  // Led3Level(fscale( MIN_SEQUENCE_LENGTH_IN_STEPS, MAX_SEQUENCE_LENGTH_IN_STEPS, 0, BRIGHT_5, sequence_length_in_steps, -1.5));   
   
   // UPPER Pot LOW Button (Jitter Reduction AKA Stability)
   //jitter_reduction = (sequence_length_input & jitter_reduction_bits_5_4_3_2_1) >> 0;
   //Led3Level(fscale( 0, 31, 0, BRIGHT_3, jitter_reduction, -1.5));

   
   //jitter_reduction = fscale( 0, 255, 0, 4, left_peak_level, 0);
   //rt_printf("jitter_reduction is: ") + jitter_reduction  );
   //Led3Level(fscale( 0, 255, 0, BRIGHT_3, jitter_reduction, -1.5));


// analog_clock_in_level check TODO is this correct?
 
   float amp_1_gain = fscale( 0, 1, 0, 1, analog_clock_in_level, 0);
   //rt_printf("amp_1_gain is: ") + amp_1_gain  );
// TODO what is this really setting ?
  // amp_1_object.gain(amp_1_gain); // setting-
   //Led3Level(fscale( 0, 1, 0, BRIGHT_3, amp_1_gain, -1.5));


   ////////////////////////////////////// 
   // CV stuff
   // ***LOWER Pot HIGH Button***
   // cv_waveform_a_frequency_raw =  (lfo_a_frequency_input & cv_waveform_a_frequency_raw_bits_8_through_1) >> 0 ; 
   
   cv_waveform_a_frequency_raw =  lfo_a_frequency_input; 
   
   //rt_printf("cv_waveform_a_frequency_raw is: ") + cv_waveform_a_frequency_raw  );
   // LFO up to 20Hz
   cv_waveform_a_frequency = fscale( 3, 1023, 0.001, 10, cv_waveform_a_frequency_raw, -1.5);
   // rt_printf("cv_waveform_a_frequency is: ") + cv_waveform_a_frequency  );
   ////////////////////////

   // ***LOWER Pot LOW Button*** (Multiplex on lfo_b_frequency_input)
   // cv_waveform_b_frequency_raw = ((lfo_b_frequency_input & cv_waveform_b_frequency_bits_4_3_2_1) >> 0);
   cv_waveform_b_frequency_raw = lfo_b_frequency_input;
   //rt_printf("cv_waveform_b_frequency_raw is: ") + cv_waveform_b_frequency_raw  );
   ///////////////////////

  // if the pot is turned clockwise i.e. the CV lasts for a long time, reset it at step 1.
  if (cv_waveform_b_frequency_raw > CV_WAVEFORM_B_FREQUENCY_RAW_MAX_INPUT - 20){
    reset_cv_lfo_at_FIRST_STEP = true;
    //rt_printf("reset_cv_lfo_at_FIRST_STEP is: ") + reset_cv_lfo_at_FIRST_STEP);
  }


   // We want a value that goes from high to low as we turn the pot to the right.
   // So reverse the number range by subtracting from the maximum value.
   //int cv_waveform_b_amplitude_delta_raw = CV_WAVEFORM_B_FREQUENCY_RAW_MAX_INPUT - cv_waveform_b_frequency_raw;
   
  // int cv_waveform_b_amplitude_delta_raw =  cv_waveform_b_frequency_raw;
   
   //rt_printf("cv_waveform_b_amplitude_delta_raw is: ") + cv_waveform_b_amplitude_delta_raw  );
   
   // setting-b-amp-delta
   //cv_waveform_b_amplitude_delta = fscale( 0, CV_WAVEFORM_B_FREQUENCY_RAW_MAX_INPUT, -10, 10, cv_waveform_b_frequency_raw, 1.5) / 100;
   
   cv_waveform_b_amplitude_delta = linearScale( 0, CV_WAVEFORM_B_FREQUENCY_RAW_MAX_INPUT, 0.0, 0.3, cv_waveform_b_frequency_raw);
   

   
   //rt_printf("cv_waveform_b_amplitude_delta is: ") + cv_waveform_b_amplitude_delta  );

   // Lower Pot LOW Button (Multiplex on lfo_b_frequency_input)
   //cv_waveform_a_amplitude_raw = (lfo_b_frequency_input & cv_waveform_a_amplitude_bits_8_7_6_5) >> 4 ; 
   //rt_printf("cv_waveform_a_amplitude_raw is: ") + cv_waveform_a_amplitude_raw  );  
   //cv_waveform_a_amplitude = fscale( 0, 7, 0.1, 0.99, cv_waveform_a_amplitude_raw, -1.5);
   //rt_printf("cv_waveform_a_amplitude is: ") + cv_waveform_a_amplitude  );

   cv_waveform_a_amplitude = 0.99;

   // Put this and above on the inputs.

   // TODO Add offset?
   // Lower Pot LOW Button
   //   cv_offset_raw = (lfo_b_frequency_input & bits_2_1);
   //   rt_printf("cv_offset_raw is: ") + cv_offset_raw  );
   //   cv_offset = fscale( 0, 3, 0, 1, cv_offset_raw, -1.5);
   //   rt_printf("cv_offset is: ") + cv_offset  );

 
    // Used for CV
    // TODO SET SOME BELA OSC HERE
    //cv_waveform_a_object.frequency(cv_waveform_a_frequency); // setting-a-freq
    //cv_waveform_a_object.amplitude(cv_waveform_a_amplitude); // setting-a-amp
    //cv_waveform_a_object.offset(0);




    // MONITOR GATE 
    
    // TODO DRIVE LED OF GATE
    
    // if (gate_monitor.available())
    // {
    //     float gate_peak = gate_monitor.read();
    //     //rt_printf("gate_monitor gate_peak ") + gate_peak  );
    //     Led1Level(fscale( 0.0, 1.0, 0, 255, gate_peak, 0));
    // } else {
    //   //rt_printf("gate_monitor not available ")   );
    // }
    
    
    // TODO DRIVE LED OF CV
    // MONITOR CV
    /// This is connected to cv_waveform and reads the level. We use that to drive the led.
    // if (cv_monitor.available())
    // {
    //     float cv_peak = cv_monitor.read();
    //     //rt_printf("gate_monitor cv_peak ") + cv_peak  );
    //     Led4Level(fscale( 0.0, 1.0, 0, 255, cv_peak, 0));
    // } else {
    //   //rt_printf("gate_monitor not available ")   );
    // }


  return called_on_step;
 
  } // End of SequenceSettings
////////////////////////////////////////////////



////








void InitMidiSequence(){

  rt_printf("InitMidiSequence Start ");

  // Loop through steps
  for (uint8_t sc = FIRST_STEP; sc <= MAX_STEP; sc++) {
    //rt_printf("Step ") + sc );
  
    // Loop through notes
    for (uint8_t n = 0; n <= 127; n++) {
      // Initialise and print Note on (1) and Off (2) contents of the array.
      // WRITE MIDI MIDI_DATA
     channel_a_midi_note_events[sc][n][1].is_active = 0;
     channel_a_midi_note_events[sc][n][0].is_active = 0;

      
      //rt_printf("Init Step ") + sc + String(" Note ") + n +  String(" ON ticks value is ") + channel_a_midi_note_events[sc][n][1].is_active);
      //rt_printf("Init Step ") + sc + String(" Note ") + n +  String(" OFF ticks value is ") + channel_a_midi_note_events[sc][n][0].is_active);
    } 
  }


  for (uint8_t n = 0; n <= 127; n++) {
     channel_a_ghost_events[n].is_active = 0;
     rt_printf("Init Step with ghost Note: %s is_active false", n );
  } 
  

rt_printf("InitMidiSequence Done");
}

















void OnTick(BelaContext *context){
// Called on Every MIDI or Analogue clock pulse
// Drives sequencer settings and activity.

 // rt_printf("Hello from OnTick \n");

  // Read inputs and update settings.  
  //SequenceSettings();



  // Decide if we have a "step"
  if (loop_timing.tick_count_in_sequence % 6 == 0){
    clockShowHigh();
    //rt_printf("loop_timing.tick_count_in_sequence is: ") + loop_timing.tick_count_in_sequence + String(" the first tick of a crotchet or after MIDI Start message") );    
    //////////////////////////////////////////
    OnStep(context);
    /////////////////////////////////////////   
  } else {
    clockShowLow();
    // The other ticks which are not "steps".
    OnNotStep(context);
    //rt_printf("timing.tick_count_in_sequence is: ") + timing.tick_count_in_sequence );
  }

  // Play any suitable midi in the sequence 
  PlayMidi();
   
  // Advance and Reset ticks and steps
  AdvanceSequenceChronology();

}

void MaybeOnTick(BelaContext *context){
  if (do_tick == true){
    do_tick = false;
    OnTick(context);
  }
}


/////////////////////////////////////////////////////////

bool setup(BelaContext *context, void *userData)
{
	
	rt_printf("Hello from Setup: SimonSaysSeeq on Bela :-) \n");
 
 	// If the amout of audio and analog input and output channels is not the same
	// we will use the minimum between input and output
	gAudioChannelNum = std::min(context->audioInChannels, context->audioOutChannels);
	gAnalogChannelNum = std::min(context->analogInChannels, context->analogOutChannels);
 	//gDigitalChannelNum = context->digitalChannels;
 
 
  rt_printf("context->audioSampleRate is: %d \n", context->audioSampleRate);
  rt_printf("context->analogSampleRate is: %d \n", context->analogSampleRate);
  rt_printf("context->audioFrames (per block) is: %d \n", context->audioFrames);

  rt_printf("Using %d In/Out audio channels\n", gAudioChannelNum);
  rt_printf("Using %d In/Out analog channels\n", gAnalogChannelNum);
  //rt_printf("Using %d In/Out digital channels\n", gDigitalChannelNum);



	pinMode(context, 0, 0, INPUT); //set input


	pinMode(context, 0, 0, OUTPUT); // Set gOutputPin as output


	midi.readFrom(gMidiPort0);
	midi.writeTo(gMidiPort0);
	midi.enableParser(false);
	midi.setParserCallback(midiMessageCallback, (void*) gMidiPort0);
	gSamplingPeriod = 1 / context->audioSampleRate;
	
	
	rt_printf("Before checking analog frames.. \n");
	
		// Check if analog channels are enabled
	if(context->analogFrames == 0 || context->analogFrames > context->audioFrames) {
		rt_printf("Error: this example needs analog enabled, with 4 or 8 channels\n");
		return false;
	}

	// Useful calculations
	if(context->analogFrames)
		gAudioFramesPerAnalogFrame = context->audioFrames / context->analogFrames;
	gInverseSampleRate = 1.0 / context->audioSampleRate;
	gPhase = 0.0;

//////

// BELA OSC LGPL


       if(context->audioOutChannels != 2) {
                rt_printf("Error: this example needs stereo audio enabled\n");
                return false;
        }
        srandom(time(NULL));
        
        rt_printf("Creating an oscilator in Setup. \n");
        
        osc.setup(context->audioSampleRate, gWavetableLength, gNumOscillators);
        // Fill in the wavetable with one period of your waveform
        float* wavetable = osc.getWavetable();
        for(int n = 0; n < osc.getWavetableLength() + 1; n++){
                wavetable[n] = sinf(2.0 * M_PI * (float)n / (float)osc.getWavetableLength());
        }
        
        // Initialise frequency and amplitude
        float freq = kMinimumFrequency;
        float increment = (kMaximumFrequency - kMinimumFrequency) / (float)gNumOscillators;
        for(int n = 0; n < gNumOscillators; n++) {
                if(context->analogFrames == 0) {
                        // Random frequencies when used without analogInputs
                        osc.setFrequency(n, kMinimumFrequency + (kMaximumFrequency - kMinimumFrequency) * ((float)random() / (float)RAND_MAX));
                }
                else {
                        // Constant spread of frequencies when used with analogInputs
                        osc.setFrequency(n, freq);
                        freq += increment;
                }
                osc.setAmplitude(n, (float)random() / (float)RAND_MAX / (float)gNumOscillators);
        }
        increment = 0;
        freq = 440.0;
        for(int n = 0; n < gNumOscillators; n++) {
                // Update the frequencies to a regular spread, plus a small amount of randomness
                // to avoid weird phase effects
                float randScale = 0.99 + .02 * (float)random() / (float)RAND_MAX;
                float newFreq = freq * randScale;
                // For efficiency, frequency is expressed in change in wavetable position per sample, not Hz or radians
                osc.setFrequency(n, newFreq);
                freq += increment;
        }
        // Initialise auxiliary tasks
        if((gFrequencyUpdateTask = Bela_createAuxiliaryTask(&recalculate_frequencies, 85, "bela-update-frequencies")) == 0)
                return false;
        gSampleCount = 0;

        rt_printf("Bye from Setup \n");

        return true;
}


enum {kVelocity, kNoteOn, kNoteNumber};
void render(BelaContext *context, void *userData)
{

	// Added by Simon
	bool noteOn;
	int velocity;
	float f0;
	float phaseIncrement;

	

    // Print the global variables periodically for debugging purposes.
    printStatus();
	

	// one way of getting the midi data is to parse them yourself
//	(you should set midi.enableParser(false) above):

	static midi_byte_t noteOnStatus = 0x90; //on channel 1
	static int noteNumber = 0;
	static int waitingFor = kNoteOn;
	static int playingNote = -1;
	int message;
	
	////////////
	
	// BELA OSC LGPL
	// float arr[context->audioFrames];
 //       // Render audio frames
 //       osc.process(context->audioFrames, arr);
 //       for(unsigned int n = 0; n < context->audioFrames; ++n){
 //               audioWrite(context, n, 0, arr[n]);
 //               audioWrite(context, n, 1, arr[n]);
 //       }
 //       if(context->analogFrames != 0 && (gSampleCount += context->audioFrames) >= 128) {
 //               gSampleCount = 0;
 //               gNewMinFrequency = map(context->analogIn[0], 0, 1.0, 1000.0f, 8000.0f);
 //               gNewMaxFrequency = map(context->analogIn[1], 0, 1.0, 1000.0f, 8000.0f);
 //               // Make sure max >= min
 //               if(gNewMaxFrequency < gNewMinFrequency) {
 //                       float temp = gNewMaxFrequency;
 //                       gNewMaxFrequency = gNewMinFrequency;
 //                       gNewMinFrequency = temp;
 //               }
 //               // Request that the lower-priority task run at next opportunity
 //               Bela_scheduleAuxiliaryTask(gFrequencyUpdateTask);
 //       }
	
	
	/////////// END BELA OSC
	
	// Some kind of MIDI detection
	
	while ((message = midi.getInput()) >= 0){
		rt_printf("%d\n", message);
		switch(waitingFor){
		case kNoteOn:
			if(message == noteOnStatus){
				waitingFor = kNoteNumber;
			}
			break;
		case kNoteNumber:
			if((message & (1<<8)) == 0){
				noteNumber = message;
				waitingFor = kVelocity;
			}
			break;
		case kVelocity:
			if((message & (1<<8)) == 0){
				int _velocity = message;
				waitingFor = kNoteOn;
				// "monophonic" behaviour, with priority to the latest note on
				// i.e.: a note off from a previous note does not stop the current note
				// still you might end up having a key down and no note being played if you pressed and released another
				// key in the meantime
				if(_velocity == 0 && noteNumber == playingNote){
					noteOn = false;
					playingNote = -1;
					velocity = _velocity;
				} else if (_velocity > 0) {
					noteOn = true;
					velocity = _velocity;
					playingNote = noteNumber;
					f0 = powf(2, (playingNote-69)/12.0f) * 440;
					phaseIncrement = 2 * M_PI * f0 / context->audioSampleRate;
				}
				rt_printf("NoteOn: %d, NoteNumber: %d, velocity: %d\n", noteOn, noteNumber, velocity);
			}
			break;
		}
	}
	
	    ////////////////
	
	
	  /// SimonSaysSeeq first bits_2_1
	
	
    // Analog Clock (and left input checking) //////


    ///////////////////////////////////////////
   // Look for Analogue Clock (24 PPQ)
   // Note: We use this input for other things too.
   

	// Simplest possible case: pass inputs through to outputs
  // AUDIO LOOP
	for(unsigned int n = 0; n < context->audioFrames; n++) {
		for(unsigned int ch = 0; ch < gAudioChannelNum; ch++){
			audioWrite(context, n, ch, audioRead(context, n, ch));
		}
	}

// ANALOG LOOP
	for(unsigned int n = 0; n < context->analogFrames; n++) {
		for(unsigned int ch = 0; ch < gAnalogChannelNum; ch++){
			analogWrite(context, n, ch, analogRead(context, n, ch));
		}
	}


// DIGITAL LOOP need to handle this differently because of pin in / out
 // for(unsigned int n = 0; n < context->digitalFrames; n++) {
	// 	for(unsigned int ch = 0; ch < gDigitalChannelNum; ch++){
	// 		digitalWrite(context, n, ch, digitalRead(context, n, ch));
	// 	}
	// }


  /*

  // ANALOG LOOP
  for(unsigned int p = 0; p < context->analogFrames; p++) {
        	// This will grab the last value from the last frame 
        //	binary_sequence_input_raw = analogRead(context, p, SEQUENCE_CV_IN_PIN);

      if(gAudioFramesPerAnalogFrame && !(p % gAudioFramesPerAnalogFrame)) {
	          binary_sequence_input_raw = analogRead(context, p/gAudioFramesPerAnalogFrame, 0);
        }
	}
	
	*/
	
	// DIGITAL LOOP 
	    for(unsigned int m = 0; m < context->digitalFrames; m++) {

	  // This function returns the value of an analog input, at the time indicated by frame. 
			  // The returned value ranges from 0 to 1, corresponding to a voltage range of 0 to 4.096V.
			  
        // Next state
        new_digital_clock_in_state = digitalRead(context, m, CLOCK_INPUT_DIGITAL_PIN);


            // Rising clock edge? // state-change-1
            if ((new_digital_clock_in_state == HIGH) && (current_digital_clock_in_state == LOW)){
    
              if (sequence_is_running == LOW){
                StartSequencer(context);
              }
              
              current_digital_clock_in_state = HIGH;
              //Serial.println(String("Went HIGH "));
               
              OnTick(context);
              last_clock_pulse = milliseconds();
              
            } 
    
            // Falling clock edge?
            if ((new_digital_clock_in_state == LOW) && (current_digital_clock_in_state == HIGH)){
              current_digital_clock_in_state = LOW;
              //Serial.println(String("Went LOW "));
            } 
			 
      }
	

        // Only look for this clock if we don't have midi.
        if (midi_clock_detected == LOW) {

          //Serial.println(String(">>>>>NO<<<<<<< Midi Clock Detected midi_clock_detected is: ") + midi_clock_detected) ;
          
          

/*

WORKS
      // We have something like 16 audio frames (block size) each time render runs.
      // Loop Through them and see if we have a high level this time.
	    for(unsigned int n = 0; n < context->analogFrames; n++) {

	  // This function returns the value of an analog input, at the time indicated by frame. 
			  // The returned value ranges from 0 to 1, corresponding to a voltage range of 0 to 4.096V.
			  analog_clock_in_level = analogRead(context, n, CLOCK_INPUT_ANALOG_IN_PIN);


            // Rising clock edge? // state-change-1
            if ((analog_clock_in_level > 0.5) && (analog_clock_in_state == LOW)){
    
              if (sequence_is_running == LOW){
                StartSequencer();
              }
              
              analog_clock_in_state = HIGH;
              //Serial.println(String("Went HIGH "));
               
              OnTick(context);
              last_clock_pulse = milliseconds();
              
            } 
    
            // Falling clock edge?
            if ((analog_clock_in_level < 0.5) && (analog_clock_in_state == HIGH)){
              analog_clock_in_state = LOW;
              //Serial.println(String("Went LOW "));
            } 
			 
      }

*/


      
      





      

   
      
      

        } 
	} // End of render
   
   




   
   
   //if (peak_L.available()){
   
  /* 
   	for(unsigned int n = 0; n < context->audioFrames; n++) {
   		
   		// Increment a counter on every frame
        gCount++;
        //MaybeOnTick();
   		
		  if(gAudioFramesPerAnalogFrame && !(n % gAudioFramesPerAnalogFrame)) {


        MaybeOnTick();

			  // read analog inputs and update frequency and amplitude
			  // Depending on the sampling rate of the analog inputs, this will
			  // happen every audio frame (if it is 44100)
			  // or every two audio frames (if it is 22050)
			  //frequency = map(analogRead(context, n/gAudioFramesPerAnalogFrame, gSensorInputFrequency), 0, 1, 100, 1000);
			  //amplitude = analogRead(context, n/gAudioFramesPerAnalogFrame, CLOCK_INPUT_ANALOG_IN_PIN);
			

			
			
			  // This function returns the value of an analog input, at the time indicated by frame. 
			  // The returned value ranges from 0 to 1, corresponding to a voltage range of 0 to 4.096V.
			  analog_clock_in_level = 0; // causes bug - analogRead(context, n/gAudioFramesPerAnalogFrame, CLOCK_INPUT_ANALOG_IN_PIN);
			 
			 
			 //rt_printf("analog_clock_in_level is: %f \n", analog_clock_in_level);
			
		  }

		// float out = amplitude * sinf(gPhase);

		// for(unsigned int channel = 0; channel < context->audioOutChannels; channel++) {
		// 	audioWrite(context, n, channel, out);
		// }


			
			
			
			
		        // 1.0; // peak_L.read() * 1.0; // minimum seems to be 0.1 from intelij attenuator
        // rt_printf("**** analog_clock_in_level: ") + analog_clock_in_level) ;

        //rt_printf("analog_clock_in_state: ") + analog_clock_in_state) ;

         // ONLY if no MIDI clock, run the sequencer from the Analogue clock.
        if (midi_clock_detected == LOW) {

          //rt_printf(">>>>>NO<<<<<<< Midi Clock Detected midi_clock_detected is: ") + midi_clock_detected) ;
          
          // Only look for this clock if we don't have midi.

            // Rising clock edge? // state-change-1
            if ((analog_clock_in_level > 0.5) && (analog_clock_in_state == LOW)){
    
              if (sequence_is_running == LOW){
              	// TODO
              	 //rt_printf("would StartSequencer because: %f \n", analog_clock_in_level);
              	
                StartSequencer();
              }
              
              analog_clock_in_state = HIGH;
              
              //rt_printf("Set analog_clock_in_state HIGH because: %f \n", analog_clock_in_level);
              
              //rt_printf("Went HIGH "));
              
              
              
              do_tick = true;
              last_clock_pulse = milliseconds();
              
            } 
    
            // Falling clock edge?
            if ((analog_clock_in_level < 0.5) && (analog_clock_in_state == HIGH)){
              analog_clock_in_state = LOW;
              
              //rt_printf("I set analog_clock_in_state LOW because: %f \n", analog_clock_in_level);
              
              
              //rt_printf("Went LOW "));
            } 

        } // 	
			
			
	}

}

*/



   


void cleanup(BelaContext *context, void *userData)
{

}


// BELA LGPL
// This is a lower-priority call to update the frequencies which will happen
// periodically when the analog inputs are enabled. By placing it at a lower priority,
// it has minimal effect on the audio performance but it will take longer to
// complete if the system is under heavy audio load.
void recalculate_frequencies(void*)
{
        float freq = gNewMinFrequency;
        float increment = (gNewMaxFrequency - gNewMinFrequency) / (float)gNumOscillators;
        for(int n = 0; n < gNumOscillators; n++) {
                // Update the frequencies to a regular spread, plus a small amount of randomness
                // to avoid weird phase effects
                float randScale = 0.99 + .02 * (float)random() / (float)RAND_MAX;
                float newFreq = freq * randScale;
                osc.setFrequency(n, newFreq);
                freq += increment;
        }
}





// Here follows some used and abused code:

