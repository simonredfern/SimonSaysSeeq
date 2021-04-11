// For important notes on Compiling / Running this sketch, please see the README.md in this folder.
// For usage instructions also see the README.md in this folder

const float simon_says_seq_version = 0.27;


//////////////////////////////////////////////////////////
#include <MIDI.h> // Note use of Serial1 / Serial2 below. Serial1 for Euroshield it seems but can't use that for Betweener



#include <Betweener.h>
#include <Audio.h>

Betweener b; // Must begin it below!

AudioSynthWaveform       test_waveform_object;
AudioAnalyzeRMS          test_rms_object;



AudioConnection          patchCordTest1(test_waveform_object, test_rms_object);

AudioSynthWaveformDc     gate_dc_waveform;


AudioSynthWaveformDc     gate_2_dc_waveform;

/////////////////////////////////////
// How we generate CV

//  carrier a ---  (no params)                 (gain = 0 to 32767.0)
//                |---Multiply (amplitude modulation)---||---RMS Monitor(0.0-1.0)---|(Scale in code)|---(0-4095)WriteCV---|Output
//  modulator a (dc ramp) ---
//
//

AudioSynthWaveform       carrier_a_object;
AudioSynthWaveformDc     modulator_a_object;
AudioEffectMultiply      multiply_a;
AudioAnalyzeRMS          result_a_rms_object;


AudioSynthWaveform       carrier_b_object;
AudioSynthWaveformDc     modulator_b_object;
AudioEffectMultiply      multiply_b;
AudioAnalyzeRMS          result_b_rms_object;


//////////////
// Modulate CV
// Carier (LFO) and Modulator (Decaying Ramp) A multiplied together
AudioConnection          patchCord6a(carrier_a_object, 0, multiply_a, 1);
AudioConnection          patchCord7a(modulator_a_object, 0, multiply_a, 0);

// Carier (LFO) and Modulator (Decaying Ramp) B multiplied together
AudioConnection          patchCord6b(carrier_b_object, 0, multiply_b, 1);
AudioConnection          patchCord7b(modulator_b_object, 0, multiply_b, 0);



// CV Output is via result_a_rms_object which we read in code.
AudioConnection          patchCord2a(multiply_a, result_a_rms_object); // CV -> monitor to drive output indirectly

AudioConnection          patchCord2b(multiply_b, result_b_rms_object);

////////////////////////////////////////////

AudioInputI2S        audioInput;         // audio shield: mic or line-in
AudioOutputI2S       audioOutput;        // audio shield: headphones & line-out

AudioAnalyzeRMS          gate_monitor_rms;

//////////////////////////
// GATE Output and Monitor

AudioConnection          patchCord9(gate_dc_waveform, gate_monitor_rms); // GATE -> montior (for LED)


AudioControlSGTL5000     audioShield;



/////////////
// Setup pins
const uint8_t teensy_led_pin = 13;
const uint8_t audio1OutPin = 22;



// This the the pin for the upper pot on the Euroshield
const uint8_t upper_pot_pin = 20;
// This the the pin for the upper pot on the Euroshield
const uint8_t lower_pot_pin = 21;

const uint8_t betweener_led_pin = 8;



const uint8_t BRIGHT_0 = 0;
const uint8_t BRIGHT_1 = 10;
const uint8_t BRIGHT_2 = 20;
const uint8_t BRIGHT_3 = 75;
const uint8_t BRIGHT_4 = 100;
const uint8_t BRIGHT_5 = 255;

// Use zero based index for sequencer. i.e. step_count for the first step is 0.
const uint8_t FIRST_STEP = 0;
const uint8_t MAX_STEP = 15;

// Use zero based index for bar.
const uint8_t FIRST_BAR = 0;
const uint8_t MAX_BAR = 1; // Memory User!


const uint8_t MIN_SEQUENCE_LENGTH_IN_STEPS = 4; // ONE INDEXED
const uint8_t MAX_SEQUENCE_LENGTH_IN_STEPS = 16; // ONE INDEXED

///////////////////////

const int CV_MODULATOR_A_FREQUENCY_RAW_MAX_INPUT = 1023;


const uint8_t MIDI_NOTE_ON = 1;
const uint8_t MIDI_NOTE_OFF = 0;




unsigned int min_pot_value;
unsigned int max_pot_value;


////////////////////////////////////////////////////


unsigned int pot1_input_value;

unsigned int pot2_input_value;

unsigned int pot3_input_value;

unsigned int pot4_input_value;




float cv1_input_value;
float cv2_input_value;
float cv3_input_value;
float cv4_input_value;

////////////////////////////////////////////////////


////////////////////////////////////////////////////
// Musical parameters that the user can tweak.

uint8_t sequence_length_in_steps_raw;


