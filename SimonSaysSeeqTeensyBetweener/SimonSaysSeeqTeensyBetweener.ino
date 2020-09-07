

//****NOTE: Because the Betweener library uses USB midi, you must set up the Arduino IDE to expect USB MIDI even if you don’t use it in your sketch. 
// To do this, you simply go to the main menu bar in Arduino and select Tools->USB Type->[any that includes MIDI]. I usually select “Serial + MIDI”. If you do not see this option on the menu, make sure that you have selected “Teensy 3.1/3.2 from the Tools->Board menu.
// Also Turn off DODINMIDI in Betweener.h (see below)



// We use : https://github.com/PaulStoffregen/Audio
// See https://www.pjrc.com/teensy/gui/index.html

// Betweener library https://github.com/jkrame1/Betweener/wiki/03_Library-Reference

// Libraries seem to be in /Users/simonredfern/Documents/Arduino/libraries/ 


// Sketchbook location is: /Users/simonredfern/Documents/Arduino

const float simon_says_seq_version = 0.23; 


#include <Audio.h>
#include <MIDI.h> // Note use of Serial1 / Serial2 below. Serial1 for Euroshield it seems but can't use that for Betweener


// Note: We want to use https://github.com/FortySevenEffects/arduino_midi_library
// However even with (a recent verion of) this library we get compile errors, so we comment out the offending MIDI over DIN in Betweener.cpp
// NOTE In case of compile errors with Betweener and the MIDI library used, 


// Turn off DODINMIDI in Betweener.h thus:
// # DODINMIDI


// or modify Betweener.cpp thus:

//#ifdef DODINMIDI
//    Serial2.setRX(DINMIDIIN);
//    Serial2.setTX(DINMIDIOUT);
//    // Compile Error DINMIDI.begin(MIDI_CHANNEL_OMNI);  //monitor all input channels
//#endif


//#ifdef DODINMIDI
//void Betweener::readDINMIDI(void){
//        // Compile bug DINMIDI.read();
//}
//#endif

/////////////////////////////////////////////////////////////////////////////////////////

#include <Betweener.h>

Betweener b; // Must begin it below!

AudioSynthWaveform       test_waveform_object;
AudioAnalyzeRMS          test_rms_object;
AudioConnection          patchCordTest1(test_waveform_object, test_rms_object);  

AudioSynthWaveformDc     gate_dc_waveform;

/////////////////////////////////////
// How we generate CV
//  (offset -1 to + 1)
//  waveform a ---  (no params)                 (gain = 0 to 32767.0)
//                |---Multiply---|---Amp(0-1?)---             
//  waveform b ---                               |---Summing Mixer---|---RMS Monitor(0.0-1.0)---|(Scale in code)|---(0-4095)WriteCV---|Output 
//                                    dc-offset---       
//
//

AudioSynthWaveform       cv_waveform_a_object;      
AudioSynthWaveformDc     cv_waveform_b_object; 
AudioEffectMultiply      multiply1;      
AudioAmplifier           amp_1_object;
AudioSynthWaveformDc     cv_dc_offset_object;
AudioMixer4              mixer_1_object;
AudioAnalyzeRMS          cv_monitor_rms; 


//////////////
// Modulate CV
// Waveform a and b are multiplied together
AudioConnection          patchCord6(cv_waveform_a_object, 0, multiply1, 1);
AudioConnection          patchCord7(cv_waveform_b_object, 0, multiply1, 0);

// Amplify the result
AudioConnection          patchCord13(multiply1, 0, amp_1_object, 0);

// Add a DC offset.
AudioConnection          patchCord11(amp_1_object, 0, mixer_1_object, 0); 
AudioConnection          patchCord12(cv_dc_offset_object, 0, mixer_1_object, 1); 

// CV Output is via cv_monitor_rms which we read in code.
AudioConnection          patchCord2(mixer_1_object, cv_monitor_rms); // CV -> monitor (for LED)



////////////////////////////////////////////

AudioInputI2S        audioInput;         // audio shield: mic or line-in
AudioOutputI2S       audioOutput;        // audio shield: headphones & line-out         
          
AudioAnalyzeRMS          gate_monitor_rms; 

//////////////////////////
// GATE Output and Monitor
// Euroshield AudioConnection          patchCord3(gate_dc_waveform, 0, audioOutput, 0); // GATE -> Upper Audio Out
AudioConnection          patchCord9(gate_dc_waveform, gate_monitor_rms); // GATE -> montior (for LED)



AudioAnalyzePeak     peak_L;
AudioAnalyzePeak     peak_R;

// Euroshield AudioConnection c1(audioInput, 0, peak_L, 0);
// Euroshield AudioConnection c2(audioInput, 1, peak_R, 0);

AudioControlSGTL5000     audioShield; 

//Euroshield AudioConnection          patchCord10(amp_1_object, 0, audioOutput, 1); // CV -> Lower Audio Out
//Euroshield AudioConnection          patchCord11(multiply1, 0, amp_1_object, 0); // CV -> Lower Audio Out

/////////////
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

const uint8_t betweener_led_pin = 8;

const uint8_t device = 1; // 0 Euroshield, 1 Betweener



///////////////



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

const uint8_t CV_WAVEFORM_B_FREQUENCY_RAW_MAX_INPUT = 15;


const uint8_t MIDI_NOTE_ON = 1;
const uint8_t MIDI_NOTE_OFF = 0;




unsigned int min_pot_value;
unsigned int max_pot_value;


////////////////////////////////////////////////////
// Actual pot values
unsigned int upper_input_raw; // TODO Make t type.
unsigned int lower_input_raw;


// Create 4 virtual pots out of two pots and a button.
// To handle the case when 1) Pot is fully counter clockwise 2) We press the button 3) Move the pot fully clockwise 4) Release the button.
// We introduce the concept that a virtual pot can be "engaged" or not so we can catchup with its stored value only when the pot gets back to that position.
bool upper_pot_high_engaged = true;
unsigned int input_1_value_raw;
unsigned int input_1_value;
unsigned int input_1_value_last;
unsigned int input_1_value_at_button_change;

