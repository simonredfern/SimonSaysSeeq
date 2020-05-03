// the setup() method runs once, when the sketch starts

// We use : https://github.com/PaulStoffregen/Audio
// See https://www.pjrc.com/teensy/gui/index.html



const float simon_says_seq_version = 0.23; 


#include <Audio.h>
#include <MIDI.h>





AudioSynthWaveformDc     gate_dc_waveform;
AudioSynthWaveform       cv_waveform_a_object;      
AudioSynthWaveformDc     cv_waveform_b_object; 
AudioEffectMultiply      multiply1;      

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

// CV Output (via multiply) and Monitor
AudioConnection          patchCord10(amp_1_object, 0, audioOutput, 1); // CV -> Lower Audio Out
AudioConnection          patchCord2(amp_1_object, cv_monitor); // CV -> monitor (for LED)

AudioConnection          patchCord11(multiply1, 0, amp_1_object, 0); // CV -> Lower Audio Out

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

const uint8_t MIN_SEQUENCE_LENGTH_IN_STEPS = 1; // ONE INDEXED
const uint8_t MAX_SEQUENCE_LENGTH_IN_STEPS = 16; // ONE INDEXED

///////////////////////

const uint8_t CV_WAVEFORM_B_FREQUENCY_RAW_MAX_INPUT = 15;


const uint8_t MIDI_NOTE_ON = 1;
const uint8_t MIDI_NOTE_OFF = 0;


////////////////////////////////////////////////////
// Actual pot values
unsigned int upper_input_raw; // TODO Make t type.
unsigned int lower_input_raw;


// Create 4 virtual pots out of two pots and a button.
// To handle the case when 1) Pot is fully counter clockwise 2) We press the button 3) Move the pot fully clockwise 4) Release the button.
// We introduce the concept that a virtual pot can be "engaged" or not so we can catchup with its stored value only when the pot gets back to that position.
bool upper_pot_high_engaged = true;
unsigned int upper_pot_high_value_raw;
unsigned int upper_pot_high_value;
unsigned int upper_pot_high_value_last;
unsigned int upper_pot_high_value_at_button_change;

bool lower_pot_high_engaged = true;
unsigned int lower_pot_high_value_raw;
unsigned int lower_pot_high_value;
unsigned int lower_pot_high_value_last;
unsigned int lower_pot_high_value_at_button_change;

bool upper_pot_low_engaged = true;
unsigned int upper_pot_low_value_raw;
unsigned int upper_pot_low_value;
unsigned int upper_pot_low_value_last;
unsigned int upper_pot_low_value_at_button_change;

bool lower_pot_low_engaged = true;
unsigned int lower_pot_low_value_raw;
unsigned int lower_pot_low_value;
unsigned int lower_pot_low_value_last;
unsigned int lower_pot_low_value_at_button_change;



float left_peak_level;
float right_peak_level;


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

// Jitter Reduction: Used to flatten out glitches from the analog pots 
uint8_t jitter_reduction = 0; // was 20

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
  
  Serial.println(String("ResetSequenceCounters Done. sequence_length_in_steps is ") + sequence_length_in_steps + String(" step_count is now: ") + step_count);
}



uint8_t IncrementStepCount(){
  step_count = step_count_sanity(step_count + 1);

  Serial.println(String("IncrementStepCount. sequence_length_in_steps is ") + sequence_length_in_steps + String(" step_count is now: ") + step_count);
  return step_count_sanity(step_count);
}

boolean midi_clock_detected = LOW;