// The Primary GATE sequence pattern // Needs to be upto 16 bits. Maybe more later.
unsigned int binary_sequence_1;
unsigned int grey_sequence_1;
unsigned int hybrid_sequence_1;
unsigned int last_binary_sequence_1; // So we can detect changes







/*

   python:


  ---X---X---X---X
  python_bd_list = [
  0B0000000000000001,
  0B0000000100000001,
  0B0001000100010001,
  0B0000000100010001,
  0B0001001100011001,
  0B0010000100101001,
  0B0001010110110001,
  0B0000100010010001,
  0B1000100100010001,
  0B0010000100001001,
  0B1001000100010001,
  0B1001100010010001,
  0B1001000010010011,
  0B1000100110011001,
  0B0011000100010001,
  0B0001000100010001]

  >>> for x in python_bd_list:
  ...     print (str(x)+",")


*/


long bd_seqs[] = { // can use the python interpreter above to get these. (can't use literals for long sizes)
  1,
  257,
  4369,
  273,
  4889,
  8489,
  5553,
  2193,
  35089,
  8457,
  37137,
  39057,
  37011,
  35225,
  12561,
  4369
  // 65535 // All 1's
};


// Sequence Length
uint8_t sequence_length_in_steps = 8;

// Used to control when/how we change sequence length

uint8_t new_sequence_length_in_ticks;

// Just counts 0 to 5 within each step
uint8_t ticks_after_step;

// Jitter Reduction: Used to flatten out glitches from the analog pots
uint8_t jitter_reduction = 0; // was 20

// LFO
unsigned int cv_carrier_a_frequency_raw;


float cv_carrier_a_frequency;
float cv_carrier_b_frequency;

boolean reset_cv_lfo_at_FIRST_STEP = false;

// Amplitude of the LFO

float cv_waveform_a_amplitude;



float cv_offset;


//////////////////////
// Midi clock and start / stop related
// We use the following library  https://github.com/FortySevenEffects/arduino_midi_library/wiki/Using-custom-Settings

MIDI_CREATE_INSTANCE(HardwareSerial, Serial1, MIDI); // This was Euroshield

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
   uint8_t tick_count_since_step = 0; 
   boolean is_active = 0;
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
NoteInfo channel_a_midi_note_events[MAX_BAR+1][MAX_STEP+1][128][2]; 
////////


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





///////////////////////////
// Values set via the Pots (and later maybe audio inputs).
unsigned int cv_modulator_a_frequency_raw = 0;
float cv_waveform_b_frequency = 0;

float cv_modulator_a_amplitude;
float cv_modulator_a_amplitude_delta;

float cv_modulator_b_amplitude;
float cv_modulator_b_amplitude_delta;


// Timing 
// count the ticks (24 per quarter note / crotchet since the last crotchet or start)

struct Timing
{
    uint8_t tick_count_in_sequence = 0; // Since we started the sequencer  
    uint32_t tick_count_since_start = 0; // since the clock started running this time.
    uint8_t tick_count_since_step = 0; // between 0 and 5 as there are 6 ticks in a step
};




// Timing is controlled by the loop. Only the loop should update it.
Timing loop_timing;

// Count of the main pulse i.e. sixteenth notes or eigth notes
uint8_t step_count;

// Count of the bar / measure.
uint8_t bar_count;

// Helper functions that operate on global variables. Yae!

void SetTickCountInSequence(uint8_t value) {
  loop_timing.tick_count_in_sequence = value;
}

void SetTotalTickCount(int value) {
  loop_timing.tick_count_since_start = value;
}

void ResetSequenceCounters() {
  SetTickCountInSequence(0);
  step_count = FIRST_STEP;
  // TODO Changes to Sequence Length should be done here / or when done this function should be called immediately.

  Serial.println(String("ResetSequenceCounters Done. sequence_length_in_steps is ") + sequence_length_in_steps + String(" step_count is now: ") + step_count);
}



uint8_t IncrementStepCount() {
  step_count = step_count_sanity(step_count + 1);

  Serial.println(String("IncrementStepCount. sequence_length_in_steps is ") + sequence_length_in_steps + String(" step_count is now: ") + step_count);
  return step_count_sanity(step_count);
}



boolean midi_clock_detected = LOW;

boolean do_reset_sequence_counters = false;