bool lower_pot_high_engaged = true;
unsigned int input_2_value_raw;
unsigned int input_2_value;
unsigned int input_2_value_last;
unsigned int input_2_value_at_button_change;

bool upper_pot_low_engaged = true;
unsigned int input_3_value_raw;
unsigned int input_3_value;
unsigned int input_3_value_last;
unsigned int input_3_value_at_button_change;

bool lower_pot_low_engaged = true;
unsigned int input_4_value_raw;
unsigned int input_4_value;
unsigned int input_4_value_last;
unsigned int input_4_value_at_button_change;



float input_5_value;
float input_6_value;

float input_7_value;
float input_8_value;

////////////////////////////////////////////////////


////////////////////////////////////////////////////
// Musical parameters that the user can tweak.

uint8_t sequence_length_in_steps_raw;


// The Primary GATE sequence pattern // Needs to be upto 16 bits. Maybe more later.
unsigned int binary_sequence_1;
unsigned int grey_sequence_1;
unsigned int hybrid_sequence_1;
unsigned int last_binary_sequence_1; // So we can detect changes


unsigned int hard_coded_seqs[] = {
B00000000,
B00000001, // same as 11111111 if we loop
B00010001,
B00001001,
B00100001,
B01000001,
B01010001,
B01010111,
B11010101,
B10110101,
B10010101,
B11011101,
B10011101,
B00000101,
B00000111,
B10000111,
B11000111,
B11010111,
B11010101,
B11010110,
B10010110, // Used up to about here.
B11010011,
B01010101,
B01100101,
B01001101,
B01011101,
B01011110,
B11011110,
B11111110,
B11111111,
B00001001,
B00100001,
B01000001,
B01010001,
B01010111,
B11010101,
B10110101,
B10010101,
B11011101,
B10011101,
B00000101,
B00000111,
B10000111,
B11000111,
B11010111,
B11010101,
B11010110,
B10010110,
B11010011,
B01010101,
B01100101,
B01001101,
B01011101,
B01011110,
B11011110,
B11111110,
B11111111,
B00001001,
B00100001,
B01000001,
B01010001,
B01010111,
B11010101,
B10110101,
B10010101,
B11011101,
B10011101,
B00000101,
B00000111,
B10000111,
B11000111,
B11010111,
B11010101,
B11010110,
B10010110,
B11010011,
B01010101,
B01100101,
B01001101,
B01011101,
B01011110,
B11011110,
B11111110,
B00001001,
B00100001,
B01000001,
B01010001,
B01010111,
B11010101,
B10110101,
B10010101,
B11011101,
B10011101,
B00000101,
B00000111,
B10000111,
B11000111,
B11010111,
B11010101,
B11010110,
B10010110,
B11010011,
B01010101,
B01100101,
B01001101,
B01011101,
B01011110,
B11011110,
B11111110,
B11111111,
B11111111,
B00010001,
B00001001,
B00100001,
B01000001,
B01010001,
B01010111,
B11010101,
B10110101,
B10010101,
B11011101,
B10011101,
B00000101,
B00000111,
B10000111,
B11000111,
B11010111,
B11010101,
B11010110,
B10010110,
B11010011,
B01010101,
B01100101,
B01001101,
B01011101,
B01011110,
B11011110,
B11111110,
B11111111,
B00001001,
B00100001,
B01000001,
B01010001,
B01010111,
B11010101,
B10110101,
B10010101,
B11011101,
B10011101,
B00000101,
B00000111,
B10000111,
B11000111,
B11010111,
B11010101,
B11010110,
B10010110,
B11010011,
B01010101,
B01100101,
B01001101,
B01011101,
B01011110,
B11011110,
B11111110,
B11111111,
B00001001,
B00100001,
B01000001,
B01010001,
B01010111,
B11010101,
B10110101,
B10010101,
B11011101,
B10011101,
B00000101,
B00000111,
B10000111,
B11000111,
B11010111,
B11010101,
B11010110,
B10010110,
B11010011,
B01010101,
B01100101,
B01001101,
B01011101,
B01011110,
B11011110,
B11111110,
B00001001,
B00100001,
B01000001,
B01010001,
B01010111,
B11010101,
B10110101,
B10010101,
B11011101,
B10011101,
B00000101,
B00000111,
B10000111,
B11000111,
B11010111,
B11010101,
B11010110,
B10010110,
B11010011,
B01010101,
B01100101,
B01001101,
B01011101,
B01011110,
B11011110,
B11111110,
B11110111,
B01111111,
B11011111,
B11111111,
B11111110,
B11111111,
B11111011,
B10111111,
B11111111,
B11111101,
B11010011,
B01010101,
B01100101,
B01001101,
B01011101,
B01011110,
B11011110,
B11110110,
B11011111,
B01001101,
B01011101,
B01011110,
B11011110,
B11011010,
B00001001,
B00100001,
B01000001,
B01010001,
B01010111,
B11010101,
B10110101,
B10010101,
B11011101,
B10011101,
B10101101,
B10000111,
B10111111,
B11111111,
B11111111,
};






// Sequence Length
uint8_t sequence_length_in_steps = 8; 

// Used to control when/how we change sequence length 
//uint8_t previous_sequence_length_in_ticks;
uint8_t new_sequence_length_in_ticks; 

// Just counts 0 to 5 within each step
uint8_t ticks_after_step;

// Jitter Reduction: Used to flatten out glitches from the analog pots 
uint8_t jitter_reduction = 0; // was 20

// LFO
unsigned int cv_waveform_a_frequency_raw;
float cv_waveform_a_frequency;

boolean reset_cv_lfo_at_FIRST_STEP = false;

