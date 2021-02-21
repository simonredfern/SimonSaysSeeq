// This file is licenced under the MIT license (https://opensource.org/licenses/MIT) except for the code at the end of the file.

//Copyright 2020 Simon Redfern
//
//Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
//
//The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
//
//THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.


/////////////////////////////////
// Simon Says Gray Code Seeq.
// This is a gate sequencer and associated envelope generator and a midi looper.
// We use: https://github.com/PaulStoffregen/Audio See https://www.pjrc.com/teensy/gui/index.html
// This file is programmed to use a: https://1010music.com/euroshield-user-guide (unfortuanatly discontinued) 
// Please see the README for more info. 

const char hardware[16]= "Euroshield";

const float simon_says_seq_version = 0.24
; 


#include <Audio.h>
#include <MIDI.h>


AudioSynthWaveformDc     external_modulator_object;

AudioSynthWaveformDc     gate_dc_waveform;
AudioSynthWaveform       cv_waveform_a_object;      
AudioSynthWaveformDc     cv_waveform_b_object; 
AudioEffectMultiply      multiply1;      
AudioEffectMultiply      multiply2;

AudioAmplifier amp_1_object;
 

AudioInputI2S        audioInput;         // audio shield: mic or line-in
AudioOutputI2S       audioOutput;        // audio shield: headphones & line-out         

AudioAnalyzeRMS          cv_monitor;           
AudioAnalyzeRMS          gate_monitor; 

//////////////////////////
// GATE Output and Monitor
AudioConnection          patchCord3(gate_dc_waveform, 0, audioOutput, 0); // GATE -> Upper Audio Out
AudioConnection          patchCord9(gate_dc_waveform, gate_monitor); // GATE -> montior (for LED)

//////////////
// Modulate CV
AudioConnection          patchCord6(cv_waveform_a_object, 0, multiply1, 1);
AudioConnection          patchCord7(cv_waveform_b_object, 0, multiply1, 0);


// And modulate the result.
AudioConnection          patchCord12(multiply1, 0, multiply2, 1);
AudioConnection          patchCord13(external_modulator_object, 0, multiply2, 0);




// CV Output (via multiply) and Monitor
AudioConnection          patchCord10(amp_1_object, 0, audioOutput, 1); // CV -> Lower Audio Out
AudioConnection          patchCord2(amp_1_object, cv_monitor); // CV -> monitor (for LED)

//AudioConnection          patchCord11(multiply1, 0, amp_1_object, 0); // CV -> Lower Audio Out

AudioConnection          patchCord11(multiply2, 0, amp_1_object, 0); // CV -> Lower Audio Out


AudioAnalyzePeak     peak_L;
AudioAnalyzePeak     peak_R;

AudioConnection c1(audioInput, 0, peak_L, 0);
AudioConnection c2(audioInput, 1, peak_R, 0);

AudioControlSGTL5000     audioShield; 



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


const uint8_t BRIGHT_0 = 0;
const uint8_t BRIGHT_1 = 10;
const uint8_t BRIGHT_2 = 20;
const uint8_t BRIGHT_3 = 75;
const uint8_t BRIGHT_4 = 100;
const uint8_t BRIGHT_5 = 255;

// Use zero based index for sequencer. i.e. step_count for the first step is 0.
const uint8_t FIRST_STEP = 0;
const uint8_t MAX_STEP = 15;

const uint8_t MIN_SEQUENCE_LENGTH_IN_STEPS = 4; // ONE INDEXED
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
unsigned int upper_pot_high_value_raw;
unsigned int upper_pot_high_value = 20;
unsigned int upper_pot_high_value_last;
unsigned int upper_pot_high_value_at_button_change;

//bool lower_pot_high_engaged = true;
unsigned int lower_pot_high_value_raw;
unsigned int lower_pot_high_value = 20;
unsigned int lower_pot_high_value_last;
unsigned int lower_pot_high_value_at_button_change;

//bool upper_pot_low_engaged = true;
unsigned int upper_pot_low_value_raw;
unsigned int upper_pot_low_value = 20;
unsigned int upper_pot_low_value_last;
unsigned int upper_pot_low_value_at_button_change;

//bool lower_pot_low_engaged = true;
unsigned int lower_pot_low_value_raw;
unsigned int lower_pot_low_value = 20;
unsigned int lower_pot_low_value_last;
unsigned int lower_pot_low_value_at_button_change;



float left_peak_level;
float right_peak_level;

float external_modulator_object_level;


////////////////////////////////////////////////////


////////////////////////////////////////////////////
// Musical parameters that the user can tweak.

uint8_t sequence_length_in_steps_raw;


// The Primary GATE sequence pattern // Needs to be upto 16 bits. Maybe more later.
unsigned int binary_sequence;
unsigned int gray_code_sequence;
unsigned int the_sequence;
unsigned int last_binary_sequence; // So we can detect changes