void setup() {

  // Debugging hello
  //Serial.begin(57600);
  Serial.begin(115200);
  Serial.println(String("Hello from SimonSaysSeeq Version: ") + simon_says_seq_version);


  int resolution = 10;



  //Serial.println(String("I'm expecting to be plugged into a Betweener. Starting b...")) ;

  //////////
  b.begin();
  /////////


  resolution = 10; // Need to use 10 bits for Betweener

  analogReadResolution(resolution);

  min_pot_value = 0;
  max_pot_value = pow(2, resolution) - 1;

  Serial.println(String("resolution is : ") + resolution + String(" bits. The range is ") + min_pot_value + " to " + max_pot_value ) ;

  //Serial.println(String("audioShield.inputSelect on: ") + AUDIO_INPUT_LINEIN ) ;

  // Audio connections require memory to work.  For more
  // detailed information, see the MemoryAndCpuUsage example
  AudioMemory(15);


  ///////////////////////////////////////////
  // Pin configuration
  // initialize the digital pin as an output.
  // pinMode(teensy_led_pin, OUTPUT);

  //uint8_t i;

  pinMode(betweener_led_pin, OUTPUT);

  ///////////////////////////////////////


  // Begin Midi
  //MIDI.begin(MIDI_CHANNEL_OMNI);



  //MIDI.turnThruOn(midi::Thru::Full); //  Off, Full, SameChannel, DifferentChannel




  carrier_a_object.begin(WAVEFORM_SAWTOOTH);
  carrier_a_object.frequency(1);
  carrier_a_object.amplitude(1);
  carrier_a_object.offset(0);


  carrier_b_object.begin(WAVEFORM_SINE);
  carrier_b_object.frequency(1);
  carrier_b_object.amplitude(1);
  carrier_b_object.offset(0);


  ////////////////////////////
  // TEST OBJECT /////////////
  test_waveform_object.begin(WAVEFORM_SAWTOOTH);
  test_waveform_object.frequency(1); // setting-a-freq
  test_waveform_object.amplitude(1); // TEMP cv_waveform_a_amplitude); // setting-a-amp
  test_waveform_object.offset(0);
  ///////////////////////////////





  // Enable the audio shield, select input, and enable output
  // This is to read peak.



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





  // Say hello to Betweener. There is only one LED on the Betweener, so make it more obvious
  Flash(my_delay_time * 2, my_no_of_times * 2, betweener_led_pin);


  // https://en.cppreference.com/w/cpp/types/integer
  //Serial.println(String("Max value in INT8_MAX (int8_t): ") + INT8_MAX ) ; // int8_t max value is 127
  //Serial.println(String("Max value in UINT8_MAX (uint8_t): ") + UINT8_MAX ) ; // uint8_t max value is 255


}



/////////////// LOOP ////////////////////////////
// the loop() method runs over and over again, as long as the board has power


unsigned long last_clock_pulse = 0;

boolean analogue_gate_state = LOW;

boolean sequence_is_running = LOW;


boolean mute_gate_a = false;
boolean mute_gate_b = false;


// copy paste betweener-teensy
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
          Serial.println(String("Setting MIDI note ON for note ") + note + String(" when bar is ") + bar_count + String(" when step is ") + step_count + String(" velocity is ") + velocity );
          // WRITE MIDI MIDI_DATA
          channel_a_midi_note_events[bar_count][step_count][note][1].tick_count_since_step = loop_timing.tick_count_since_step; // Only one of these per step.
          channel_a_midi_note_events[bar_count][step_count][note][1].velocity = velocity;
          channel_a_midi_note_events[bar_count][step_count][note][1].is_active = 1;
           Serial.println(String("Done setting MIDI note ON for note ") + note + String(" when bar is ") + bar_count + String(" when step is ") + step_count + String(" velocity is ") + velocity );

        } 
      
          
        } else {
          
            // Note Off
             Serial.println(String("Setting MIDI note OFF for note ") + note + String(" when bar is ") + bar_count + String(" when step is ") + step_count );
             // WRITE MIDI MIDI_DATA
             channel_a_midi_note_events[bar_count][step_count][note][0].tick_count_since_step = loop_timing.tick_count_since_step;
             channel_a_midi_note_events[bar_count][step_count][note][0].velocity = velocity;
             channel_a_midi_note_events[bar_count][step_count][note][0].is_active = 1;
             Serial.println(String("Done setting MIDI note OFF for note ") + note + String(" when bar is ") + bar_count + String(" when step is ") + step_count );

          
  }
  } 