// Amplitude of the LFO
unsigned int cv_waveform_a_amplitude_raw;
float cv_waveform_a_amplitude;


float cv_offset_raw;
float cv_offset;


//////////////////////
// Midi clock and start / stop related
// We use the following library  https://github.com/FortySevenEffects/arduino_midi_library/wiki/Using-custom-Settings

//MIDI_CREATE_INSTANCE(HardwareSerial, Serial1, MIDI); // This was Euroshield

//MIDI_CREATE_INSTANCE(HardwareSerial, Serial2, MIDI); // Check this

//MIDI_CREATE_CUSTOM_INSTANCE(HardwareSerial, Serial8, MIDI);


// https://github.com/jkrame1/Betweener/blob/master/examples/Hardware%20Tests/A_Menu_Driven_Hardware_Test/A_Menu_Driven_Hardware_Test.ino



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


////////////////////////////
// LED Display 
// unsigned int led_1_level = 0; 


////////////////////////////////////////
// Bit Constants for bit wise operations 


 
uint8_t sequence_bits_8_through_1 = 128 + 64 + 32 + 16 + 8 + 4 + 2 + 1;
  
uint8_t jitter_reduction_bits_5_4_3_2_1 = 16 + 8 + 4 + 2 + 1; 


uint8_t sequence_length_in_steps_bits_8_7_6 = 128 + 64 + 32;  

uint8_t cv_waveform_a_frequency_raw_bits_8_through_1 = 128 + 64 + 32 + 16 + 8 + 4 + 2 + 1; // CV frequency


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
// TODO Changes to Sequence Length should be done here / or when done this function should be called immediately.
  
  //Serial.println(String("ResetSequenceCounters Done. sequence_length_in_steps is ") + sequence_length_in_steps + String(" step_count is now: ") + step_count);
}



uint8_t IncrementStepCount(){
  step_count = step_count_sanity(step_count + 1);

  //Serial.println(String("IncrementStepCount. sequence_length_in_steps is ") + sequence_length_in_steps + String(" step_count is now: ") + step_count);
  return step_count_sanity(step_count);
}



boolean midi_clock_detected = LOW;

uint8_t Device(){
  // Return 1 for Betweener, 0 for Euroshield.
  return 1; 
}

boolean IsBetweener(){
  if (Device()==1){
    return true;
  } else {
    return false;
  }
}

boolean IsEuroshield(){
  if (Device()==0){
    return true;
  } else {
    return false;
  }
}



void setup() {

  // Debugging hello
  //Serial.begin(57600);
  Serial.begin(115200);
  //Serial.println(String("Hello from Simon-Says-Grey-Code-Seeq"));


  int resolution = 10;



      //Serial.println(String("I'm expecting to be plugged into a Betweener. Starting b...")) ;

      //////////
      b.begin();
      /////////






        resolution = 10; // Need to use 10 for Betweener





  analogReadResolution(resolution);

  min_pot_value = 0;
  max_pot_value = pow(2, resolution) - 1;

  //Serial.println(String("resolution is : ") + resolution + String(" bits. The range is ") + min_pot_value + " to " + max_pot_value ) ;

  //Serial.println(String("audioShield.inputSelect on: ") + AUDIO_INPUT_LINEIN ) ;

  // Audio connections require memory to work.  For more
  // detailed information, see the MemoryAndCpuUsage example
  AudioMemory(15);


  

  ///////////////////////////////////////////
  // Pin configuration
  // initialize the digital pin as an output.
  // pinMode(teensy_led_pin, OUTPUT);

  uint8_t i;

    pinMode(betweener_led_pin, OUTPUT);

  ///////////////////////////////////////


  // Begin Midi
//MIDI.begin(MIDI_CHANNEL_OMNI);



//MIDI.turnThruOn(midi::Thru::Full); //  Off, Full, SameChannel, DifferentChannel
 


  //cv_waveform_a_object.begin(WAVEFORM_SAWTOOTH);

  cv_waveform_a_object.begin(WAVEFORM_SINE);
  
  ////////////////////////////
  // TEST OBJECT /////////////
  test_waveform_object.begin(WAVEFORM_SAWTOOTH);
  test_waveform_object.frequency(1); // setting-a-freq
  test_waveform_object.amplitude(1); // TEMP cv_waveform_a_amplitude); // setting-a-amp
  test_waveform_object.offset(0);
  ///////////////////////////////



  mixer_1_object.gain(1,1);

   
  // Enable the audio shield, select input, and enable output
  // This is to read peak.

if (IsEuroshield()){
  
//  audioShield.enable();
//  audioShield.inputSelect(AUDIO_INPUT_LINEIN);
//  audioShield.volume(0.82);
//  audioShield.adcHighPassFilterDisable();
//  audioShield.lineInLevel(5,5);
}

  // Initialise timing
  // Use zero based indexing for step_count so that modular operations (counting multiples of 4, 16 etc are easier)
  step_count = FIRST_STEP;
  loop_timing.tick_count_in_sequence = 0;
  //

  // Show contents / initialise the midi sequence 
 InitMidiSequence();


   /////////////////////////////////////////////////////////
   // Say hello, show we are ready to sequence. 
  uint8_t my_delay_time = 50;
  uint8_t my_no_of_times = 10;
  
  // Say Hello to the Teensy LED 
  Flash(my_delay_time, my_no_of_times, teensy_led_pin);

 if (IsEuroshield()) {
    // Say hello to the Euroshield LEDs 
    for (i = 0; i < euroshield_led_pin_count; i++) { 
       Flash(my_delay_time, my_no_of_times, euroshieldLedPins[i]);
    }
 }



      // Say hello to Betweener. There is only one LED on the Betweener, so make it more obvious
      Flash(my_delay_time * 2, my_no_of_times * 2, betweener_led_pin);


// https://en.cppreference.com/w/cpp/types/integer
  //Serial.println(String("Max value in INT8_MAX (int8_t): ") + INT8_MAX ) ; // int8_t max value is 127 
   //Serial.println(String("Max value in UINT8_MAX (uint8_t): ") + UINT8_MAX ) ; // uint8_t max value is 255


}



