// the setup() method runs once, when the sketch starts

// We use : https://github.com/PaulStoffregen/Audio
// See https://www.pjrc.com/teensy/gui/index.html



const float simon_says_seq_version = 0.23; 


#include <Audio.h>
#include <Wire.h>
#include <SPI.h>
#include <SD.h>
#include <SerialFlash.h>
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
//AudioAnalyzeRMS      rms_L;
//AudioAnalyzeRMS      rms_R;


AudioConnection c1(audioInput, 0, peak_L, 0);
AudioConnection c2(audioInput, 1, peak_R, 0);




AudioControlSGTL5000     audioShield; 

////////////////////////////////
// Operating...
//const int AS_GATE_SEQUENCER = 0;
//const int AS_CV_SEQUENCER = 1;


/////////////
// Setup pins
const int teensy_led_pin = 13;
const int audio1OutPin = 22; 
const int euroshield_button_pin = 2;

const int euroshield_led_pin_count = 4;
const int euroshieldLedPins[euroshield_led_pin_count] = { 6, 5, 4, 3 }; // { 3, 4, 5, 6 }; 

// This the the pin for the upper pot on the Euroshield
const int upper_pot_pin = 20;
// This the the pin for the upper pot on the Euroshield
const int lower_pot_pin = 21;


const int BRIGHT_0 = 0;
const int BRIGHT_1 = 10;
const int BRIGHT_2 = 20;
const int BRIGHT_3 = 75;
const int BRIGHT_4 = 100;
const int BRIGHT_5 = 255;

// Use zero based index for sequencer. i.e. step_count for the first step is 0.
const int FIRST_STEP = 0;
///////////////////////

const int CV_WAVEFORM_B_FREQUENCY_RAW_MAX_INPUT = 15;


////////////////////////////////////////////////////
// Actual pot values
int upper_input_raw;
int lower_input_raw;


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

// The Primary GATE sequence pattern
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
unsigned int sequence_length_in_steps = 8; 

// Used to control when/how we change sequence length 
int previous_sequence_length_in_ticks;
unsigned int new_sequence_length_in_ticks;

// Jitter Reduction: Used to flatten out glitches from the analog pots 
unsigned int jitter_reduction = 0; // was 20

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





////////////////////////////
// LED Display 
unsigned int led_1_level = 0; 


////////////////////////////////////////
// Bit Constants for bit wise operations 
 
unsigned int sequence_bits_8_through_1 = 128 + 64 + 32 + 16 + 8 + 4 + 2 + 1;
  
// unsigned int jitter_reduction_bits_5_4_3_2_1 = 16 + 8 + 4 + 2 + 1; 
unsigned int jitter_reduction_bits_5_4_3_2_1 = 16 + 8 + 4 + 2 + 1; 


unsigned int sequence_length_in_steps_bits_8_7_6 = 128 + 64 + 32;  

unsigned int cv_waveform_a_frequency_raw_bits_8_through_1 = 128 + 64 + 32 + 16 + 8 + 4 + 2 + 1; // CV frequency


unsigned int bits_2_1 = 2 + 1; // CV lfo shape
// how long the CV pulse will last for in terms of ticks
unsigned int cv_waveform_b_frequency_bits_4_3_2_1 = 8 + 4 + 2 + 1; 


 
unsigned int cv_waveform_a_amplitude_bits_8_7_6_5 = 128 + 64 + 32 + 16;
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
    unsigned int tick_count_in_sequence = 0;
    unsigned int tick_count_since_start = 0;
};

// Timing is controlled by the loop. Only the loop should update it.
Timing loop_timing;

// Count of the main pulse i.e. sixteenth notes or eigth notes 
unsigned int step_count;

// Helper functions that operate on global variables. Yae!  

void SetTickCountInSequence(int value){
  loop_timing.tick_count_in_sequence = value;
}

void SetTotalTickCount(int value){
  loop_timing.tick_count_since_start = value;
}

void ResetSequenceCounters(){
  SetTickCountInSequence(0);
  step_count = FIRST_STEP;
  //Serial.println(String("**** ResetSequenceCounters Done *** "));
}


int IncrementStepCount(){
  step_count = step_count + 1;
  return step_count;
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

  int i;
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

   /////////////////////////////////////////////////////////
   // Say hello, show we are ready to sequence. 
  int my_delay_time = 50;
  int my_no_of_times = 10;
  
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
        } else {
          //Serial.println(String("Note Off: ch=") + channel + ", note=" + note);
        }
        break;
      case midi::NoteOff:
        note = MIDI.getData1();
        velocity = MIDI.getData2();
        channel = MIDI.getChannel();
        //Serial.println(String("Note Off: ch=") + channel + ", note=" + note + ", velocity=" + velocity);
        break;
      case midi::Clock:
        // Midi devices sending clock send one of these 24 times per crotchet (a quarter note). 24PPQ
        midi_clock_detected = HIGH;
        Serial.println(String("+++++++++++++++++++++++++++++++++ midi_clock_detected SET TO TRUE is: ") + midi_clock_detected) ;
        note = MIDI.getData1();
        velocity = MIDI.getData2();
        channel = MIDI.getChannel();
        //Serial.println(String("We got Clock: ch=") + channel + ", note=" + note + ", velocity=" + velocity);

        OnClockPulse();


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
        //Serial.println(String("Message, type=") + type);
        //int d1 = MIDI.getData1();
        //int d2 = MIDI.getData2();
        int dummy = 1;
        //Serial.println(String("Message, type=") + type + ", data = " + d1 + " " + d2);
    }
  }