// Sequence Length (and default)
uint8_t sequence_length_in_steps = 8; 

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

  boolean reset_cv_lfo_at_FIRST_STEP = false;

// Amplitude of the LFO
unsigned int cv_waveform_a_amplitude_raw;
float cv_waveform_a_amplitude;

//////////////////////
// Midi clock and start / stop related
MIDI_CREATE_INSTANCE(HardwareSerial, Serial1, MIDI);


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


uint8_t bar_count;

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
  //Serial.println(String("ResetSequenceCounters Done. sequence_length_in_steps is ") + sequence_length_in_steps + String(" step_count is now: ") + step_count);
}



uint8_t IncrementStepCount(){
  step_count = step_count_sanity(step_count + 1);

  Serial.println(String("IncrementStepCount. sequence_length_in_steps is ") + sequence_length_in_steps + String(" step_count is now: ") + step_count);
  return step_count_sanity(step_count);
}



boolean midi_clock_detected = LOW;


 int  min_pot_value;
 int max_pot_value;


void printVersion(){
    Serial.println(String("SimonSaysSeeq! Version: v") +  simon_says_seq_version + String("-") + hardware);
}

void setup() {



  // Audio connections require memory to work.  For more
  // detailed information, see the MemoryAndCpuUsage example
  AudioMemory(15);

  // Can set to 10 (default?) or 12 bits. Higher resolutions will result in aproximation.
  // Resolution 

   int resolution = 10;
  analogReadResolution(resolution);



   min_pot_value = 0;
  max_pot_value = pow(2, resolution) - 1;

    Serial.println(String("resolution is : ") + resolution + String(" bits. The range is ") + min_pot_value + " to " + max_pot_value ) ;


  ///////////////////////////////////////////
  // Pin configuration

  uint8_t i;
  for (i = 0; i < euroshield_led_pin_count; i++) { 
    pinMode(euroshieldLedPins[i], OUTPUT);    
  }

  pinMode(euroshield_button_pin, INPUT);
  ///////////////////////////////////////


  // Begin Midi
  MIDI.begin(MIDI_CHANNEL_OMNI);



  MIDI.turnThruOn(midi::Thru::Full); // Possible values: Off, Full, SameChannel, DifferentChannel


  cv_waveform_a_object.begin(WAVEFORM_SAWTOOTH);
  
  // Enable the audio shield, select input, and enable output
  // This is to read peak.
  audioShield.enable();
  audioShield.inputSelect(AUDIO_INPUT_LINEIN);
  audioShield.volume(0.82);
  audioShield.adcHighPassFilterDisable();
  audioShield.lineInLevel(5,5);

  // Initialise timing
  // Use zero based indexing for step_count so that modular operations (counting multiples of 4, 16 etc are easier)
  step_count = FIRST_STEP;
  loop_timing.tick_count_in_sequence = 0;
  //

  // Show contents / initialise the midi sequence 
 InitMidiSequence();


 


   /////////////////////////////////////////////////////////
   // Say hello by flashing the LEDs, show we are ready to sequence. 
  uint8_t my_delay_time = 50;
  uint8_t my_no_of_times = 10;

  // Get values at setup so isCrossing etc works later
  upper_input_raw = analogRead(upper_pot_pin);
  upper_pot_high_value = upper_input_raw;  
  upper_pot_low_value = upper_input_raw; 

  lower_input_raw = analogRead(lower_pot_pin);
  lower_pot_high_value = lower_input_raw; 
  lower_pot_low_value = lower_input_raw; 
  
  // Say Hello to the Teensy LED 
  Flash(my_delay_time, my_no_of_times, teensy_led_pin);
  
  // Say hello to the Euroshield LEDs 
  for (i = 0; i < euroshield_led_pin_count; i++) { 
     Flash(my_delay_time, my_no_of_times, euroshieldLedPins[i]);
  }
  ////////////////////////////////////////  

  ///////////////////////////////////////
  // Debugging hello
  Serial.begin(57600);
  printVersion();
  
  Serial.println(String("audioShield.inputSelect on: ") + AUDIO_INPUT_LINEIN ) ;

  // https://en.cppreference.com/w/cpp/types/integer
  Serial.println(String("Max value in INT8_MAX (int8_t): ") + INT8_MAX ) ; // int8_t max value is 127 
  Serial.println(String("Max value in UINT8_MAX (uint8_t): ") + UINT8_MAX ) ; // uint8_t max value is 255
   
}






unsigned long last_clock_pulse=0;

boolean analogue_gate_state = LOW;

boolean sequence_is_running = LOW;