/////////////// LOOP ////////////////////////////
// the loop() method runs over and over again, as long as the board has power


unsigned long last_clock_pulse=0;

boolean analogue_gate_state = LOW;

boolean sequence_is_running = LOW;




void loop() {

// The way we "connect" the Audio Objects to the output of the Betweener is to read an rms object and use that value
// to writeCVOut.

//if (test_rms_object.available()){
//        float test_rms = test_rms_object.read();
//
//        // Also use this to drive the betweener ouput
//        if (IsBetweener()){
//          int test_rms_betweener_scaled = fscale( 0.0, 1.0, 0, 4095, test_rms, 0); 
//          Serial.println(String("test_rms_betweener_scaled is: ") + test_rms_betweener_scaled  );
//          b.writeCVOut(2, test_rms_betweener_scaled);
//          b.writeCVOut(3, test_rms_betweener_scaled);
//          b.writeCVOut(4, test_rms_betweener_scaled);
//        }
//
//} else {
//  //Serial.println(String("test_rms_object is not available!!"));
//}
  


    //b.readTriggers();
    b.readAllInputs();

    if (b.trig1.risingEdge()) {
          if (sequence_is_running == LOW){
                StartSequencer();
          }
          OnTick();
          last_clock_pulse = millis();
    }
    


///////////////////

// Analog Clock (and left input checking) //////





 
/////////////////////////////////////////////////////////////////////////////////
// When relying on the analogue clock, we don't have a stop as such, so if we don't detect a clock for a while, then assume its stopped.
// Note that the Beat Step Pro takes a while to kill its clock out after pressing the Stop button.
if (midi_clock_detected == LOW){
  if ((millis() - last_clock_pulse > 500) && (sequence_is_running == HIGH)) {
    Serial.println("No analogue clock detected for 500 ms. Stopping sequencer.");

    // state-change-2
    
    StopSequencer();
  }
}


} //////////////////////////////////////
///// END of LOOP //////////////////////





// Each time we start the sequencer we want to start from the same conditions.
void InitSequencer(){
  GateLow();
  CvStop();
  loop_timing.tick_count_since_start = 0;
  ResetSequenceCounters();
}

void StartSequencer(){
  Serial.println(String("Starting Sequencer... "));
  InitSequencer();
  sequence_is_running = HIGH;
}

void StopSequencer(){
  Serial.println(String("Stopping Sequencer... Hint: I'm now waiting for a midi / cv clock to do something.... "));      
  InitSequencer();
  sequence_is_running = LOW;        
}



void OnTick(){
// Called on Every MIDI or Analogue clock pulse
// Drives sequencer settings and activity.

   // Serial.println(String("loop_timing.tick_count_in_sequence is: ") + loop_timing.tick_count_in_sequence);    
  

  // Read inputs and update settings.  
  ReadInputsAndUpdateSettings();

  // Decide if we have a "step"
  if (loop_timing.tick_count_in_sequence % 6 == 0){
    clockShowHigh();
    //Serial.println(String("loop_timing.tick_count_in_sequence is: ") + loop_timing.tick_count_in_sequence + String(" the first tick of a crotchet or after MIDI Start message") );    
    //////////////////////////////////////////
    OnStep();
    /////////////////////////////////////////   
  } else {
    clockShowLow();
    // The other ticks which are not "steps".
    OnNotStep();
    //Serial.println(String("timing.tick_count_in_sequence is: ") + timing.tick_count_in_sequence );
  }



  SetSequenceLength();

  SetSequencePattern();
   
  // Advance and Reset ticks and steps
  AdvanceSequenceChronology();
}