void loop() {

  // The way we "connect" the Audio Objects to the output of the Betweener is to read an rms object and use that value
  // to writeCVOut.


/////////////



int note, velocity, channel; 
  if (MIDI.read()) {                    // Is there a MIDI message incoming ?
    byte type = MIDI.getType();
    switch (type) {
      case midi::NoteOn:
        note = MIDI.getData1();
        velocity = MIDI.getData2();
        channel = MIDI.getChannel();
        if (velocity > 0) {
          Serial.println(String("Note On:  ch=") + channel + ", note=" + note + ", velocity=" + velocity);
          OnMidiNoteInEvent(MIDI_NOTE_ON,note, velocity,channel);
        } else {
          Serial.println(String("Note Off: ch=") + channel + ", note=" + note);
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
    else {
  // Serial.println(String("No MIDI message found"));
}




/////////////

  b.readAllInputs();


  if (b.trig2.risingEdge()) {
    Serial.println(String("YES Incoming Reset on trig2 ")   );
    do_reset_sequence_counters = true;
  } else {
    //Serial.println(String("NO Incoming Reset on trig2 ")   );
    //do_reset_sequence_counters = false;
  }



  if (b.trig1.risingEdge()) {
    if (sequence_is_running == LOW) {
      StartSequencer();
    }
    OnTick();
    last_clock_pulse = millis();
  }

  //  b.readTriggers();
  //  if (b.triggerRose(2)){
  //    Serial.println(String("Incoming Reset on trig2 ")   );
  //    do_reset_sequence_counters = true;
  //  } else {
  //    //Serial.println(String("NO Incoming Reset on trig2 ")   );
  //  }




  // DRIVE CV
  /// This is connected to cv_waveform and reads the level. We use that to drive CV out.

  // This is Carrier * Modulator A output
  if (result_a_rms_object.available())
  {
    float result_a_rms_value = result_a_rms_object.read();
    //Serial.println(String("result_a_rms_value is: ") + result_a_rms_value  );

    // Also use this to drive the betweener ouput

    int cv_out_a_driver = fscale( 0.0, 1.0, 0, 4095, result_a_rms_value, 0);
    //Serial.println(String("cv_out_a_driver is: ") + cv_out_a_driver  );

    b.writeCVOut(3, cv_out_a_driver);
    //Led4Level(fscale( 0.0, 1.0, 0, 255, result_a_rms_value, 0));
  } else {
    //Serial.println(String("result_a_rms_object not available ")   );
  }



  // This is Carrier * Modulator B output
  if (result_b_rms_object.available())
  {
    float result_b_rms_value = result_b_rms_object.read();
    //Serial.println(String("result_b_rms_value is: ") + result_b_rms_value  );

    // Also use this to drive the betweener ouput

    int cv_out_b_driver = fscale( 0.0, 1.0, 0, 4095, result_b_rms_value, 0);
    //Serial.println(String("cv_out_b_driver is: ") + cv_out_b_driver  );


    // channel is 1-4 , value is 0 - 4095

    b.writeCVOut(4, cv_out_b_driver);
    //Led4Level(fscale( 0.0, 1.0, 0, 255, result_b_rms_value, 0));
  } else {
    //Serial.println(String("result_b_rms_object not available ")   );
  }







  //if (false){
  //  if (test_rms_object.available()){
  //          float test_rms = test_rms_object.read();
  //
  //          // Also use this to drive the betweener ouput
  //
  //            int test_rms_scaled = fscale( 0.0, 1.0, 0, 4095, test_rms, 0);
  //            Serial.println(String("test_rms_scaled is: ") + test_rms_scaled  );
  //            //b.writeCVOut(1, test_rms_scaled);
  //            //b.writeCVOut(2, test_rms_scaled);
  //            //b.writeCVOut(3, test_rms_scaled);
  //            b.writeCVOut(4, test_rms_scaled);
  //
  //
  //  } else {
  //    //Serial.println(String("test_rms_object is not available!!"));
  //  }
  //}





  ///////////////////

  // Analog Clock (and left input checking) //////

  /////////////////////////////////////////////////////////////////////////////////
  // When relying on the analogue clock, we don't have a stop as such, so if we don't detect a clock for a while, then assume its stopped.
  // Note that the Beat Step Pro takes a while to kill its clock out after pressing the Stop button.
  if (midi_clock_detected == LOW) {
    if ((millis() - last_clock_pulse > 500) && (sequence_is_running == HIGH)) {
      Serial.println("No analogue clock detected for 500 ms. Stopping sequencer.");

      // state-change-2

      StopSequencer();
    }
  }


} //////////////////////////////////////
///// END of LOOP //////////////////////





// Each time we start the sequencer we want to start from the same conditions.
void InitSequencer() {
  GateALow();
  GateBLow();
  CvStop();
  loop_timing.tick_count_since_start = 0;
  ResetSequenceCounters();
}

void StartSequencer() {
  Serial.println(String("Starting Sequencer... "));
  InitSequencer();
  sequence_is_running = HIGH;
}

void StopSequencer() {
  Serial.println(String("Stopping Sequencer... Hint: I'm now waiting for a midi / cv clock to do something.... "));
  InitSequencer();
  sequence_is_running = LOW;
}



void OnTick() {
  // Called on Every MIDI or Analogue clock pulse
  // Drives sequencer settings and activity.

  // Serial.println(String("loop_timing.tick_count_in_sequence is: ") + loop_timing.tick_count_in_sequence);


  // Read inputs and update settings.
  ReadInputsAndUpdateSettings();

  // Decide if we have a "step"
  // Could do some interesting things (swing?) if change the mod 6 below?
  if (loop_timing.tick_count_in_sequence % 6 == 0) {
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
void ReadInputsAndUpdateSettings() {
  // Note we set tick_count_in_sequence to 0 following at the stop and start midi messages.
  // The midi clock standard sends 24 ticks per crochet. (quarter note).



  // physical -> logical -> actual
  // pot -> value -> result

  //b.readTriggers();
  b.readAllInputs();

  // Note: we're using RAW inputs

  // Pot1
  pot1_input_value = b.readKnobRaw(1); // Sequence
  //Serial.println(String("pot1_input_value is: ") + pot1_input_value  );

  // Pot2
  pot2_input_value = b.readKnobRaw(2); // Length
  //Serial.println(String("pot2_input_value is: ") + pot2_input_value  );

  // Pot3
  pot3_input_value = b.readKnobRaw(3); // Frequency
  //Serial.println(String("pot3_input_value is: ") + pot3_input_value  );

  // Pot4
  pot4_input_value = b.readKnobRaw(4); // Env Release
  //Serial.println(String("pot4_input_value is: ") + pot4_input_value  );

  // Cv1
  cv1_input_value = b.readCV(1);
  //Serial.println(String("cv1_input_value is: ") + cv1_input_value  );

  // Cv2
  cv2_input_value = b.readCV(2);
  //Serial.println(String("cv2_input_value is: ") + cv2_input_value  );




  if (b.triggerHigh(3)) {
    mute_gate_a = true;
  } else {
    mute_gate_a = false;
  }

  //Serial.println(String("Mute GateA is: ") + mute_gate_a   );

  if (b.triggerHigh(4)) {
    mute_gate_b = true;
  } else {
    mute_gate_b = false;
  }

  //Serial.println(String("Mute GateB is: ") + mute_gate_b   );







  //////////////////////////////////////////
  // Assign values to change the sequencer.
  ///////////////////////////////////


  //////////////////////////////////////
  // CV stuff
  // ****** "Freq" ******
  //Serial.println(String("pot3_input_value is: ") + pot3_input_value  );

  cv_carrier_a_frequency = fscale( min_pot_value, max_pot_value, 0.01, 20, pot3_input_value, -1.5);

  cv_carrier_b_frequency = cv_carrier_a_frequency / 2;

  //Serial.println(String("cv_carrier_a_frequency is: ") + cv_carrier_a_frequency  );

  // ****** "Shape" ********

  cv_modulator_a_frequency_raw = pot4_input_value;
  //Serial.println(String("cv_modulator_a_frequency_raw is: ") + cv_modulator_a_frequency_raw  );


  // We want a value that goes from high to low as we turn the pot to the right.
  // So reverse the number range by subtracting from the maximum value.

  //Serial.println(String("CV_MODULATOR_A_FREQUENCY_RAW_MAX_INPUT is: ") + CV_MODULATOR_A_FREQUENCY_RAW_MAX_INPUT  );

  int cv_modulator_a_amplitude_delta_raw = CV_MODULATOR_A_FREQUENCY_RAW_MAX_INPUT - cv_modulator_a_frequency_raw;
  //Serial.println(String("cv_modulator_a_amplitude_delta_raw is: ") + cv_modulator_a_amplitude_delta_raw  );


  cv_modulator_a_amplitude_delta = fscale( 0, CV_MODULATOR_A_FREQUENCY_RAW_MAX_INPUT, 0.01, 0.4, cv_modulator_a_amplitude_delta_raw, -1.5);
  //Serial.println(String("cv_modulator_a_amplitude_delta is: ") + cv_modulator_a_amplitude_delta  );

  // B decays slower
  cv_modulator_b_amplitude_delta = cv_modulator_a_amplitude_delta / 2;
  //Serial.println(String("cv_modulator_b_amplitude_delta is: ") + cv_modulator_b_amplitude_delta  );


  // CV A
  carrier_a_object.frequency(cv_carrier_a_frequency);
  carrier_a_object.amplitude(1);

  // CV B
  carrier_b_object.frequency(cv_carrier_b_frequency);
  carrier_b_object.amplitude(1);


  // MONITOR GATE
  if (gate_monitor_rms.available())
  {
    float gate_peak = gate_monitor_rms.read();
    //Serial.println(String("gate_monitor_rms gate_peak ") + gate_peak  );
    Led1Level(fscale( 0.0, 1.0, 0, 255, gate_peak, 0));
  } else {
    //Serial.println(String("gate_monitor_rms not available ")   );
  }




  // return called_on_step;

} // End of ReadInputsAndUpdateSettings
////////////////////////////////////////////////
void InitMidiSequence(){

  Serial.println(String("InitMidiSequence Start ")  );

  // Loop through bars
  for (uint8_t bc = FIRST_BAR; bc <= MAX_BAR; bc++) {

    // Loop through steps
    for (uint8_t sc = FIRST_STEP; sc <= MAX_STEP; sc++) {
      //Serial.println(String("Step ") + sc );
    
      // Loop through notes
      for (uint8_t n = 0; n <= 127; n++) {
        // Initialise and print Note on (1) and Off (2) contents of the array.
        // WRITE MIDI MIDI_DATA
        channel_a_midi_note_events[bc][sc][n][1].is_active = 0;
        channel_a_midi_note_events[bc][sc][n][0].is_active = 0;
  
        
        //Serial.println(String("Init Step ") + sc + String(" Note ") + n +  String(" ON ticks value is ") + channel_a_midi_note_events[bc][sc][n][1].is_active);
        //Serial.println(String("Init Step ") + sc + String(" Note ") + n +  String(" OFF ticks value is ") + channel_a_midi_note_events[bc][sc][n][0].is_active);
      } 
    }
  }

  for (uint8_t n = 0; n <= 127; n++) {
     channel_a_ghost_events[n].is_active = 0;
     Serial.println(String("Init Step with ghost ") + String(" Note ") + n +  String(" is_active false ") );
  } 
  

  Serial.println(String("InitMidiSequence Done ")  );
}






/////////////////////////////////////////////////////////////
void OnStep() {

  Serial.println(String("OnStep step_count/MAX_STEP is: ") + step_count + "/" + MAX_STEP); // zero indexed

  if (step_count > MAX_STEP) {
    Serial.println(String("*****************************************************************************************"));
    Serial.println(String("************* ERROR! step_count is: ") + step_count + String("*** ERROR *** "));
    Serial.println(String("*****************************************************************************************"));
  }

  //
   // if (step_count != FIRST_STEP){
      if (do_reset_sequence_counters == true){
        ResetSequenceCounters();
        do_reset_sequence_counters = false;
        Serial.println(String("do_reset_sequence_counters was true. I called ResetSequenceCounters"));
      }
   // }

  uint8_t play_note = bitRead(hybrid_sequence_1, step_count_sanity(step_count));

  if (step_count == FIRST_STEP) {
    OnFirstStep();
  }

  // Go low
  GateALow();
  GateBLow();

  // Play Note (GateA)
  if (play_note) {
    //Serial.println(String("****************** play ")   );

    if (not mute_gate_a) {
      GateAHigh();
      //ResetCVA();
    } else {
      //Serial.println(String("mute_gate_a is Muted"));
    }

    // OnFirstStep();

    // We might want to only start the CV pulse on the first step
    //        if (reset_cv_lfo_at_FIRST_STEP == true){
    //          if (step_count == FIRST_STEP) {
    //            OnFirstStep();
    //          }
    //        } else {
    //          OnFirstStep();
    //        }


  } else {
    // Opposite (GateB)
  
    //Serial.println(String("not play ")   );


    if (not mute_gate_b) {
      GateBHigh();
      ResetCVB();
    } else {
      //Serial.println(String("mute_gate_b is Muted"));
    }



    
  }




}

void OnNotStep() {
  //Serial.println(String("NOT step_countIn is: ") + step_countIn  );

  // We must set the gates low between steps so they are read to go high OnStep
  GateALow();
  GateBLow();
  ChangeCvModulatorAAmplitude();
  ChangeCvModulatorBAmplitude();
}

void GateAHigh() {
  //Serial.println(String("Gate HIGH at tick_count_since_start: ") + loop_timing.tick_count_since_start);

  // This is to drive the LED (other things too?)
  gate_dc_waveform.amplitude(0.99, 10);

  // The output
  b.writeCVOut(1, 4095); //cvout selects channel 1 through 4; value is in range 0-4095


  ////////////////



}

void ResetCVB() {

  carrier_b_object.phase(90); // Sine wave has maximum at 90 degrees

  cv_modulator_b_amplitude = 0.99;

  //Serial.println(String("ResetCVB to ") + cv_modulator_b_amplitude);
  modulator_b_object.amplitude(cv_modulator_b_amplitude, 10);
}


void GateALow() {
  //Serial.println(String("Gate LOW") );

  // For LED
  gate_dc_waveform.amplitude(0);

  // For output
  b.writeCVOut(1, 0);


}


void GateBHigh() {
  //Serial.println(String("Gate HIGH at tick_count_since_start: ") + loop_timing.tick_count_since_start);

  // For what?
  gate_2_dc_waveform.amplitude(0.99, 10);

  // For output
  b.writeCVOut(2, 4095); //cvout selects channel 1 through 4; value is in range 0-4095

}

void GateBLow() {
  //Serial.println(String("Gate LOW") );

  // For what?
  gate_2_dc_waveform.amplitude(0);

  // For output
  b.writeCVOut(2, 0);


}




void ResetCVA() {
  carrier_a_object.phase(90); // Sine wave has maximum at 90 degrees

  // Used to modulate CV.
  // Allow the amplitude to fall to zero before we lift it back up. (if it indeed gets to zero)
  if (cv_modulator_a_amplitude == 0) {
    cv_modulator_a_amplitude = 0.99;
    modulator_a_object.amplitude(cv_modulator_a_amplitude, 10);
  }

}



void OnFirstStep() {
  //Serial.println(String("CV Pulse On") );

  ResetCVA();
  //ResetCVB();

}

void ChangeCvModulatorAAmplitude() {
  cv_modulator_a_amplitude -= cv_modulator_a_amplitude_delta;
  if (cv_modulator_a_amplitude <= 0) {
    cv_modulator_a_amplitude = 0;
  }

  modulator_a_object.amplitude(cv_modulator_a_amplitude, 10); // setting-b-amplitude
  //Serial.println(String("cv_modulator_a_amplitude is: ") + cv_modulator_a_amplitude);

}

void ChangeCvModulatorBAmplitude() {
  cv_modulator_b_amplitude -= cv_modulator_b_amplitude_delta;
  if (cv_modulator_b_amplitude <= 0) {
    cv_modulator_b_amplitude = 0;
  }

  //Serial.println(String("cv_modulator_b_amplitude is: ") + cv_modulator_b_amplitude);
  modulator_b_object.amplitude(cv_modulator_b_amplitude, 10); // setting-b-amplitude

}





void CvStop() {
  Serial.println(String("CvStop") );
  cv_modulator_a_amplitude = 0;
  modulator_a_object.amplitude(cv_modulator_a_amplitude, 10);

  cv_modulator_b_amplitude = 0;
  modulator_b_object.amplitude(cv_modulator_b_amplitude, 10);

}





void clockShowHigh() {
  //Serial.println(String("Clock Show HIGH ") );
  digitalWrite(teensy_led_pin, HIGH);   // set the LED on
}

void clockShowLow() {
  //Serial.println(String("Clock Show LOW") );
  digitalWrite(teensy_led_pin, LOW);   // set the LED off
}





void Led1Level(uint8_t level) {

  analogWrite(betweener_led_pin, level);

  if (level > 150) {
    digitalWrite(betweener_led_pin, HIGH);
  } else {
    digitalWrite(betweener_led_pin, LOW);
  }




}





//////////
bool last_button_1_state = HIGH;

bool Button1HasChanged(bool button_1_state) {

  bool button_1_has_changed;
  if (button_1_state != last_button_1_state) {
    button_1_has_changed = true;
    last_button_1_state = button_1_state;
    //Serial.println(String("button_1_has_changed is: ") + button_1_has_changed + String(" to: ") + button_1_state );
  } else {
    button_1_has_changed = false;
  }
  return button_1_has_changed;
}


void SetSequencePattern() {
  // Button is in Normal state (not pressed) (HIGH) (button_1_state == HIGH)
  // UPPER Pot HIGH Button //////////
  // 8 bit sequence - 8 Least Significant Bits
  last_binary_sequence_1 = binary_sequence_1;



  // If we have 8 bits, use the range up to 255


  uint8_t binary_sequence_lower_limit = 1;  // Setting to 1 means we never get 0 i.e. a blank sequence especially when we change seq length

  unsigned int binary_sequence_upper_limit;




  // REMEMBER, sequence_length_in_steps is ONE indexed (from MIN (e.g. 4) up to MAX (e.g. 16))
  // For a 3 step sequence we want to cover all the possibilities of a 3 step sequence which is (2^3) - 1 = 7
  // i.e. all bits on of a 3 step sequence is 111 = 7 decimal
  // or (2^sequence_length_in_steps) - 1



  //Serial.println(String("sequence_length_in_steps is: ") + sequence_length_in_steps  );


  binary_sequence_upper_limit = pow(2.0, sequence_length_in_steps) - 1;

  //Serial.println(String("binary_sequence_upper_limit is: ") + binary_sequence_upper_limit  );

  binary_sequence_1 = fscale( min_pot_value, max_pot_value, binary_sequence_lower_limit, binary_sequence_upper_limit, pot1_input_value, 0);


  if (binary_sequence_1 != last_binary_sequence_1) {
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


  hybrid_sequence_1 = grey_sequence_1;

  //bitClear(hybrid_sequence_1, sequence_length_in_steps - 1); // sequence_length_in_steps is 1 based index. bitClear is zero based index.

  //hybrid_sequence_1 = ~ hybrid_sequence_1; // Invert


  // So pot fully counter clockwise is 1 on the first beat
  //  if (binary_sequence_1 == 1) {
  //    hybrid_sequence_1 = 1;
  //  }


  //Serial.println(String("hybrid_sequence_1 is: ") + hybrid_sequence_1  );
  //Serial.print("\t");
  //Serial.print(hybrid_sequence_1, BIN);
  //Serial.println();


}


void SetSequenceLength() {

  //  Serial.println(String("min_pot_value is: ") + min_pot_value  );
  // Serial.println(String("max_pot_value is: ") + max_pot_value  );
  //  Serial.println(String("pot2_input_value is: ") + pot2_input_value  );



  // NOTE Sometimes we might not get 0 out of a pot - or 1.0 so use the middle range
  sequence_length_in_steps = fscale( min_pot_value, max_pot_value, MIN_SEQUENCE_LENGTH_IN_STEPS, MAX_SEQUENCE_LENGTH_IN_STEPS, pot2_input_value, 0);

  //Serial.println(String("sequence_length_in_steps_raw is: ") + sequence_length_in_steps_raw  );
  // Reverse because we want fully clockwise to be short so we get 1's if sequence is 1.
  //sequence_length_in_steps = 16 - sequence_length_in_steps_raw;

  // sequence_length_in_steps = sequence_length_in_steps_raw;

  if (sequence_length_in_steps < MIN_SEQUENCE_LENGTH_IN_STEPS) {
    sequence_length_in_steps = MIN_SEQUENCE_LENGTH_IN_STEPS;
    Serial.println(String("**** ERROR with sequence_length_in_steps but it is NOW: ") + sequence_length_in_steps  );
  }

  if (sequence_length_in_steps > MAX_SEQUENCE_LENGTH_IN_STEPS) {
    sequence_length_in_steps = MAX_SEQUENCE_LENGTH_IN_STEPS;
    Serial.println(String("**** ERROR with sequence_length_in_steps but it is NOW: ") + sequence_length_in_steps  );
  }

  //Serial.println(String("sequence_length_in_steps is: ") + sequence_length_in_steps  );

  new_sequence_length_in_ticks = (sequence_length_in_steps) * 6;

  //Serial.println(String("new_sequence_length_in_ticks is: ") + new_sequence_length_in_ticks  );
}


void AdvanceSequenceChronology() {




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

void lightStatus(int inputNumber) {
}


int GetValue(int raw, int last, int jitter_reduction) {
  int value;

  int diff = abs(raw - last);

  // because the value seems to woble, only take the even value for less jitter.
  if (diff >= jitter_reduction) {
    value = raw;
    //Serial.println(String("GetValue says use RAW value because diff is ") + diff );
  } else {
    value = last;
    //Serial.println(String("GetValue says use LAST value because diff is ") + diff );
  }
  return value;
}

bool IsCrossing(int value_1, int value_2, int fuzzyness) {
  // because the value seems to woble, only take the even value for less jitter.
  if (abs(value_1 - value_2) <= fuzzyness) {
    return true;
  } else {
    return false;
  }
}



void Flash(int delayTime, int noOfTimes, int ledPin) {

  int i;

  // blink a few times with a shorter delay each time.
  for (i = 0; i < noOfTimes; i++) {
    digitalWrite(ledPin, LOW);
    delay(delayTime);
    digitalWrite(ledPin, HIGH);   // set the LED on
    delay(delayTime);// wait
    digitalWrite(ledPin, LOW);    // set the LED off
    if (delayTime > 15) {
      delayTime = delayTime - 10;
    }
  }

}



//mydatatype_t stack[MAXSTACKSIZE];
//int stackptr = 0;
//#define push(d) stack[stackptr++] = d
//#define pop stack[--stackptr]
//#define topofstack stack[stackptr - 1]



// copy paste betweener-teensy
void DisableNotes(uint8_t note){
             // Disable that note for all steps.
           uint8_t sc = 0;
           uint8_t bc = 0;
           for (bc = FIRST_BAR; bc <= MAX_BAR; bc++){
            for (sc = FIRST_STEP; sc <= MAX_STEP; sc++){
              // WRITE MIDI MIDI_DATA
              channel_a_midi_note_events[bc][sc][note][1].velocity = 0;
              channel_a_midi_note_events[bc][sc][note][1].is_active = 0;
              channel_a_midi_note_events[bc][sc][note][0].velocity = 0;
              channel_a_midi_note_events[bc][sc][note][0].is_active = 0;         
            }
           }
}

//








uint8_t step_count_sanity(uint8_t step_count_) {
  uint8_t step_count_fixed;

  if (step_count_ > MAX_STEP) {
    Serial.println(String("**** ERROR step_count_ > MAX_STEP i.e. ") + step_count_ );
    step_count_fixed = MAX_STEP;
  } else if (step_count_ < FIRST_STEP) {
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
              newEnd, float inputValue, float curve) {

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

  if (newEnd > newBegin) {
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

  if (invFlag == 0) {
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
  unsigned int n_data = (data >> 1);
  n_data = (data ^ n_data);

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