void setup() {



  // Audio connections require memory to work.  For more
  // detailed information, see the MemoryAndCpuUsage example
  AudioMemory(15);

  // Can set to 10 (default?) or 12 bits. Higher resolutions will result in aproximation.
  // Resolution 
  analogReadResolution(10);

  ///////////////////////////////////////////
  // Pin configuration
  // initialize the digital pin as an output.
  // pinMode(teensy_led_pin, OUTPUT);

  uint8_t i;
  for (i = 0; i < euroshield_led_pin_count; i++) { 
    pinMode(euroshieldLedPins[i], OUTPUT);    
  }

  pinMode(euroshield_button_pin, INPUT);
  ///////////////////////////////////////


  // Begin Midi
  MIDI.begin(MIDI_CHANNEL_OMNI);

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
   // Say hello, show we are ready to sequence. 
  uint8_t my_delay_time = 50;
  uint8_t my_no_of_times = 10;
  
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
  Serial.println(String("Simon Says Seeq! Version: ") + simon_says_seq_version);
  Serial.println(String("audioShield.inputSelect on: ") + AUDIO_INPUT_LINEIN ) ;

// https://en.cppreference.com/w/cpp/types/integer
  Serial.println(String("Max value in INT8_MAX (int8_t): ") + INT8_MAX ) ; // int8_t max value is 127 
   Serial.println(String("Max value in UINT8_MAX (uint8_t): ") + UINT8_MAX ) ; // uint8_t max value is 255
    

}



/////////////// LOOP ////////////////////////////
// the loop() method runs over and over again, as long as the board has power


unsigned long last_clock_pulse=0;

boolean analogue_gate_state = LOW;

boolean sequence_is_running = LOW;




void loop() {


  
int note, velocity, channel, d1, d2;
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
        OnMidiNoteInEvent(MIDI_NOTE_OFF,note, velocity,channel);
        //Serial.println(String("Note Off: ch=") + channel + ", note=" + note + ", velocity=" + velocity);
        break;
      case midi::Clock:
        // Midi devices sending clock send one of these 24 times per crotchet (a quarter note). 24PPQ
        midi_clock_detected = HIGH;
        //Serial.println(String("+++++++++++++++++++++++++++++++++ midi_clock_detected SET TO TRUE is: ") + midi_clock_detected) ;
        note = MIDI.getData1();
        velocity = MIDI.getData2();
        channel = MIDI.getChannel();
        //Serial.println(String("We got Clock: ch=") + channel + ", note=" + note + ", velocity=" + velocity);

        ///////////////////////////////
        // Drive the sequencer via MIDI
        OnClockPulse();
        ///////////////////////////////

        break;
      case midi::Start:
        note = MIDI.getData1();
        velocity = MIDI.getData2();
        channel = MIDI.getChannel();
        StartSequencer();
        break;
      case midi::Stop:
        note = MIDI.getData1();
        velocity = MIDI.getData2();
        channel = MIDI.getChannel();
        StopSequencer();

        //Serial.println(String("We got Stop: ch=") + channel + ", note=" + note + ", velocity=" + velocity);
        break;
      case midi::Continue:
        note = MIDI.getData1();
        velocity = MIDI.getData2();
        channel = MIDI.getChannel();
        //Serial.println(String("We got Continue: ch=") + channel + ", note=" + note + ", velocity=" + velocity);
        break;     
      default:
        Serial.println(String("Message, type=") + type);
        int d1 = MIDI.getData1();
        int d2 = MIDI.getData2();
        int dummy = 1;
        Serial.println(String("Message, type=") + type + ", data = " + d1 + " " + d2);
    }
  } // End of MIDI message detected


///////////////////

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

            // Rising clock edge?
            if ((left_peak_level > 0.5) && (analogue_gate_state == LOW)){
    
              if (sequence_is_running == LOW){
                StartSequencer();
              }
              
              analogue_gate_state = HIGH;
              //Serial.println(String("Went HIGH "));
              OnClockPulse();
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
  Serial.println(String("Start Sequencer "));
  InitSequencer();
  sequence_is_running = HIGH;
}

void StopSequencer(){
  Serial.println(String("Stop Sequencer "));      
  InitSequencer();
  sequence_is_running = LOW;        
}