// This is called from the main loop() function on every Midi / Ananlogue Clock message.
// Things that we want to happen every tick..
void ReadInputsAndUpdateSettings(){
  // Note we set tick_count_in_sequence to 0 following at the stop and start midi messages.
  // The midi clock standard sends 24 ticks per crochet. (quarter note).



// physical -> logical -> actual 
// pot -> value -> result

    //b.readTriggers();
    b.readAllInputs();
  
  // Pot1
  input_1_value = b.readKnob(1); // Sequence  
  //Serial.println(String("input_1_value is: ") + input_1_value  );
  
  // Pot2
  input_2_value = b.readKnob(2); // Length
  //Serial.println(String("input_2_value is: ") + input_2_value  );
  
  // Pot3
  input_3_value = b.readKnob(3); // this is smoothed
  //Serial.println(String("input_3_value is: ") + input_3_value  );
   
  // Pot4
  input_4_value = b.readKnob(4);
  //Serial.println(String("input_4_value is: ") + input_4_value  );

  // Cv1  
  input_5_value = b.readCV(1);
  //Serial.println(String("input_5_value is: ") + input_5_value  );
  
  // Cv2  
  input_6_value = b.readCV(2);
  //Serial.println(String("input_6_value is: ") + input_6_value  );
  
  // Cv3  b.readCV(3);
  // Cv4  b.readCV(4); 








//////////////////////////////////////////
// Assign values to change the sequencer.
///////////////////////////////////




   


  // Highlight the first step 
  if (step_count == FIRST_STEP) {
    Led2Level(BRIGHT_5);
  } else {
    // and even sequences (but less bright)
    if (sequence_length_in_steps % 2 == 0){
      Led2Level(BRIGHT_2);
    } else {
      // Else off.
      Led2Level(BRIGHT_0);
      //Led2Level(fscale( FIRST_STEP, sequence_length_in_steps, 0, BRIGHT_1, sequence_length_in_steps, 0));
    }
  }
   
   
   // UPPER Pot LOW Button (Jitter Reduction AKA Stability)
   //jitter_reduction = (input_3_value & jitter_reduction_bits_5_4_3_2_1) >> 0;
   //Led3Level(fscale( 0, 31, 0, BRIGHT_3, jitter_reduction, -1.5));

   
   //jitter_reduction = fscale( 0, 255, 0, 4, input_5_value, 0);
   //Serial.println(String("jitter_reduction is: ") + jitter_reduction  );
   //Led3Level(fscale( 0, 255, 0, BRIGHT_3, jitter_reduction, -1.5));

 
   float amp_1_gain = fscale( min_pot_value, max_pot_value, 0.0, 1.0, input_5_value, 0);
   // Serial.println(String("amp_1_gain is: ") + amp_1_gain  );

   amp_1_object.gain(amp_1_gain); // setting-
   Led3Level(fscale( 0, 1, 0, BRIGHT_3, amp_1_gain, -1.5));


   ////////////////////////////////////// 
   // CV stuff
   // LOWER Pot HIGH Button
   // ****** "Freq" ******
   //cv_waveform_a_frequency_raw =  (input_3_value & cv_waveform_a_frequency_raw_bits_8_through_1) >> 0 ; 
   //Serial.println(String("cv_waveform_a_frequency_raw is: ") + cv_waveform_a_frequency_raw  );
   // LFO up to 20Hz
   
   
   
   //Eurorack cv_waveform_a_frequency = fscale( 0, 255, 0.01, 20, cv_waveform_a_frequency_raw, -1.5);
   
   //Serial.println(String("input_3_value is: ") + input_3_value  );
   
   cv_waveform_a_frequency = fscale( min_pot_value, max_pot_value, 0.01, 40, input_3_value, -1.5);
   
   
   //Serial.println(String("cv_waveform_a_frequency is: ") + cv_waveform_a_frequency  );

   // LOWER Pot LOW Button (Multiplex on input_4_value)
   // ****** "Shape" ********
   cv_waveform_b_frequency_raw = ((input_4_value & cv_waveform_b_frequency_bits_4_3_2_1) >> 0);
   //Serial.println(String("cv_waveform_b_frequency_raw is: ") + cv_waveform_b_frequency_raw  );


  // if the pot is turned clockwise i.e. the CV lasts for a long time, reset it at step 1.
  if (cv_waveform_b_frequency_raw == CV_WAVEFORM_B_FREQUENCY_RAW_MAX_INPUT){
    reset_cv_lfo_at_FIRST_STEP = true;
    Serial.println(String("reset_cv_lfo_at_FIRST_STEP is: ") + reset_cv_lfo_at_FIRST_STEP);
  }


   // We want a value that goes from high to low as we turn the pot to the right.
   // So reverse the number range by subtracting from the maximum value.
   int cv_waveform_b_amplitude_delta_raw = CV_WAVEFORM_B_FREQUENCY_RAW_MAX_INPUT - cv_waveform_b_frequency_raw;
   //Serial.println(String("cv_waveform_b_amplitude_delta_raw is: ") + cv_waveform_b_amplitude_delta_raw  );
   
   // setting-b-amp-delta
   cv_waveform_b_amplitude_delta = fscale( 0, CV_WAVEFORM_B_FREQUENCY_RAW_MAX_INPUT, 0.01, 0.4, cv_waveform_b_amplitude_delta_raw, -1.5);
   //Serial.println(String("cv_waveform_b_amplitude_delta is: ") + cv_waveform_b_amplitude_delta  );

   // Lower Pot LOW Button (Multiplex on input_4_value)
   // ****** "Amp"******
   // Euroshield cv_waveform_a_amplitude_raw = (input_5_value & cv_waveform_a_amplitude_bits_8_7_6_5) >> 4 ; 

   // Euroshield cv_waveform_a_amplitude_raw = (input_5_value & cv_waveform_a_amplitude_bits_8_7_6_5) >> 4 ; 
   // not setting amp of this now cv_waveform_a_amplitude_raw = (input_5_value); 
   //Serial.println(String("cv_waveform_a_amplitude_raw is: ") + cv_waveform_a_amplitude_raw  );  
   // not setting amp of this now cv_waveform_a_amplitude = fscale( 0, 7, 0.1, 0.99, cv_waveform_a_amplitude_raw, -1.5);
   //Serial.println(String("cv_waveform_a_amplitude is: ") + cv_waveform_a_amplitude  );


   // Put this and above on the inputs.

  
      // Euroshield cv_offset_raw = (input_6_value & bits_2_1);
      // *** Offset ****
      cv_offset_raw = (input_6_value);
      //Serial.println(String("cv_offset_raw is: ") + cv_offset_raw  );
      cv_offset = fscale( min_pot_value, max_pot_value, -1, 1, cv_offset_raw, -1.5);
      //Serial.println(String("cv_offset is: ") + cv_offset  );

 
    // Used for CV
    cv_waveform_a_object.frequency(cv_waveform_a_frequency); // setting-a-freq
    cv_waveform_a_object.amplitude(1);
    //cv_waveform_a_object.amplitude(cv_waveform_a_amplitude); // setting-a-amp

    // Offset
    cv_dc_offset_object.amplitude(cv_offset, 100); // take 100 ms to adjust
    
    //cv_waveform_a_object.offset(cv_offset);




    // MONITOR GATE    
    if (gate_monitor_rms.available())
    {
        float gate_peak = gate_monitor_rms.read();
        Serial.println(String("gate_monitor_rms gate_peak ") + gate_peak  );
        Led1Level(fscale( 0.0, 1.0, 0, 255, gate_peak, 0));
    } else {
      Serial.println(String("gate_monitor_rms not available ")   );
    }
    
    // MONITOR CV
    /// This is connected to cv_waveform and reads the level. We use that to drive the led.
    if (cv_monitor_rms.available())
    {
        float cv_peak = cv_monitor_rms.read();

        // Also use this to drive the betweener ouput
        
          int cv_peak_betweener_scaled = fscale( 0.0, 1.0, 0, 4095, cv_peak, 0); 
          //Serial.println(String("cv_peak_betweener_scaled is: ") + cv_peak_betweener_scaled  );

          Serial.println(cv_peak_betweener_scaled  );

          
          b.writeCVOut(2, cv_peak_betweener_scaled);
       
        
        //Serial.println(String("gate_monitor_rms cv_peak ") + cv_peak  );
        Led4Level(fscale( 0.0, 1.0, 0, 255, cv_peak, 0));
    } else {
      Serial.println(String("cv_monitor_rms not available ")   );
    }


 // return called_on_step;
 
  } // End of ReadInputsAndUpdateSettings