///////////////////

// Analog Clock (and left input checking) //////


    ///////////////////////////////////////////
   // Look for Analogue Clock (24 PPQ)
   // Note: We use this input for other things too.
   if (peak_L.available()){
        left_peak_level = peak_L.read() * 1.0; // minimum seems to be 0.1 from intelij attenuator
        // Serial.println(String("**** left_peak_level: ") + left_peak_level) ;

        //Serial.println(String("analogue_gate_state: ") + analogue_gate_state) ;

       
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
/////////////////////////////////////////////////////////////////////////////////////

}


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

void OnClockPulse(){
          ////////////////////////////////// 
        // This drives sequencer activity.
      
              OnTick(loop_timing);
      
              // Keep track of ticks 
              SetTickCountInSequence(loop_timing.tick_count_in_sequence += 1);
              SetTotalTickCount(loop_timing.tick_count_since_start += 1);
      
       
               // We have 24 ticks per beat 
               // crotchet * 1 = 24 (4 semiquavers)
               // crotchet * 2 = 48 (8 semiquavers)
               // crotchet * 4 = 96 (16 semiauqvers)
      
                previous_sequence_length_in_ticks = new_sequence_length_in_ticks;
                new_sequence_length_in_ticks = (sequence_length_in_steps) * 6;
               //Serial.println(String("new_sequence_length_in_ticks is: ") + new_sequence_length_in_ticks  );  
      
              // Reset every sequence length - or if we change the length so we don't wizz past the end and carry on.
              if ((loop_timing.tick_count_in_sequence >= (new_sequence_length_in_ticks)) && (loop_timing.tick_count_since_start % new_sequence_length_in_ticks == 0) ) { 
                ResetSequenceCounters(); 
              }

}





//////// OnTick (Everytime we get a midi clock pulse) ////////////////////////////
// This is called from the main loop() function on every Midi Clock message.
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
  Serial.println(String("***** upper_input_raw *** is: ") + upper_input_raw  );
  lower_input_raw = analogRead(lower_pot_pin);
  Serial.println(String("*****lower_input_raw *** is: ") + lower_input_raw  );