/////////////// LOOP ////////////////////////////
// this runs over and over again, as long as the board has power
void loop() {

int note, velocity, channel; 
  if (MIDI.read()) {                    // Is there a MIDI message incoming ?
    byte type = MIDI.getType();
    switch (type) {
      case midi::NoteOn:
        note = MIDI.getData1();
        velocity = MIDI.getData2();
        channel = MIDI.getChannel();
        if (velocity > 0) {
          //Serial.println(String("Note On:  ch=") + channel + ", note=" + note + ", velocity=" + velocity);
          OnMidiNoteInEvent(MIDI_NOTE_ON,note, velocity,channel);
        } else {
          //Serial.println(String("Note Off: ch=") + channel + ", note=" + note);
          OnMidiNoteInEvent(MIDI_NOTE_OFF,note, velocity,channel);
        }
        break;
      case midi::NoteOff:
        note = MIDI.getData1();
        velocity = MIDI.getData2();
        channel = MIDI.getChannel();

       // Before we do something with this note OFF, check we're not expecting a note OFF from the disable mechanism.
       // If so, ignore the Note OFF, else proceed. 
       if (channel_a_ghost_events[note].is_active == 1){
            Serial.println(String("I was expecting a ghost Note OFF for ") + note + String(" Thus I will ignore this Note OFF ") );
            // Reset the ghost event
            channel_a_ghost_events[note].is_active = 0;
       } else {
          OnMidiNoteInEvent(MIDI_NOTE_OFF, note, velocity, channel);
       }
        
        break;
      case midi::Clock:
        // Midi devices sending clock send one of these 24 times per crotchet (a quarter note). 24PPQ
        midi_clock_detected = HIGH;
        //Serial.println(String("+++++++++++++++++++++++++++++++++ midi_clock_detected SET TO TRUE is: ") + midi_clock_detected) ;
//        note = MIDI.getData1();
//        velocity = MIDI.getData2();
//        channel = MIDI.getChannel();
        //Serial.println(String("We got Clock: ch=") + channel + ", note=" + note + ", velocity=" + velocity);

        ///////////////////////////////
        // Drive the sequencer via MIDI
        OnTick();
        ///////////////////////////////



        break;
      case midi::Start:
        StartSequencer();
        break;
      case midi::Stop:
        StopSequencer();

        //Serial.println(String("We got Stop: ch=") + channel + ", note=" + note + ", velocity=" + velocity);
        break;
      case midi::Continue:
        //Serial.println(String("We got Continue: ch=") + channel + ", note=" + note + ", velocity=" + velocity);
        break;     
      default:
        Serial.println(String("Message, type=") + type);
        int d1 = MIDI.getData1();
        int d2 = MIDI.getData2();
        Serial.println(String("**************************** Message, type=") + type + ", data = " + d1 + " " + d2);

        // If sustain pedal pressed, clear the sequence  
        if (type == 176 && d1 == 64 && d2 == 127){
          Serial.println(String("Sustain pedal pressed. Let's clear our sequence..") + type);
          InitMidiSequence();
        }        
    }
  } // End of MIDI message detected


// Analog Clock (and left input checking) //////


    ///////////////////////////////////////////
   // Look for Analogue Clock (24 PPQ)
   // Note: We use this input for other things too.
   if (peak_L.available()){
        left_peak_level = peak_L.read() * 1.0; // minimum seems to be 0.1 from intelij attenuator
        // Serial.println(String("**** left_peak_level: ") + left_peak_level) ;

        //Serial.println(String("analogue_gate_state: ") + analogue_gate_state) ;

         // ONLY if no MIDI clock, run the sequencer from the Analogue clock.
        if (midi_clock_detected == LOW) {

          //Serial.println(String(">>>>>NO<<<<<<< Midi Clock Detected midi_clock_detected is: ") + midi_clock_detected) ;
          
          // Only look for this clock if we don't have midi.

            // Rising clock edge? // state-change-1
            if ((left_peak_level > 0.5) && (analogue_gate_state == LOW)){
    
              if (sequence_is_running == LOW){
                StartSequencer();
              }
              
              analogue_gate_state = HIGH;
              //Serial.println(String("Went HIGH "));
              
              
              
              OnTick();
              last_clock_pulse = millis();
              
            } 
    
            // Falling clock edge?
            if ((left_peak_level < 0.5) && (analogue_gate_state == HIGH)){
              analogue_gate_state = LOW;
              //Serial.println(String("Went LOW "));
            } 

        } // 
        
    } else {
      //Serial.println(String("gate_monitor not available ")   );
    }

 
/////////////////////////////////////////////////////////////////////////////////
// When relying on the analogue clock, we don't have a stop as such, so if we don't detect a clock for a while, then assume its stopped.
// Note that the Beat Step Pro takes a while to kill its clock out after pressing the Stop button.
if (midi_clock_detected == LOW){
  if ((millis() - last_clock_pulse > 500) && (sequence_is_running == HIGH)) {
    Serial.println("No analogue clock for a moment. Stopping sequencer.");

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
  Serial.println(String("Starting Sequencer.."));
  printVersion(); 
  InitSequencer();
  sequence_is_running = HIGH;
}

void StopSequencer(){
  Serial.println(String("Stopping Sequencer.."));
  printVersion();      
  InitSequencer();
  sequence_is_running = LOW;        
}



void OnTick(){
// Called on Every MIDI or Analogue clock pulse
// Drives sequencer settings and activity.

  // Read inputs and update settings.  
  SequenceSettings();

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

  // Play any suitable midi in the sequence 
  PlayMidi();
   
  // Advance and Reset ticks and steps
  AdvanceSequenceChronology();
}





//////// SequenceSettings (Everytime we get a midi clock pulse) ////////////////////////////
// This is called from the main loop() function on every Midi Clock message.
// It contains things that we want to check / happen every tick..
int SequenceSettings(){
  // Note we set tick_count_in_sequence to 0 following stop and start midi messages.
  // The midi clock standard sends 24 ticks per crochet. (quarter note).

 int called_on_step = 0; // not currently used


  ////////////////////////////////////////////////////////////////
  // Read button state
  int button_1_state = digitalRead(euroshield_button_pin); // Pressed = LOW, Normal = HIGH
  //Serial.println(String("button_1_state is: ") + button_1_state);


  // state-change-3
  
  int button_1_has_changed = Button1HasChanged(button_1_state);
  //Serial.println(String("button_1_has_changed is: ") + button_1_has_changed);




  ////////////////////////////////////////////
  // Get the Pot positions. 
  // We will later assign the values dependant on the push button state
  upper_input_raw = analogRead(upper_pot_pin);
  Serial.println(String("***** upper_input_raw *** is: ") + upper_input_raw  );
  lower_input_raw = analogRead(lower_pot_pin);
  Serial.println(String("*****lower_input_raw *** is: ") + lower_input_raw  );


  if ((button_1_state == HIGH) & IsCrossing(upper_pot_high_value, upper_input_raw, FUZZINESS_AMOUNT)) {
    upper_pot_high_value = GetValue(upper_input_raw, upper_pot_high_value, jitter_reduction);
    Serial.println(String("**** NEW value for upper_pot_high_value is: ") + upper_pot_high_value  );
    
  } else {
    Serial.println(String("NO new value for upper_pot_high_value . Sticking at: ") + upper_pot_high_value  );
  }
  
  if ((button_1_state == LOW) & IsCrossing(upper_pot_low_value, upper_input_raw, FUZZINESS_AMOUNT)) {   
    upper_pot_low_value = GetValue(upper_input_raw, upper_pot_low_value, jitter_reduction);
    Serial.println(String("**** NEW value for upper_pot_low_value is: ") + upper_pot_low_value  );
  } else {
    Serial.println(String("NO new value for upper_pot_low_value . Sticking at: ") + upper_pot_low_value  );
  }
  
  if ((button_1_state == HIGH) & IsCrossing(lower_pot_high_value, lower_input_raw, FUZZINESS_AMOUNT)) {    
    lower_pot_high_value = GetValue(lower_input_raw, lower_pot_high_value, jitter_reduction);
    Serial.println(String("**** NEW value for lower_pot_high_value is: ") + lower_pot_high_value  );  
  } else {
    Serial.println(String("NO new value for lower_pot_high_value . Sticking at: ") + lower_pot_high_value  );
  }
  
  
  if ((button_1_state == LOW) & IsCrossing(lower_pot_low_value, lower_input_raw, FUZZINESS_AMOUNT)) {   
    lower_pot_low_value = GetValue(lower_input_raw, lower_pot_low_value, jitter_reduction);
    Serial.println(String("**** NEW value for lower_pot_low_value is: ") + lower_pot_low_value  );
  } else {
    Serial.println(String("NO new value for lower_pot_low_value . Sticking at: ") + lower_pot_low_value  );
  }


//Serial.println(String("**** upper_pot_high_value is now: ") + upper_pot_high_value  ); 
//Serial.println(String("**** upper_pot_low_value is now: ") + upper_pot_low_value  ); 
//Serial.println(String("**** lower_pot_high_value is now: ") + lower_pot_high_value  );
//Serial.println(String("**** lower_pot_low_value is now: ") + lower_pot_low_value  ); 



  
   if (peak_R.available())
    {
        right_peak_level = peak_R.read() * 1.0;
        //Serial.println(String("right_peak_level: ") + right_peak_level );  
    } else {
      //Serial.println(String("right_peak_level not available ")   );
    }

     external_modulator_object_level = right_peak_level;

    external_modulator_object.amplitude(1 - external_modulator_object_level, 10);


//////////////////////////////////////////////

//amp_1_object.gain(1.0);



//////////////////////////////////////////
// Assign values to change the sequencer.
///////////////////////////////////

   // 8 bit sequence - 8 Least Significant Bits
   last_binary_sequence = binary_sequence;

 //  binary_sequence = (upper_pot_high_value & sequence_bits_8_through_1) + 1;

   // If we have 8 bits, use the range up to 255

   
   uint8_t binary_sequence_lower_limit = 1;  // Setting to 1 means we never get 0 i.e. a blank sequence especially when we change seq length
   // TODO Could probably use a smaller type 
   unsigned int binary_sequence_upper_limit; 


//binary_sequence_upper_limit = pow(sequence_length_in_steps, 2);

// REMEMBER, sequence_length_in_steps is ONE indexed (from 1 up to 16) 
// For a 3 step sequence we want to cover all the possibilities of a 3 step sequence which is (2^3) - 1 = 7
// i.e. all bits on of a 3 step sequence is 111 = 7 decimal 
// or (2^sequence_length_in_steps) - 1
binary_sequence_upper_limit = pow(2.0, sequence_length_in_steps) - 1; 

   //Serial.println(String("binary_sequence_upper_limit is: ") + binary_sequence_upper_limit  );
    


  // Button is in Normal state (not pressed) (HIGH) (button_1_state == HIGH)
   // ***UPPER Pot HIGH Button*** //////////
  // Generally the lowest value from the pot we get is 2 or 3 
  // setting-1
  binary_sequence = fscale( 1, 1023, binary_sequence_lower_limit, binary_sequence_upper_limit, upper_pot_high_value, 0);

   


   if (binary_sequence != last_binary_sequence){
    //Serial.println(String("binary_sequence has changed **"));
   }


   //Serial.println(String("binary_sequence is: ") + binary_sequence  );
   //Serial.print("\t");
   //Serial.print(binary_sequence, BIN);
   //Serial.println();

   gray_code_sequence = Binary2Gray(binary_sequence);
   //Serial.println(String("gray_code_sequence is: ") + gray_code_sequence  );
   //Serial.print("\t");
   //Serial.print(gray_code_sequence, BIN);
   //Serial.println();



    the_sequence = gray_code_sequence;

/* Make simpler
    bitClear(the_sequence, sequence_length_in_steps -1); // sequence_length_in_steps is 1 based index. bitClear is zero based index.

    the_sequence = ~ the_sequence; // Invert

   
    // So pot fully counter clockwise is 1 on the first beat 
    if (binary_sequence == 1){
      the_sequence = 1;
    }

*/
    
    

   Serial.println(String("the_sequence is: ") + the_sequence  );
   Serial.print("\t");
   Serial.print(the_sequence, BIN);
   Serial.println();


   
  //Serial.println(String("right_peak_level is: ") + right_peak_level  );

 

// Sequence length raw
// ***UPPER pot LOW value***





// sequence_length_in_steps_raw = fscale( 15, 1023, 0, 15, upper_pot_low_value, 0);   


 sequence_length_in_steps_raw = fscale( min_pot_value, max_pot_value, MIN_SEQUENCE_LENGTH_IN_STEPS, MAX_SEQUENCE_LENGTH_IN_STEPS, upper_pot_low_value, 0);

 
 // Serial.println(String("sequence_length_in_steps is: ") + sequence_length_in_steps  );
   
   //((upper_pot_low_value & sequence_length_in_steps_bits_8_7_6) >> 5) + 1; // We want a range 1 - 8
   

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
   //jitter_reduction = (upper_pot_low_value & jitter_reduction_bits_5_4_3_2_1) >> 0;
   //Led3Level(fscale( 0, 31, 0, BRIGHT_3, jitter_reduction, -1.5));

   
   //jitter_reduction = fscale( 0, 255, 0, 4, left_peak_level, 0);
   //Serial.println(String("jitter_reduction is: ") + jitter_reduction  );
   //Led3Level(fscale( 0, 255, 0, BRIGHT_3, jitter_reduction, -1.5));



 
   float amp_1_gain = fscale( 0, 1, 0, 1, left_peak_level, 0);
   //Serial.println(String("amp_1_gain is: ") + amp_1_gain  );

   amp_1_object.gain(amp_1_gain); // setting-
   //Led3Level(fscale( 0, 1, 0, BRIGHT_3, amp_1_gain, -1.5));


   ////////////////////////////////////// 
   // CV stuff
   // ***LOWER Pot HIGH Button***
   // cv_waveform_a_frequency_raw =  (lower_pot_high_value & cv_waveform_a_frequency_raw_bits_8_through_1) >> 0 ; 
   
   cv_waveform_a_frequency_raw =  lower_pot_high_value; 
   
   //Serial.println(String("cv_waveform_a_frequency_raw is: ") + cv_waveform_a_frequency_raw  );
   // LFO up to 20Hz
   cv_waveform_a_frequency = fscale( 3, 1023, 0.001, 10, cv_waveform_a_frequency_raw, -1.5);
   // Serial.println(String("cv_waveform_a_frequency is: ") + cv_waveform_a_frequency  );
   ////////////////////////

   // ***LOWER Pot LOW Button*** (Multiplex on lower_pot_low_value)
   // cv_waveform_b_frequency_raw = ((lower_pot_low_value & cv_waveform_b_frequency_bits_4_3_2_1) >> 0);
   cv_waveform_b_frequency_raw = lower_pot_low_value;
   //Serial.println(String("cv_waveform_b_frequency_raw is: ") + cv_waveform_b_frequency_raw  );
   ///////////////////////

  // if the pot is turned clockwise i.e. the CV lasts for a long time, reset it at step 1.
  if (cv_waveform_b_frequency_raw > CV_WAVEFORM_B_FREQUENCY_RAW_MAX_INPUT - 20){
    reset_cv_lfo_at_FIRST_STEP = true;
    //Serial.println(String("reset_cv_lfo_at_FIRST_STEP is: ") + reset_cv_lfo_at_FIRST_STEP);
  }


   // We want a value that goes from high to low as we turn the pot to the right.
   // So reverse the number range by subtracting from the maximum value.
   //int cv_waveform_b_amplitude_delta_raw = CV_WAVEFORM_B_FREQUENCY_RAW_MAX_INPUT - cv_waveform_b_frequency_raw;
   
  // int cv_waveform_b_amplitude_delta_raw =  cv_waveform_b_frequency_raw;
   
   //Serial.println(String("cv_waveform_b_amplitude_delta_raw is: ") + cv_waveform_b_amplitude_delta_raw  );
   
   // setting-b-amp-delta
   //cv_waveform_b_amplitude_delta = fscale( 0, CV_WAVEFORM_B_FREQUENCY_RAW_MAX_INPUT, -10, 10, cv_waveform_b_frequency_raw, 1.5) / 100;
   
   cv_waveform_b_amplitude_delta = linearScale( 0, CV_WAVEFORM_B_FREQUENCY_RAW_MAX_INPUT, 0.0, 0.3, cv_waveform_b_frequency_raw);
   

   
   //Serial.println(String("cv_waveform_b_amplitude_delta is: ") + cv_waveform_b_amplitude_delta  );

   // Lower Pot LOW Button (Multiplex on lower_pot_low_value)
   //cv_waveform_a_amplitude_raw = (lower_pot_low_value & cv_waveform_a_amplitude_bits_8_7_6_5) >> 4 ; 
   //Serial.println(String("cv_waveform_a_amplitude_raw is: ") + cv_waveform_a_amplitude_raw  );  
   //cv_waveform_a_amplitude = fscale( 0, 7, 0.1, 0.99, cv_waveform_a_amplitude_raw, -1.5);
   //Serial.println(String("cv_waveform_a_amplitude is: ") + cv_waveform_a_amplitude  );

   cv_waveform_a_amplitude = 0.99;

   // Put this and above on the inputs.

   // TODO Add offset?
   // Lower Pot LOW Button
   //   cv_offset_raw = (lower_pot_low_value & bits_2_1);
   //   Serial.println(String("cv_offset_raw is: ") + cv_offset_raw  );
   //   cv_offset = fscale( 0, 3, 0, 1, cv_offset_raw, -1.5);
   //   Serial.println(String("cv_offset is: ") + cv_offset  );

 
    // Used for CV
    cv_waveform_a_object.frequency(cv_waveform_a_frequency); // setting-a-freq
    cv_waveform_a_object.amplitude(cv_waveform_a_amplitude); // setting-a-amp
    cv_waveform_a_object.offset(0);




    // MONITOR GATE    
    if (gate_monitor.available())
    {
        float gate_peak = gate_monitor.read();
        //Serial.println(String("gate_monitor gate_peak ") + gate_peak  );
        Led1Level(fscale( 0.0, 1.0, 0, 255, gate_peak, 0));
    } else {
      //Serial.println(String("gate_monitor not available ")   );
    }
    
    // MONITOR CV
    /// This is connected to cv_waveform and reads the level. We use that to drive the led.
    if (cv_monitor.available())
    {
        float cv_peak = cv_monitor.read();
        //Serial.println(String("gate_monitor cv_peak ") + cv_peak  );
        Led4Level(fscale( 0.0, 1.0, 0, 255, cv_peak, 0));
    } else {
      //Serial.println(String("gate_monitor not available ")   );
    }


  return called_on_step;
 
  } // End of SequenceSettings
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
     Serial.println(String("Init Step with ghost ") + String(" Note ") + n +  String(" is_active false ") );
  } 
  

Serial.println(String("InitMidiSequence Done ")  );
}