////////////////////////////////////////////////


void InitMidiSequence(){

  Serial.println(String("InitMidiSequence Start ")  );

  // Loop through steps
  for (uint8_t sc = FIRST_STEP; sc <= MAX_STEP; sc++) {
    //Serial.println(String("Step ") + sc );
  
    // Loop through notes
    for (uint8_t n = 0; n <= 127; n++) {
      // Initialise and print Note on (1) and Off (2) contents of the array.
      // WRITE MIDI MIDI_DATA
     channel_a_midi_note_events[sc][n][1].is_active = 0;
     channel_a_midi_note_events[sc][n][0].is_active = 0;

      
      //Serial.println(String("Init Step ") + sc + String(" Note ") + n +  String(" ON ticks value is ") + channel_a_midi_note_events[sc][n][1].is_active);
      //Serial.println(String("Init Step ") + sc + String(" Note ") + n +  String(" OFF ticks value is ") + channel_a_midi_note_events[sc][n][0].is_active);
    } 
  }


  for (uint8_t n = 0; n <= 127; n++) {
     channel_a_ghost_events[n].is_active = 0;
     //Serial.println(String("Init Step with ghost ") + String(" Note ") + n +  String(" is_active false ") );
  } 
  

Serial.println(String("InitMidiSequence Done ")  );
}






/////////////////////////////////////////////////////////////
void OnStep(){

  //Serial.println(String("OnStep step_count is: ") + step_count + String(" Note: FIRST_STEP is: ") + FIRST_STEP  );

  if (step_count > MAX_STEP) {
    Serial.println(String("*****************************************************************************************"));  
    Serial.println(String("************* ERROR! step_count is: ") + step_count + String("*** ERROR *** "));
    Serial.println(String("*****************************************************************************************"));    
  }
  
  uint8_t play_note = bitRead(hybrid_sequence_1, step_count_sanity(step_count));

    if (step_count == FIRST_STEP) {
      CvPulseOn();
    }


   if (play_note){
     //Serial.println(String("****************** play ")   );
        GateHigh();

       // CvPulseOn();

        // We might want to only start the CV pulse on the first step
//        if (reset_cv_lfo_at_FIRST_STEP == true){
//          if (step_count == FIRST_STEP) {
//            CvPulseOn();
//          }
//        } else {
//          CvPulseOn();
//        }
        
       
   } else {
    GateLow();
     //Serial.println(String("not play ")   );
   }
  

    
}

void OnNotStep(){
  //Serial.println(String("NOT step_countIn is: ") + step_countIn  ); 
  GateLow();
  ChangeCvWaveformBAmplitude(); 
}

void GateHigh(){
  //Serial.println(String("Gate HIGH at tick_count_since_start: ") + loop_timing.tick_count_since_start);
  if (IsEuroshield()){
      gate_dc_waveform.amplitude(0.99, 10);
  }



gate_dc_waveform.amplitude(0.99, 10);
    
    b.writeCVOut(1, 4095); //cvout selects channel 1 through 4; value is in range 0-4095


  

}

void GateLow(){
  //Serial.println(String("Gate LOW") );

   gate_dc_waveform.amplitude(0);
    
    b.writeCVOut(1, 0);
  
  
}


void CvPulseOn(){
  //Serial.println(String("CV Pulse On") );
   
      cv_waveform_a_object.phase(90); // Sine wave has maximum at 90 degrees
  
    // Used to modulate CV. This signal is multiplied by cv_waveform 

  // Allow the amplitude to fall to zero before we lift it back up. (if it indeed gets to zero)
  if (cv_waveform_b_amplitude == 0) {
    cv_waveform_b_amplitude = 0.99;
    cv_waveform_b_object.amplitude(cv_waveform_b_amplitude, 10);
  }

}

void ChangeCvWaveformBAmplitude(){
  cv_waveform_b_amplitude -= cv_waveform_b_amplitude_delta;
  if (cv_waveform_b_amplitude <= 0) {
    cv_waveform_b_amplitude = 0;
    }

  cv_waveform_b_object.amplitude(cv_waveform_b_amplitude, 10); // setting-b-amplitude
  //Serial.println(String("cv_waveform_b_amplitude is: ") + cv_waveform_b_amplitude);
 
}


void CvStop(){
  Serial.println(String("CvStop") );
  cv_waveform_b_amplitude = 0;
  cv_waveform_b_object.amplitude(cv_waveform_b_amplitude, 10);
}



//
//void showStepOne(){
//  //Serial.println(String("Clock Show HIGH ") );
//  analogWrite(teensy_led_pin, BRIGHT_5);   // set the LED on
//  //digitalWrite(teensy_led_pin, HIGH);   // set the LED on
//}


void clockShowHigh(){
  //Serial.println(String("Clock Show HIGH ") );
  //analogWrite(teensy_led_pin, BRIGHT_4);   // set the LED on
  digitalWrite(teensy_led_pin, HIGH);   // set the LED on
}

void clockShowLow(){
  //Serial.println(String("Clock Show LOW") );
  //analogWrite(teensy_led_pin, BRIGHT_0);
  digitalWrite(teensy_led_pin, LOW);   // set the LED off
}