/////////////////////////////////////////////////////
// Get values from the pots when they are engaged 
// The unpressed state of the button is HIGH
if (button_1_state == HIGH) {
  // only assign it if engaged i.e. we don't want to jump values if the pot is moved inbetween button press
  
   if ((upper_pot_high_engaged == true) || IsCrossing(upper_pot_high_value_at_button_change, upper_input_raw, 5)) {
       int upper_pot_high_value_raw = upper_input_raw;
        Serial.println(String("**** upper_pot_high_value_raw is: ") + upper_pot_high_value_raw  );     
        upper_pot_high_value = GetValue(upper_pot_high_value_raw, upper_pot_high_value_last, jitter_reduction);
        upper_pot_high_value_last = upper_pot_high_value;
        upper_pot_high_engaged = true;
   } else {
     Serial.println(String("upper_pot_high is: DISENGAGED value is sticking at: ") + upper_pot_high_value  ); 
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
     Serial.println(String("lower_pot_low_value_raw is: ") + lower_pot_low_value_raw  );
     lower_pot_low_value = GetValue(lower_pot_low_value_raw, lower_pot_low_value_last, jitter_reduction);
     lower_pot_low_value_last = lower_pot_low_value;
     lower_pot_low_engaged = true;
  } else {
    Serial.println(String("lower_pot_low is: DISENGAGED value is sticking at: ") + lower_pot_low_value  );
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
        Serial.println(String("right_peak_level: ") + right_peak_level );  
    } else {
      Serial.println(String("right_peak_level not available ")   );
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

 
   unsigned int binary_sequence_lower_limit = 1;  // Setting to 1 means we never get 0 i.e. a blank sequence especially when we change seq length
   unsigned int binary_sequence_upper_limit; 


//binary_sequence_upper_limit = pow(sequence_length_in_steps, 2);

// remember sequence_length_in_steps is 1 indexed (from 1 to 16 steps or so) 
// For a 3 step sequence we want to cover all the possibilities of a 3 step sequence which is (2^3) - 1 = 7
// i.e. all bits on of a 3 step sequence is 111 = 7 decimal 
// or (2^sequence_length_in_steps) - 1
binary_sequence_upper_limit = pow(2, sequence_length_in_steps) - 1; 

   Serial.println(String("binary_sequence_upper_limit is: ") + binary_sequence_upper_limit  );
    



  // Generally the lowest value from the pot we get is 2 or 3 
  binary_sequence_1 = fscale( 1, 1023, binary_sequence_lower_limit, binary_sequence_upper_limit, upper_pot_high_value, 0);

   


   if (binary_sequence_1 != last_binary_sequence_1){
    //Serial.println(String("binary_sequence_1 has changed **"));
   }


   Serial.println(String("binary_sequence_1 is: ") + binary_sequence_1  );
   Serial.print("\t");
   Serial.print(binary_sequence_1, BIN);
   Serial.println();

   grey_sequence_1 = Binary2Gray(binary_sequence_1);
   Serial.println(String("grey_sequence_1 is: ") + grey_sequence_1  );
   Serial.print("\t");
   Serial.print(grey_sequence_1, BIN);
   Serial.println();


 
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


    
    //Serial.println(String("Using Grey Code sequence which is: ") + grey_sequence_1  );


   Serial.println(String("hybrid_sequence_1 is: ") + hybrid_sequence_1  );
   Serial.print("\t");
   Serial.print(hybrid_sequence_1, BIN);
   Serial.println();


   
 

  unsigned int sequence_length_in_steps_raw = fscale( 0, 1, 0, 15, right_peak_level, 0);
  // Reverse because we want fully clockwise to be short so we get 1's if sequence is 1.
  sequence_length_in_steps = 16 - sequence_length_in_steps_raw;
   
   
   //((upper_pot_low_value & sequence_length_in_steps_bits_8_7_6) >> 5) + 1; // We want a range 1 - 8
   Serial.println(String("sequence_length_in_steps is: ") + sequence_length_in_steps  );

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
   Serial.println(String("amp_1_gain is: ") + amp_1_gain  );

   amp_1_object.gain(amp_1_gain);
   Led3Level(fscale( 0, 1, 0, BRIGHT_3, amp_1_gain, -1.5));


   ////////////////////////////////////// 
   // CV stuff
   // LOWER Pot HIGH Button //CHECK HERE
   cv_waveform_a_frequency_raw =  (lower_pot_high_value & cv_waveform_a_frequency_raw_bits_8_through_1) >> 0 ; 
   //Serial.println(String("cv_waveform_a_frequency_raw is: ") + cv_waveform_a_frequency_raw  );
   // LFO up to 20Hz
   cv_waveform_a_frequency = fscale( 0, 255, 0.01, 20, cv_waveform_a_frequency_raw, -1.5);
   Serial.println(String("cv_waveform_a_frequency is: ") + cv_waveform_a_frequency  );

   // LOWER Pot LOW Button
   cv_waveform_b_frequency_raw = ((lower_pot_low_value & cv_waveform_b_frequency_bits_4_3_2_1) >> 0);
   Serial.println(String("cv_waveform_b_frequency_raw is: ") + cv_waveform_b_frequency_raw  );


  // if the pot is turned clockwise i.e. the CV lasts for a long time, reset it at step 1.
  if (cv_waveform_b_frequency_raw == CV_WAVEFORM_B_FREQUENCY_RAW_MAX_INPUT){
    reset_cv_lfo_at_FIRST_STEP = true;
    Serial.println(String("reset_cv_lfo_at_FIRST_STEP is: ") + reset_cv_lfo_at_FIRST_STEP);
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
    Serial.println(String("timing.tick_count_in_sequence is: ") + timing.tick_count_in_sequence + String(" the first tick of a crotchet or after MIDI Start message") );    
    step_count = OnStep();
    called_on_step = 1; // For information
  } else {
    clockShowLow();
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
 
  }




int OnStep(){

  Serial.println(String("**** step_count is: ") + step_count  );






  //int play_note = bitRead(binary_sequence_1, step_count);


  //int play_note = bitRead(grey_sequence_1, step_count);

  int play_note = bitRead(hybrid_sequence_1, step_count);
  
  
  //Serial.println(String("play_note is: ") + play_note  );
  //Serial.println(play_note, BIN  );

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

  return IncrementStepCount();
    
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


// HERE

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

void Led1Level(int level){
  analogWrite(euroshieldLedPins[0], level);
}

void Led2Level(int level){
  analogWrite(euroshieldLedPins[1], level);
}

void Led3Level(int level){
  analogWrite(euroshieldLedPins[2], level);
}

void Led4Level(int level){
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
    Serial.println(String("GetValue says use RAW value because diff is ") + diff );
  } else {
    value = last;
    Serial.println(String("GetValue says use LAST value because diff is ") + diff );
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



// this is a map where the keys are strings and the values integers







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
//uint8_t Binary2Gray(uint8_t data)
// {
//   int n_data=(data>>1);
//   n_data=(data ^ n_data);
//   
//  return n_data;
// }


unsigned int Binary2Gray(unsigned int data)
 {
   unsigned int n_data=(data>>1);
   n_data=(data ^ n_data);
   
  return n_data;
 }
 