void PlayMidi(){
  // Serial.println(String("midi_note  ") + i + String(" value is ") + channel_a_midi_note_events[step_count][i]  );



  for (uint8_t n = 0; n <= 127; n++) {
    //Serial.println(String("** OnStep ") + step_count + String(" Note ") + n +  String(" ON value is ") + channel_a_midi_note_events[step_count][n][1]);
    
    // READ MIDI MIDI_DATA
    if (channel_a_midi_note_events[step_count_sanity(step_count)][n][1].is_active == 1) { 
           // The note could be on one of 6 ticks in the sequence
           if (channel_a_midi_note_events[step_count_sanity(step_count)][n][1].tick_count_in_sequence == loop_timing.tick_count_in_sequence){
             // Serial.println(String("Step:Ticks ") + step_count + String(":") + ticks_after_step + String(" Found and will send Note ON for ") + n );
             MIDI.sendNoteOn(n, channel_a_midi_note_events[step_count_sanity(step_count)][n][1].velocity, 1);
           }
    } 

    // READ MIDI MIDI_DATA
    if (channel_a_midi_note_events[step_count_sanity(step_count)][n][0].is_active == 1) {
       if (channel_a_midi_note_events[step_count_sanity(step_count)][n][0].tick_count_in_sequence == loop_timing.tick_count_in_sequence){ 
           // Serial.println(String("Step:Ticks ") + step_count + String(":") + ticks_after_step +  String(" Found and will send Note OFF for ") + n );
           MIDI.sendNoteOff(n, 0, 1);
       }
    }
} // End midi note loop

}