//void Led1State(char state){
//  if (state == 'L'){
//    analogWrite(euroshieldLedPins[0], 0);
//  }
//
//  if (state == 'M'){
//    analogWrite(euroshieldLedPins[0], 20);
//  }
//
//  if (state == 'H'){
//    analogWrite(euroshieldLedPins[0], 200);
//  }
//  
//}

void Led1Level(uint8_t level){

    analogWrite(betweener_led_pin, level); 

    if (level > 150){
      digitalWrite(betweener_led_pin, HIGH);
    } else {
      digitalWrite(betweener_led_pin, LOW);
    }

 
 
  
}

void Led2Level(uint8_t level){
  analogWrite(euroshieldLedPins[1], level);
}

void Led3Level(uint8_t level){
  analogWrite(euroshieldLedPins[2], level);
}

void Led4Level(uint8_t level){
  //Serial.println(String("****** Led4Level level ") + level);
  analogWrite(euroshieldLedPins[3], level);
}

void Led4Digital(bool state){
  digitalWrite(euroshieldLedPins[3], state);
}


//////////
bool last_button_1_state = HIGH;

bool Button1HasChanged(bool button_1_state){
  
  bool button_1_has_changed;
  if (button_1_state != last_button_1_state){
    button_1_has_changed = true;
    last_button_1_state = button_1_state;
    //Serial.println(String("button_1_has_changed is: ") + button_1_has_changed + String(" to: ") + button_1_state );
  } else {
    button_1_has_changed = false;
  }
  return button_1_has_changed;
}


void SetSequencePattern(){
    // Button is in Normal state (not pressed) (HIGH) (button_1_state == HIGH)
   // UPPER Pot HIGH Button //////////
   // 8 bit sequence - 8 Least Significant Bits
   last_binary_sequence_1 = binary_sequence_1;

 //  binary_sequence_1 = (input_1_value & sequence_bits_8_through_1) + 1;

   // If we have 8 bits, use the range up to 255

   
   uint8_t binary_sequence_lower_limit = 1;  // Setting to 1 means we never get 0 i.e. a blank sequence especially when we change seq length

   unsigned int binary_sequence_upper_limit; 




// REMEMBER, sequence_length_in_steps is ONE indexed (from 1 up to 16) 
// For a 3 step sequence we want to cover all the possibilities of a 3 step sequence which is (2^3) - 1 = 7
// i.e. all bits on of a 3 step sequence is 111 = 7 decimal 
// or (2^sequence_length_in_steps) - 1
binary_sequence_upper_limit = pow(2, sequence_length_in_steps) - 1; 

   //Serial.println(String("binary_sequence_upper_limit is: ") + binary_sequence_upper_limit  );
    
  binary_sequence_1 = fscale( min_pot_value, max_pot_value, binary_sequence_lower_limit, binary_sequence_upper_limit, input_1_value, 0);

  
   if (binary_sequence_1 != last_binary_sequence_1){
    //Serial.println(String("binary_sequence_1 has changed **"));
   }


//   Serial.println(String("binary_sequence_1 is: ") + binary_sequence_1  );
//   Serial.print("\t");
//   Serial.print(binary_sequence_1, BIN);
//   Serial.println();

   grey_sequence_1 = Binary2Gray(binary_sequence_1);
   //Serial.println(String("grey_sequence_1 is: ") + grey_sequence_1  );
   //Serial.print("\t");
   //Serial.print(grey_sequence_1, BIN);
   //Serial.println();


 
  // Choose which sequence we will actually use.
  // If the pot is left or right, use the hard coded sequence, else use grey code sequence 
//  if (binary_sequence_1 < 20 || binary_sequence_1 > 230 ) {
//      hybrid_sequence_1 = hard_coded_seqs[binary_sequence_1];
//      Serial.println(String("Using hard_coded_seq at position: ") + binary_sequence_1  );
//   } else  {
//    hybrid_sequence_1 = grey_sequence_1;
//    Serial.println(String("Using Grey Code sequence which is: ") + grey_sequence_1  );
//  }


//
//   if (sequence_length_in_steps > 8){
//      hybrid_sequence_1 = grey_sequence_1 + (binary_sequence_1 << 8);
//      Serial.println(String("Using Binary (MSB) + Grey Code ")  );    
//   } else {
//      hybrid_sequence_1 = grey_sequence_1;
//      Serial.println(String("Using Grey Code sequence")   );
//   }





    hybrid_sequence_1 = grey_sequence_1;

    bitClear(hybrid_sequence_1, sequence_length_in_steps -1); // sequence_length_in_steps is 1 based index. bitClear is zero based index.

    hybrid_sequence_1 = ~ hybrid_sequence_1; // Invert

   
    // So pot fully counter clockwise is 1 on the first beat 
    if (binary_sequence_1 == 1){
      hybrid_sequence_1 = 1;
    }


    
    

//   Serial.println(String("hybrid_sequence_1 is: ") + hybrid_sequence_1  );
//   Serial.print("\t");
//   Serial.print(hybrid_sequence_1, BIN);
//   Serial.println();


}


void SetSequenceLength(){

//  Serial.println(String("min_pot_value is: ") + min_pot_value  );
// Serial.println(String("max_pot_value is: ") + max_pot_value  );
//  Serial.println(String("input_2_value is: ") + input_2_value  );


  // NOTE Sometimes we might not get 0 out of a pot - or 1.0 so use the middle range
  sequence_length_in_steps_raw = fscale( min_pot_value, max_pot_value, 0, 15, input_2_value, 0);

  //Serial.println(String("sequence_length_in_steps_raw is: ") + sequence_length_in_steps_raw  );
  // Reverse because we want fully clockwise to be short so we get 1's if sequence is 1.
  sequence_length_in_steps = 16 - sequence_length_in_steps_raw;

  if (sequence_length_in_steps < MIN_SEQUENCE_LENGTH_IN_STEPS){
    sequence_length_in_steps = MIN_SEQUENCE_LENGTH_IN_STEPS; 
    Serial.println(String("**** ERROR with sequence_length_in_steps but it is NOW: ") + sequence_length_in_steps  );
  }
  
  if (sequence_length_in_steps > MAX_SEQUENCE_LENGTH_IN_STEPS){
    sequence_length_in_steps = MAX_SEQUENCE_LENGTH_IN_STEPS; 
    Serial.println(String("**** ERROR with sequence_length_in_steps but it is NOW: ") + sequence_length_in_steps  );
  }

  new_sequence_length_in_ticks = (sequence_length_in_steps) * 6;
  //Serial.println(String("sequence_length_in_steps is: ") + sequence_length_in_steps  ); 
  //Serial.println(String("new_sequence_length_in_ticks is: ") + new_sequence_length_in_ticks  ); 
}