// Called on Every MIDI / Analogue clock pulse
void OnClockPulse(){
  ////////////////////////////////// 
  // This drives sequencer activity.
      
  OnTick(loop_timing);

  // Keep track of ticks 
  //SetTickCountInSequence(loop_timing.tick_count_in_sequence += 1);
  //SetTotalTickCount(loop_timing.tick_count_since_start += 1);
      

         //////////////
  UpdateSequenceChronology();
  /////////////// 


}





//////// OnTick (Everytime we get a midi clock pulse) ////////////////////////////
// This is called from the main loop() function on every Midi Clock message.
// Things that we want to happen every tick..
int OnTick(Timing timing){
  // Note we set tick_count_in_sequence to 0 following at the stop and start midi messages.
  // The midi clock standard sends 24 ticks per crochet. (quarter note).

 int called_on_step = 0;


  ////////////////////////////////////////////////////////////////
  // Read button state
  int button_1_state = digitalRead(euroshield_button_pin); // Pressed = LOW, Normal = HIGH
  //Serial.println(String("button_1_state is: ") + button_1_state);
  
  int button_1_has_changed = Button1HasChanged(button_1_state);
  //Serial.println(String("button_1_has_changed is: ") + button_1_has_changed);

  /////////////////////////////////////////////////
  // Handle engaging the pots so they change values
  // if Button state changed (pressed or released) store the last state of pots.
  // The pots might have different values for pushed/released states, 
  // so we disengage the pots untill the current value gets close to the previous value for that button state.
  // i.e. Don't use the high (normal) state of the pots until we cross the value again when we are in the HIGH state.
  if (button_1_has_changed) {
    if (button_1_state == LOW){
      
      // Disable the high pots
      upper_pot_high_engaged = false;
      lower_pot_high_engaged = false;

      //Serial.println(String("store values for state_HIGH  "));
      upper_pot_high_value_at_button_change = upper_pot_high_value;
      lower_pot_high_value_at_button_change = lower_pot_high_value;
    
    } else {
      // Disable the low pots
      upper_pot_low_engaged = false;
      lower_pot_low_engaged = false;

      //Serial.println(String("store values for state_LOW "));
      upper_pot_low_value_at_button_change = upper_pot_low_value;
      lower_pot_low_value_at_button_change = lower_pot_low_value;
    }
  }

  ////////////////////////////////////////////
  // Get the Pot positions. 
  // We will later assign the values dependant on the push button state
  upper_input_raw = analogRead(upper_pot_pin);
  //Serial.println(String("***** upper_input_raw *** is: ") + upper_input_raw  );
  lower_input_raw = analogRead(lower_pot_pin);
  //Serial.println(String("*****lower_input_raw *** is: ") + lower_input_raw  );

/////////////////////////////////////////////////////
// Get values from the pots when they are engaged 
// The unpressed state of the button is HIGH
if (button_1_state == HIGH) {
  // only assign it if engaged i.e. we don't want to jump values if the pot is moved inbetween button press
  
   if ((upper_pot_high_engaged == true) || IsCrossing(upper_pot_high_value_at_button_change, upper_input_raw, 5)) {
       int upper_pot_high_value_raw = upper_input_raw;
        //Serial.println(String("**** upper_pot_high_value_raw is: ") + upper_pot_high_value_raw  );     
        upper_pot_high_value = GetValue(upper_pot_high_value_raw, upper_pot_high_value_last, jitter_reduction);
        upper_pot_high_value_last = upper_pot_high_value;
        upper_pot_high_engaged = true;
   } else {
     //Serial.println(String("upper_pot_high is: DISENGAGED value is sticking at: ") + upper_pot_high_value  ); 
   }

   if ((lower_pot_high_engaged == true) || IsCrossing(lower_pot_high_value_at_button_change, lower_input_raw, 5)){
        int lower_pot_high_value_raw = lower_input_raw;
        //Serial.println(String("lower_pot_high_value_raw is: ") + lower_pot_high_value_raw  );
        lower_pot_high_value = GetValue(lower_pot_high_value_raw, lower_pot_high_value_last, jitter_reduction);
        lower_pot_high_value_last = lower_pot_high_value;
        lower_pot_high_engaged = true;
   } else {
    //Serial.println(String("lower_pot_high is: DISENGAGED value is sticking at: ") + lower_pot_high_value  ); 
   }
  
} 
else 
// LOW state
{
  if ((upper_pot_low_engaged == true) || IsCrossing(upper_pot_low_value_at_button_change, upper_input_raw, 5)) {
     int upper_pot_low_value_raw = upper_input_raw;
     //Serial.println(String("upper_pot_low_value_raw is: ") + upper_pot_low_value_raw  );
     upper_pot_low_value = GetValue(upper_pot_low_value_raw, upper_pot_low_value_last, jitter_reduction);
     upper_pot_low_value_last = upper_pot_low_value;
     upper_pot_low_engaged = true;
  } else {
    //Serial.println(String("upper_pot_low is: DISENGAGED value is sticking at: ") + upper_pot_low_value  ); 
  }
  
  if ((lower_pot_low_engaged == true) || IsCrossing(lower_pot_low_value_at_button_change, lower_input_raw, 5)) {  
     int lower_pot_low_value_raw = lower_input_raw;
     //Serial.println(String("lower_pot_low_value_raw is: ") + lower_pot_low_value_raw  );
     lower_pot_low_value = GetValue(lower_pot_low_value_raw, lower_pot_low_value_last, jitter_reduction);
     lower_pot_low_value_last = lower_pot_low_value;
     lower_pot_low_engaged = true;
  } else {
    //Serial.println(String("lower_pot_low is: DISENGAGED value is sticking at: ") + lower_pot_low_value  );
  }

}



  // Get values from analogue ins  

//if (peak_L.available() && peak_R.available() ) {
//   left_peak_level = peak_L.read() * 255.0;
//   Serial.println(String("left_peak_level: ") + left_peak_level ); 
//
//   right_peak_level = peak_R.read() * 255.0;
//   Serial.println(String("right_peak_level: ") + right_peak_level ); 
//  
//} else {
//  Serial.println(String("peak_L or peak_R  not available ")   );
//}


//
//   if (peak_L.available())
//    {
//        left_peak_level = peak_L.read() * 255.0;
//        Serial.println(String("left_peak_level: ") + left_peak_level );  
//    } else {
//      Serial.println(String("left_peak_level not available ")   );
//    }
//
//
//
//  
   if (peak_R.available())
    {
        right_peak_level = peak_R.read() * 1.0;
        //Serial.println(String("right_peak_level: ") + right_peak_level );  
    } else {
      //Serial.println(String("right_peak_level not available ")   );
    }



  // Look for Analogue Sequence start or stop  
//   if (peak_R.available())
//    {
//        float right_peak_level = peak_R.read() * 1.0;
//
//        Serial.println(String("right_peak_level: ") + right_peak_level + String(" sequence_is_running: ") + sequence_is_running);
//
//
//        // Going Up ? - Start
//        if ((right_peak_level > 0.5) && (sequence_is_running == LOW)){
//          sequence_is_running = HIGH;
//          StartSequencer();
//        } 
//
//        // Going Down ? - Stop
//        if ((right_peak_level < 0.5) && (sequence_is_running == HIGH)){
//          sequence_is_running = LOW;
//          StopSequencer();
//        } 
//    
//    } else {
//      //Serial.println(String("gate_monitor not available ")   );
//    }
   ///////////////////////////////////////////////////////////////




//////////////////////////////////////////////

//amp_1_object.gain(1.0);



//////////////////////////////////////////
// Assign values to change the sequencer.
///////////////////////////////////
  // Button is in Normal state (not pressed) (HIGH) (button_1_state == HIGH)
   // UPPER Pot HIGH Button //////////
   // 8 bit sequence - 8 Least Significant Bits
   last_binary_sequence_1 = binary_sequence_1;

 //  binary_sequence_1 = (upper_pot_high_value & sequence_bits_8_through_1) + 1;

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

   //Serial.println(String("binary_sequence_upper_limit is: ") + binary_sequence_upper_limit  );
    



  // Generally the lowest value from the pot we get is 2 or 3 
  binary_sequence_1 = fscale( 1, 1023, binary_sequence_lower_limit, binary_sequence_upper_limit, upper_pot_high_value, 0);

   


   if (binary_sequence_1 != last_binary_sequence_1){
    //Serial.println(String("binary_sequence_1 has changed **"));
   }


   //Serial.println(String("binary_sequence_1 is: ") + binary_sequence_1  );
   //Serial.print("\t");
   //Serial.print(binary_sequence_1, BIN);
   //Serial.println();

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


    
    

   //Serial.println(String("hybrid_sequence_1 is: ") + hybrid_sequence_1  );
   //Serial.print("\t");
   //Serial.print(hybrid_sequence_1, BIN);
   //Serial.println();


   
  //Serial.println(String("right_peak_level is: ") + right_peak_level  );

  // NOTE Sometimes we might not get 0 out of a pot - or 1.0 so use the middle range
  sequence_length_in_steps_raw = fscale( 0.2, 0.9, 0, 15, right_peak_level, 0);


   
   //((upper_pot_low_value & sequence_length_in_steps_bits_8_7_6) >> 5) + 1; // We want a range 1 - 8
   //Serial.println(String("sequence_length_in_steps is: ") + sequence_length_in_steps  );

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
   //jitter_reduction = (upper_pot_low_value & jitter_reduction_bits_5_4_3_2_1) >> 0;
   //Led3Level(fscale( 0, 31, 0, BRIGHT_3, jitter_reduction, -1.5));

   
   //jitter_reduction = fscale( 0, 255, 0, 4, left_peak_level, 0);
   //Serial.println(String("jitter_reduction is: ") + jitter_reduction  );
   //Led3Level(fscale( 0, 255, 0, BRIGHT_3, jitter_reduction, -1.5));



   float amp_1_gain = fscale( 0, 1, 0, 1, left_peak_level, 0);
   //Serial.println(String("amp_1_gain is: ") + amp_1_gain  );

   amp_1_object.gain(amp_1_gain);
   Led3Level(fscale( 0, 1, 0, BRIGHT_3, amp_1_gain, -1.5));


   ////////////////////////////////////// 
   // CV stuff
   // LOWER Pot HIGH Button //CHECK HERE
   cv_waveform_a_frequency_raw =  (lower_pot_high_value & cv_waveform_a_frequency_raw_bits_8_through_1) >> 0 ; 
   //Serial.println(String("cv_waveform_a_frequency_raw is: ") + cv_waveform_a_frequency_raw  );
   // LFO up to 20Hz
   cv_waveform_a_frequency = fscale( 0, 255, 0.01, 20, cv_waveform_a_frequency_raw, -1.5);
   //Serial.println(String("cv_waveform_a_frequency is: ") + cv_waveform_a_frequency  );

   // LOWER Pot LOW Button
   cv_waveform_b_frequency_raw = ((lower_pot_low_value & cv_waveform_b_frequency_bits_4_3_2_1) >> 0);
   //Serial.println(String("cv_waveform_b_frequency_raw is: ") + cv_waveform_b_frequency_raw  );


  // if the pot is turned clockwise i.e. the CV lasts for a long time, reset it at step 1.
  if (cv_waveform_b_frequency_raw == CV_WAVEFORM_B_FREQUENCY_RAW_MAX_INPUT){
    reset_cv_lfo_at_FIRST_STEP = true;
    //Serial.println(String("reset_cv_lfo_at_FIRST_STEP is: ") + reset_cv_lfo_at_FIRST_STEP);
  }


   // We want a value that goes from high to low as we turn the pot to the right.
   // So reverse the number range by subtracting from the maximum value.
   int cv_waveform_b_amplitude_delta_raw = CV_WAVEFORM_B_FREQUENCY_RAW_MAX_INPUT - cv_waveform_b_frequency_raw;
   //Serial.println(String("cv_waveform_b_amplitude_delta_raw is: ") + cv_waveform_b_amplitude_delta_raw  );
   

   cv_waveform_b_amplitude_delta = fscale( 0, CV_WAVEFORM_B_FREQUENCY_RAW_MAX_INPUT, 0.01, 0.4, cv_waveform_b_amplitude_delta_raw, -1.5);
   //Serial.println(String("cv_waveform_b_amplitude_delta is: ") + cv_waveform_b_amplitude_delta  );

   // Lower Pot LOW Button
   cv_waveform_a_amplitude_raw = (lower_pot_low_value & cv_waveform_a_amplitude_bits_8_7_6_5) >> 4 ; 
   //Serial.println(String("cv_waveform_a_amplitude_raw is: ") + cv_waveform_a_amplitude_raw  );  
   cv_waveform_a_amplitude = fscale( 0, 7, 0.1, 0.99, cv_waveform_a_amplitude_raw, -1.5);
   //Serial.println(String("cv_waveform_a_amplitude is: ") + cv_waveform_a_amplitude  );


   // Put this and above on the inputs.

   // TODO Add offset?
   // Lower Pot LOW Button
   //   cv_offset_raw = (lower_pot_low_value & bits_2_1);
   //   Serial.println(String("cv_offset_raw is: ") + cv_offset_raw  );
   //   cv_offset = fscale( 0, 3, 0, 1, cv_offset_raw, -1.5);
   //   Serial.println(String("cv_offset is: ") + cv_offset  );

 
    // Used for CV
    cv_waveform_a_object.frequency(cv_waveform_a_frequency);
    cv_waveform_a_object.amplitude(cv_waveform_a_amplitude);
    cv_waveform_a_object.offset(0);


 //Serial.println(String("timing.tick_count_in_sequence is: ") + timing.tick_count_in_sequence  );    
 

  // Midi provides 24 PPQ (pulses per quarter note) (crotchet). 
  // 
  // We want to "advance" our sequencer every (24/2)/2 = 6 pulses / ticks. (every semi-quaver / "sixteenth" even if we have 8 of them in a sequence)
  // This is independant from the sequence length
  if (timing.tick_count_in_sequence % 6 == 0){
    clockShowHigh();
    //Serial.println(String("timing.tick_count_in_sequence is: ") + timing.tick_count_in_sequence + String(" the first tick of a crotchet or after MIDI Start message") );    
    //////////////////////////////////////////
    OnStep();
    called_on_step = 1; // For information
    /////////////////////////////////////////
    step_count = IncrementStepCount();
    
  } else {
    clockShowLow();
    // The other ticks which are not "steps".
    OnNotStep();
    //Serial.println(String("timing.tick_count_in_sequence is: ") + timing.tick_count_in_sequence );
  }
 
  // Mark the start of the sequence
//  if (timing.tick_count_in_sequence % new_sequence_length_in_ticks == 0){
//    //Serial.println(String("timing.tick_count_in_sequence is: ") + timing.tick_count_in_sequence + String(" the first tick of a crotchet or after MIDI Start message") );    
//    showStepOne();
//  } // assume clockShowLow will turn it off. 

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
 
  } // End of OnTick
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