/////////////////////////////////////////////////////////////
// These are the possible beats of the sequence
void OnStep(){

  

  //Serial.println(String("OnStep ") + step_count  );

  if (step_count > MAX_STEP) {
    Serial.println(String("----------------------------------------------------------------------------"));  
    Serial.println(String("------------------ ERROR! step_count is: ") + step_count + String("--- ERROR --- "));
    Serial.println(String("----------------------------------------------------------------------------"));    
  }

  

    if (step_count == FIRST_STEP) {
      SyncAndResetCv();
    } else {
      ChangeCvWaveformBAmplitude();
    }
  
  uint8_t play_note = bitRead(the_sequence, step_count_sanity(step_count));
  
   if (play_note){
     //Serial.println(String("****************** play ")   );
    GateHigh(); 
   } else {
    GateLow();
     //Serial.println(String("not play ")   );
   }

   
      
}



// These are ticks which are not steps - so in between possible beats.
void OnNotStep(){
  //Serial.println(String("NOT step_countIn is: ") + step_countIn  ); 
  ChangeCvWaveformBAmplitude(); 
  GateLow();
  
}

void GateHigh(){
  //Serial.println(String("Gate HIGH at tick_count_since_start: ") + loop_timing.tick_count_since_start);
  gate_dc_waveform.amplitude(0.99, 10);

}