void AdvanceSequenceChronology(){



  
  // This function advances or resets the sequence powered by the clock.

 

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
 
//////////////////////////// 

 int CountSetBits (int x)
    {
        unsigned int count;
        for (count = 0; x; count++)
            x &= x - 1; 
        return count;
    }

void lightStatus(int inputNumber){
  }


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
  // because the value seems to woble, only take the even value for less jitter.
  if (abs(value_1 - value_2) <= fuzzyness){
    return true;
  } else {
    return false;
  }
}



void Flash(int delayTime, int noOfTimes, int ledPin){

 int i;
  
  // blink a few times with a shorter delay each time.
  for (i = 0; i < noOfTimes; i++) {    
    digitalWrite(ledPin, LOW);
    delay(delayTime);
    digitalWrite(ledPin, HIGH);   // set the LED on
    delay(delayTime);// wait 
    digitalWrite(ledPin, LOW);    // set the LED off
    if (delayTime > 15){
      delayTime = delayTime - 10;
    }
  }
    
}



//mydatatype_t stack[MAXSTACKSIZE];
//int stackptr = 0;
//#define push(d) stack[stackptr++] = d
//#define pop stack[--stackptr]
//#define topofstack stack[stackptr - 1]



void DisableNotes(uint8_t note){
             // Disable that note for all steps.
           uint8_t sc = 0;
            for (sc = FIRST_STEP; sc <= MAX_STEP; sc++){
              // WRITE MIDI MIDI_DATA
              channel_a_midi_note_events[sc][note][1].velocity = 0;
              channel_a_midi_note_events[sc][note][1].is_active = 0;
              channel_a_midi_note_events[sc][note][0].velocity = 0;
              channel_a_midi_note_events[sc][note][0].is_active = 0;         
            }
}

//

    
    // We don't want the note on
//    if (channel_a_midi_note_events[step_count][note] == MIDI_NOTE_ON){
//       // If its currently on, set it off.
//       Serial.println(String("Setting MIDI note OFF: ") + note + String(" when step is ") + step_count );
//       channel_a_midi_note_events[step_count][note] = MIDI_NOTE_OFF;
//    } else {
//      // If its not currently on, just set it to unset so we don't send gazzilions of note off messages
//      Serial.println(String("Setting MIDI note UNSET: ") + note + String(" when step is ") + step_count );
//      channel_a_midi_note_events[step_count][note] = MIDI_NOTE_UNSET;
//    }





uint8_t step_count_sanity(uint8_t step_count_){
  uint8_t step_count_fixed;
  
  if (step_count_ > MAX_STEP){
    Serial.println(String("**** ERROR step_count_ > MAX_STEP i.e. ") + step_count_ );
    step_count_fixed = MAX_STEP;
  } else if (step_count_ < FIRST_STEP){
    Serial.println(String("**** ERROR step_count_ > FIRST_STEP i.e. ") + step_count_ );
    step_count_fixed = FIRST_STEP;
  } else {
    step_count_fixed = step_count_;
  }
  return step_count_fixed;
}




/////////////////////////////////////////////
/////////////////////////////////////////////
// https://playground.arduino.cc/Main/Fscale/
// Floating Point Autoscale Function V0.1
// Paul Badger 2007
// Modified from code by Greg Shakar
float fscale( float originalMin, float originalMax, float newBegin, float
newEnd, float inputValue, float curve){

  float OriginalRange = 0;
  float NewRange = 0;
  float zeroRefCurVal = 0;
  float normalizedCurVal = 0;
  float rangedValue = 0;
  boolean invFlag = 0;


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
    NewRange = newEnd - newBegin;
  }
  else
  {
    NewRange = newBegin - newEnd;
    invFlag = 1;
  }

  zeroRefCurVal = inputValue - originalMin;
  normalizedCurVal  =  zeroRefCurVal / OriginalRange;   // normalize to 0 - 1 float

  /*
  Serial.print(OriginalRange, DEC);  
   Serial.print("   ");  
   Serial.print(NewRange, DEC);  
   Serial.print("   ");  
   Serial.println(zeroRefCurVal, DEC);  
   Serial.println();  
   */

  // Check for originalMin > originalMax  - the math for all other cases i.e. negative numbers seems to work out fine
  if (originalMin > originalMax ) {
    return 0;
  }

  if (invFlag == 0){
    rangedValue =  (pow(normalizedCurVal, curve) * NewRange) + newBegin;

  }
  else     // invert the ranges
  {  
    rangedValue =  newBegin - (pow(normalizedCurVal, curve) * NewRange);
  }

  return rangedValue;
}


// From https://gist.github.com/shirish47/d21b896570a8fccbd9c3
unsigned int Binary2Gray(unsigned int data)
 {
   unsigned int n_data=(data>>1);
   n_data=(data ^ n_data);
   
  return n_data;
 }
/////////////////


//void changeStage(int* stage_p){
//    *stage_p = 1;
//}
//
//int main() {
//    //...
//    while(stage!=0){
//        //if snake hits wall
//        changeStage(&stage);
//    }
//}




 
 