Serial.println(String("InitMidiSequence Done ")  );
}


/////////////////////////////////////////////////////////////
void OnStep(){

  //Serial.println(String("step_count is: ") + step_count  );

  if (step_count > MAX_STEP) {
    Serial.println(String("*****************************************************************************************"));  
    Serial.println(String("************* ERROR! step_count is: ") + step_count + String("*** ERROR *** "));
    Serial.println(String("*****************************************************************************************"));    
  }



// Serial.println(String("midi_note  ") + i + String(" value is ") + channel_a_midi_note_events[step_count][i]  );

  for (uint8_t n = 0; n <= 127; n++) {
    //Serial.println(String("** OnStep ") + step_count + String(" Note ") + n +  String(" ON value is ") + channel_a_midi_note_events[step_count][n][1]);
    
           //Serial.println(String(" is greater than ") + loop_timing.tick_count_in_sequence );

    // READ MIDI MIDI_DATA
    if (channel_a_midi_note_events[step_count_sanity(step_count)][n][1].is_active == 1) {
       // if (channel_a_midi_note_events[step_count][n][1] >= loop_timing.tick_count_in_sequence){
           Serial.println(String("At step ") + step_count + String(" Send midi_note ON for ") + n );
           // 
           MIDI.sendNoteOn(n, channel_a_midi_note_events[step_count_sanity(step_count)][n][1].velocity, 1);
       // }
    } 

    // READ MIDI MIDI_DATA
    if (channel_a_midi_note_events[step_count_sanity(step_count)][n][0].is_active == 1) {
       // if (channel_a_midi_note_events[step_count][n][0] >= loop_timing.tick_count_in_sequence){
           Serial.println(String("At step ") + step_count + String(" Send midi_note OFF for ") + n );
           MIDI.sendNoteOff(n, 0, 1);
       // }
    }
} // End midi note loop

    
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
  gate_dc_waveform.amplitude(0.99, 10);

}