void GateLow(){
  //Serial.println(String("Gate LOW") );
  gate_dc_waveform.amplitude(0);
}


// Kind of syncs the CV 
void SyncAndResetCv(){
  //Serial.println(String("CV Pulse On") );
   
  cv_waveform_a_object.phase(90); // Sine wave has maximum at 90 degrees
  
    // Used to modulate CV. This signal is multiplied by cv_waveform 

  // Allow the amplitude to fall to zero before we lift it back up. (if it indeed gets to zero)
  
  if (RampIsPositive()){
    if (cv_waveform_b_amplitude == 1) {
      cv_waveform_b_amplitude = 0;
      Serial.println(String("SyncAndResetCv : 0") );
      
      SetWaveformBObjectAmplitude ();
    }

  } else {
    if (cv_waveform_b_amplitude == 0) {
      cv_waveform_b_amplitude = 1;

       Serial.println(String("SyncAndResetCv : 1") );
      
      SetWaveformBObjectAmplitude ();
    }
  }


}


bool RampIsPositive(){
  if (cv_waveform_b_amplitude_delta > 0)
  {
    return true;
  } 
  else 
  {
    return false;
  }
  
}

void ChangeCvWaveformBAmplitude(){
  
  // change by an amount (might go up or down)
  cv_waveform_b_amplitude += cv_waveform_b_amplitude_delta;

  SetWaveformBObjectAmplitude ();
 
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
  cv_waveform_b_object.amplitude(cv_waveform_b_amplitude, 10); // setting-b-amplitude
  //Serial.println(String("cv_waveform_b_object.amplitude was set to: ") + cv_waveform_b_amplitude);
}



void CvStop(){
  Serial.println(String("CvStop") );
  cv_waveform_b_amplitude = 0;
  cv_waveform_b_object.amplitude(cv_waveform_b_amplitude, 10);
}






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



void Led1State(char state){
  if (state == 'L'){
    analogWrite(euroshieldLedPins[0], 0);
  }

  if (state == 'M'){
    analogWrite(euroshieldLedPins[0], 20);
  }

  if (state == 'H'){
    analogWrite(euroshieldLedPins[0], 200);
  }
  
}

void Led1Level(uint8_t level){
  analogWrite(euroshieldLedPins[0], level);
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
    Serial.println(String("button_1_has_changed is: ") + button_1_has_changed + String(" to: ") + button_1_state );
  } else {
    button_1_has_changed = false;
  }
  return button_1_has_changed;
}


void AdvanceSequenceChronology(){



  
  // This function advances or resets the sequence powered by the clock.

  // But first check / set the desired sequence length


    //Serial.println(String("sequence_length_in_steps_raw is: ") + sequence_length_in_steps_raw  );
  // Reverse because we want fully clockwise to be short so we get 1's if sequence is 1.
  // sequence_length_in_steps = 16 - sequence_length_in_steps_raw;

  sequence_length_in_steps = sequence_length_in_steps_raw;

  Serial.println(String("sequence_length_in_steps is: ") + sequence_length_in_steps  );

  if (sequence_length_in_steps < MIN_SEQUENCE_LENGTH_IN_STEPS){
    Serial.println(String("**** ERROR with sequence_length_in_steps it WAS: ") + sequence_length_in_steps  + String(" but setting it to: ") + MIN_SEQUENCE_LENGTH_IN_STEPS );
    sequence_length_in_steps = MIN_SEQUENCE_LENGTH_IN_STEPS; 
    
  }
  
  if (sequence_length_in_steps > MAX_SEQUENCE_LENGTH_IN_STEPS){
    sequence_length_in_steps = MAX_SEQUENCE_LENGTH_IN_STEPS; 
    Serial.println(String("**** ERROR with sequence_length_in_steps but it is NOW: ") + sequence_length_in_steps  );
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
  // Return true if the two values are close
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


void OnMidiNoteInEvent(uint8_t on_off, uint8_t note, uint8_t velocity, uint8_t channel){

  //Serial.println(String("Got MIDI note Event ON/OFF is ") + on_off + String(" Note: ") +  note + String(" Velocity: ") +  velocity + String(" Channel: ") +  channel + String(" when step is ") + step_count );
  if (on_off == MIDI_NOTE_ON){

        // A mechanism to clear notes from memory by playing them quietly.
        if (velocity < 7 ){
           // Send Note OFF
           MIDI.sendNoteOff(note, 0, 1);
           
           // Disable the note on all steps
           Serial.println(String("DISABLE Note (for all steps) ") + note + String(" because ON velocity is ") + velocity );
           DisableNotes(note);

          // Now, when we release this note on the keyboard, the keyboard obviously generates a note off which gets stored in channel_a_midi_note_events
          // and can interfere with subsequent note ONs i.e. cause the note to end earlier than expected.
          // Since velocity of Note OFF is not respected by keyboard manufactuers, we need to find a way remove (or prevent?)
          // these Note OFF events. 
          // One way is to store them here for processing after the note OFF actually happens. 
          channel_a_ghost_events[note].is_active=1;
    
        } else {
          // We want the note on, so set it on.
          Serial.println(String("Setting MIDI note ON for note ") + note + String(" when step is ") + step_count + String(" velocity is ") + velocity );
          // WRITE MIDI MIDI_DATA
          channel_a_midi_note_events[step_count][note][1].tick_count_in_sequence = loop_timing.tick_count_in_sequence; // Only one of these per step.
          channel_a_midi_note_events[step_count][note][1].velocity = velocity;
          channel_a_midi_note_events[step_count][note][1].is_active = 1;
           Serial.println(String("Done setting MIDI note ON for note ") + note + String(" when step is ") + step_count + String(" velocity is ") + velocity );

        } 
      
          
        } else {
          
            // Note Off
             Serial.println(String("Setting MIDI note OFF for note ") + note + String(" when step is ") + step_count );
             // WRITE MIDI MIDI_DATA
             channel_a_midi_note_events[step_count][note][0].tick_count_in_sequence = loop_timing.tick_count_in_sequence;
             channel_a_midi_note_events[step_count][note][0].velocity = velocity;
             channel_a_midi_note_events[step_count][note][0].is_active = 1;
             Serial.println(String("Done setting MIDI note OFF for note ") + note + String(" when step is ") + step_count );

          
  }
  } 

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



 
 