void GateLow(){
  //Serial.println(String("Gate LOW") );
  gate_dc_waveform.amplitude(0);
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

  cv_waveform_b_object.amplitude(cv_waveform_b_amplitude, 10);
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
    //Serial.println(String("button_1_has_changed is: ") + button_1_has_changed + String(" to: ") + button_1_state );
  } else {
    button_1_has_changed = false;
  }
  return button_1_has_changed;
}


void UpdateSequenceChronology(){


    // Keep track of ticks 
  SetTickCountInSequence(loop_timing.tick_count_in_sequence += 1);
  SetTotalTickCount(loop_timing.tick_count_since_start += 1);

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


   // We have 24 ticks per beat 
   // crotchet * 1 = 24 (4 semiquavers)
   // crotchet * 2 = 48 (8 semiquavers)
   // crotchet * 4 = 96 (16 semiauqvers)


    new_sequence_length_in_ticks = (sequence_length_in_steps) * 6;
    //Serial.println(String("sequence_length_in_steps is: ") + sequence_length_in_steps  ); 
    //Serial.println(String("new_sequence_length_in_ticks is: ") + new_sequence_length_in_ticks  );  



  // Reset every sequence length - or if we change the length so we don't wizz past the end and carry on.
  // HERE! Should be OR below?
  if (
      loop_timing.tick_count_in_sequence >= new_sequence_length_in_ticks 
      && 
      // loop_timing.tick_count_since_start % new_sequence_length_in_ticks == 0 
      loop_timing.tick_count_since_start % 6 == 0 
      ) { 
    ResetSequenceCounters(); 
  }

  
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


void OnMidiNoteInEvent(uint8_t on_off, uint8_t note, uint8_t velocity, uint8_t channel){

  //Serial.println(String("Got MIDI note Event ON/OFF is ") + on_off + String(" Note: ") +  note + String(" Velocity: ") +  velocity + String(" Channel: ") +  channel + String(" when step is ") + step_count );
  if (on_off == MIDI_NOTE_ON){

        if (velocity < 7 ){
            // Disable the note
            Serial.println(String("*** Disable note ") + note + String(" when step is ") + step_count + String(" velocity is ") + velocity );
           // Send Note OFF
           MIDI.sendNoteOff(note, 0, 1);
           
           // Disable that note for all steps.
           uint8_t sc = 0;
            for (sc = 0; sc < 15; sc++){
              // WRITE MIDI MIDI_DATA
              channel_a_midi_note_events[sc][note][1].velocity = 0;
              channel_a_midi_note_events[sc][note][1].is_active = 0;
              channel_a_midi_note_events[sc][note][0].velocity = 0;
              channel_a_midi_note_events[sc][note][0].is_active = 0;         
            }
          
    
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




 
 
