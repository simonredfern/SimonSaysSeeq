  /*

 ____  _  _      ____  _      ____  ____ ___  _ ____  ____  _____ _____ ____ 
/ ___\/ \/ \__/|/  _ \/ \  /|/ ___\/  _ \\  \/// ___\/ ___\/  __//  __//  _ \
|    \| || |\/||| / \|| |\ |||    \| / \| \  / |    \|    \|  \  |  \  | / \|
\___ || || |  ||| \_/|| | \||\___ || |-|| / /  \___ |\___ ||  /_ |  /_ | \_\|
\____/\_/\_/  \|\____/\_/  \|\____/\_/ \|/_/   \____/\____/\____\\____\\____\

SIMON SAYS SEEQ is released under the AGPL and (c) Simon Redfern 2020, 2021

Version: 2021-01-28 or so.

This sequencer is dedicated to all those folks working to fight climate change! Whilst you're here, check out https://feedbackloopsclimate.com/introduction/ 

This file uses BELA libraries and example code, see below.

An intro to what this does: https://www.twitch.tv/videos/885185134

*/


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

// TO Understand render see the example in Fundamentals: minimal/render.cpp

// In general, see https://www.youtube.com/watch?v=XJ2fFqGexCM


// So Bela can get to the internet via a Mac with ethernet over USB

// ssh root@bela.local
// vi /etc/network/interfaces
// auto usb0
// iface usb0 inet dhcp
// auto usb1
// iface usb1 inet dhcp
// And enable Mac OS Sharing like this:
// Mac OS Preferences. Sharing From = Wifi. (drop down list) To Computers using = Bela (check box). Internet Sharing = Yes. (ticked)  
// then on Bela: systemctl restart networking.service
// Login again and
// ping 8.8.8.8


//
// install curl apt-get install -y libcurl-dev



#include <Bela.h>
#include <libraries/Midi/Midi.h>
#include <stdlib.h>
#include <cmath>

#include <libraries/Scope/Scope.h>

#include <libraries/ADSR/ADSR.h>





#include <chrono>

#include <libraries/UdpClient/UdpClient.h>

UdpClient myUdpClient;

// ...


const char version[16]= "v0.25-BelaSalt";

Scope scope;


//////////////////////////////
// Bela wrapping of ALSA Linux Midi library see midi.h in this IDE or http://docs.bela.io/classMidi.html
// Use a class compliant USB Midi device
Midi midi;

// To find midi ports Bela can see, type "amidi -l" in the Bela command line.
// Also  lsusb -t via ssh 

/*
root@bela:~/bin# cat log_usb.sh
#!/bin/bash
echo Hello
amidi -l
lsusb -t
*/

/*
root@bela:~/bin/SimonSaysSeeq# amidi -l
Dir Device    Name
IO  hw:0,0    f_midi <-- This is the connection to your computer (device port on Bela)
IO  hw:1,0,0  USB MIDI Interface MIDI 1 <-- This is the device connected to USB host port on the Bela
*/


//const char* gMidiPort0 = "hw:0,0"; // This is the computer via USB cable
const char* gMidiPort0 = "hw:1,0,0"; // This is the first external USB Midi device. Keyboard is connected to the USB host port on Bela / Salt+
//const char* gMidiPort0 = "hw:0,0,0"; // try me

// Let our delay have 50 different course settings (so the pot doesn't jitter)
const unsigned int MAX_COARSE_DELAY_TIME_INPUT = 50;

// Divde clock - maybe just for midi maybe for sequence too
const uint8_t MAX_CLOCK_DIVIDE_INPUT = 128;

const uint8_t MAX_PROGRESSION_INPUT = 4;

uint64_t frame_timer = 0;

uint64_t last_clock_falling_edge = 0; 

uint64_t last_quarter_note_frame = 0;
uint64_t previous_quarter_note_frame = 0;

uint64_t last_sequence_reset_frame = 0;
uint64_t previous_sequence_reset_frame = 0;

uint64_t frames_per_sequence = 0;


uint64_t last_tick_frame = 0;
 
//int clock_width = 0;

uint64_t elapsed_frames_since_last_tick = 0;

float detected_bpm = 120.0;

float env3_amp = 0.7f;



uint64_t INITIAL_FRAMES_PER_24_TICKS = 22064;
uint64_t frames_per_24_ticks = 22064; // This must never be zero else we can divide by zero errors

int audio_sample_rate;
int analog_sample_rate;

// Amount of delay in samples (needs to be smaller than or equal to the buffer size defined above)
int coarse_delay_frames = 22050;
int total_delay_frames = 22050;

// Amount of feedback
float delay_feedback_amount = 0.999; //0.999


uint8_t clock_divide_input = 1; // normal
uint8_t progression_input = 1; // normal

#include <math.h> //sinf
#include <time.h> //time
#include <libraries/Oscillator/Oscillator.h>
#include <libraries/OscillatorBank/OscillatorBank.h>


#define DELAY_BUFFER_SIZE 6400000


const float kMinimumFrequency = 20.0f;
const float kMaximumFrequency = 8000.0f;
int gSampleCount;               // Sample counter for indicating when to update frequencies
float gNewMinFrequency;
float gNewMaxFrequency;
// Task for handling the update of the frequencies using the analog inputs
AuxiliaryTask gFrequencyUpdateTask;

AuxiliaryTask gChangeSequenceTask;

AuxiliaryTask gPrintStatus;


// These settings are carried over from main.cpp
// Setting global variables is an alternative approach
// to passing a structure to userData in setup()
int gNumOscillators = 2; // was 500
int gWavetableLength = 1024;
void recalculate_frequencies(void*);
OscillatorBank osc;

Oscillator oscillator_2_audio;
Oscillator oscillator_1_analog;


int gAudioChannelNum; // number of audio channels to iterate over
int gAnalogChannelNum; // number of analog channels to iterate over

//SWITCH1 in digital channel 6	(LEFT BUTTON)
//T1 in digital channel 15
//T2/SWITCH2 in digital channel 14 (RIGHT BUTTON)
//T3/SWITCH3 in digital channel 1 (maybe this is for Salt+)
//T4/SWITCH4 in digital channel 3 (maybe this is for Salt+?)


int button_1_PIN = 6; 
int new_button_1_state = 0; 
int old_button_1_state = 0; 


int button_2_PIN = 14;
int new_button_2_state = 0; 
int old_button_2_state = 0; 

// Salt + ?

int button_3_PIN = 1; 
int new_button_3_state = 0; 
int old_button_3_state = 0; 


int button_4_PIN = 3;
int new_button_4_state = 0; 
int old_button_4_state = 0; 



/////////////////


int old_both_buttons_pressed_state = 0;
int new_both_buttons_pressed_state = 0;
int both_buttons_pressed_counter = 0;
int both_buttons_pressed_even = 0;
int do_both_buttons_action_a = 0;
int do_both_buttons_action_b = 0;

int do_button_1_action = 0;
int do_button_2_action = 0;
int do_button_3_action = 0;
int do_button_4_action = 0;

//////////////////

int fine_delay_frames_delta = 1;
int fine_delay_frames = 0;

//float feedback_delta = 1.0f;

// LED Control: https://github.com/BelaPlatform/Bela/wiki/Salt#led-and-pwm
int LED_1_PIN = 2;
int	LED_2_PIN = 4;
int LED_3_PIN = 8;
int	LED_4_PIN = 9;
int	LED_PWM_PIN = 7;

// - set the LED pin as an INPUT: LED off
// - set the LED pin as an OUTPUT, value 0: LED ON, red
// - set the LED pin as an OUTPUT, value 1: LED ON, blue


// recursive function to print binary representation of a positive integer
void print_binary(unsigned int number)
{
    if (number >> 1) {
        print_binary(number >> 1);
    }
    rt_printf("%c" , (number & 1) ? '1' : '0' );
}






// Set the analog channels to read from
//const int CLOCK_INPUT_ANALOG_IN_PIN = 0;

// Salt Pinouts salt pinouts are here: https://github.com/BelaPlatform/Bela/wiki/Salt

// T2 (Trigger 2) is Physical Channel / Pin 14 

// T1 in is	digital channel 15
const int CLOCK_INPUT_DIGITAL_PIN = 15;

////////////////////////////////
// Digital Outputs
// T1 out	digital channel 0
// T2 out	digital channel 5
const int SEQUENCE_OUT_PIN = 0;



//const int CLOCK_OUTPUT_DIGITAL_PIN = 0;

// CV I/O 1-8	ANALOG channel 0-7
const int SEQUENCE_PATTERN_ANALOG_INPUT_PIN = 0; // CV 1 input
const int SEQUENCE_LENGTH_ANALOG_INPUT_PIN = 2; // CV 3 input

const int OSC_FREQUENCY_INPUT_PIN = 1; // CV 2 input
const int ADSR_RELEASE_INPUT_PIN = 3; // CV 4 input


const int COARSE_DELAY_TIME_INPUT_PIN = 4; // CV 5 (SALT+)
const int DELAY_FEEDBACK_INPUT_PIN = 6; // CV 6 (SALT+)


const uint8_t CLOCK_DIVIDE_INPUT_PIN = 5; // CV 7 (SALT+) (TODO check)
const uint8_t PROGRESSION_INPUT_PIN = 7; // CV 8 (SALT+) (TODO check)





const int SEQUENCE_GATE_OUTPUT_1_PIN = 0; // CV 1 output
const int SEQUENCE_CV_OUTPUT_2_PIN = 1; // CV 2 input
const int SEQUENCE_CV_OUTPUT_3_PIN = 2; // CV 3 output
const int SEQUENCE_CV_OUTPUT_4_PIN = 3; // CV 4 input


////////////////////////////////////////////////


// const uint8_t BRIGHT_0 = 0;
// const uint8_t BRIGHT_1 = 10;
// const uint8_t BRIGHT_2 = 20;
// const uint8_t BRIGHT_3 = 75;
// const uint8_t BRIGHT_4 = 100;
// const uint8_t BRIGHT_5 = 255;

// Use zero based index for sequencer. i.e. step_count for the first step is 0.
const uint8_t FIRST_STEP = 0;
const uint8_t MAX_STEP = 15;

const uint8_t FIRST_BAR = 0;
const uint8_t MAX_BAR = 7; // Memory User!

const uint8_t MIN_SEQUENCE_LENGTH_IN_STEPS = 4; // ONE INDEXED
const uint8_t MAX_SEQUENCE_LENGTH_IN_STEPS = 16; // ONE INDEXED

// Sequence Length (and default)
uint8_t current_sequence_length_in_steps = 8; 

///////////////////////

//const int CV_WAVEFORM_B_FREQUENCY_RAW_MAX_INPUT = 1023;


const uint8_t MIDI_NOTE_ON = 1;
const uint8_t MIDI_NOTE_OFF = 0;


uint8_t midi_channel_a = 0; // This is zero indexed. 0 will send on midi channel 1!
uint8_t last_note_on = 0;
uint8_t last_note_off = 0;
uint8_t last_note_disabled = 0;


////////////////////////////////////////////////////
// Actual pot values
unsigned int upper_input_raw; // TODO Make t type.
unsigned int lower_input_raw;


// Create 4 virtual pots out of two pots and a button.
// To handle the case when 1) Pot is fully counter clockwise 2) We press the button 3) Move the pot fully clockwise 4) Release the button.
// We introduce the concept that a virtual pot can be "engaged" or not so we can catchup with its stored value only when the pot gets back to that position.
//bool upper_pot_high_engaged = true;
float sequence_pattern_input_raw;
unsigned int sequence_pattern_input = 20;
unsigned int sequence_pattern_input_last;
unsigned int sequence_pattern_input_at_button_change;

//bool lower_pot_high_engaged = true;
float lfo_a_frequency_input_raw;
float lfo_osc_1_frequency;
float frequency_2;
float audio_osc_2_frequency;


unsigned int lfo_a_frequency_input = 20;
unsigned int lfo_a_frequency_input_last;
unsigned int lfo_a_frequency_input_at_button_change;

//bool upper_pot_low_engaged = true;
float sequence_length_input_raw;
//unsigned int sequence_length_input = 20;
unsigned int sequence_length_input_last;
unsigned int sequence_length_input_at_button_change;

//bool lower_pot_low_engaged = true;
float lfo_b_frequency_input_raw;
unsigned int lfo_b_frequency_input = 20;
unsigned int lfo_b_frequency_input_last;
unsigned int lfo_b_frequency_input_at_button_change;


int new_digital_clock_in_state;
int current_digital_clock_in_state;
float analog_clock_in_level;
float right_peak_level;

float external_modulator_object_level;


float audio_left_input_raw;
float audio_right_input_raw;




unsigned int coarse_delay_input = 1;



////////////////////////////////////////////////////


////////////////////////////////////////////////////
// Musical parameters that the user can tweak.

//uint8_t current_sequence_length_in_steps_raw;


// The Primary GATE sequence pattern // Needs to be upto 16 bits. Maybe more later.
unsigned int binary_sequence_result;
unsigned int gray_code_sequence;
unsigned int the_sequence;
unsigned int last_binary_sequence_result; // So we can detect changes




bool do_tick = true;

bool do_envelope_1_on = false;
bool target_gate_out_state = false;
bool gate_out_state_set = false;

// Used to control when/how we change sequence length 
uint8_t new_sequence_length_in_ticks; 

// Just counts 0 to 5 within each step
uint8_t ticks_after_step;

// Jitter Reduction: Used to flatten out glitches from the analog pots. 
// Actually we like the glitches - it makes the sequencer more interesting


//double	wait_time_ms;


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
   uint8_t tick_count_since_step = 0; 
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
//NoteInfo channel_a_midi_note_events[MAX_STEP+1][128][2]; 

NoteInfo channel_a_midi_note_events[MAX_BAR+1][MAX_STEP+1][128][2]; 
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
    uint8_t tick_count_in_sequence = 0; // Since we started the sequencer  
    uint32_t tick_count_since_start = 0; // since the clock started running this time.
    uint8_t tick_count_since_step = 0; // between 0 and 5 as there are 6 ticks in a step
};

// Timing is controlled by the loop. Only the loop should update it.
Timing loop_timing;

// Count of the main pulse i.e. sixteenth notes or eigth notes 
uint8_t step_count; // write
uint8_t step_play; // read

// Count of the bar / measure.
uint8_t bar_count; // for wrting
uint8_t bar_play; // for reading

// Helper functions that operate on global variables. Yae!  

uint8_t BarCountSanity(uint8_t bar_count_in){
  uint8_t bar_count_fixed;
  
  if (bar_count_in > MAX_BAR){
  	rt_printf("**** ERROR bar_count_in > MAX_BAR i.e. %d \n" , bar_count_in );
    bar_count_fixed = MAX_BAR;
  } else if (bar_count_in < FIRST_BAR){
    rt_printf("**** ERROR bar_count_in < FIRST_BAR i.e. %d \n" , bar_count_in );
    bar_count_fixed = FIRST_BAR;
  } else {
    bar_count_fixed = bar_count_in;
  }
  return bar_count_fixed;
}


void SetTickCountInSequence(uint8_t value){
  loop_timing.tick_count_in_sequence = value;
  loop_timing.tick_count_since_step = value % 6;
}

void SetTotalTickCount(int value){
  loop_timing.tick_count_since_start = value;
}

void Beginning(){
  SetTickCountInSequence(0);
  step_count = FIRST_STEP;
  bar_count = FIRST_BAR;
  step_play = FIRST_STEP;
  bar_play = FIRST_BAR;
}


uint8_t IncrementOrResetBarCount(){

  // Every time we call this function we advance or reset the bar
  if (bar_count == MAX_BAR){
    bar_count = FIRST_BAR;
  } else {
    bar_count = BarCountSanity(bar_count + 1);
  }
  
  //rt_printf("** IncrementOrResetBarCount bar_count is now: %d \n", bar_count);
  return BarCountSanity(bar_count);
}

/*
// This is not referenced!
void ResetToFirstStep(){
  
  // TODO check if we really need this or possible bug with bars
  SetTickCountInSequence(0);
  step_count = FIRST_STEP;

  IncrementOrResetBarCount();
  
  //Serial.println(String("ResetToFirstStep Done. sequence_length_in_steps is ") + sequence_length_in_steps + String(" step_count is now: ") + step_count);
}
*/









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







float gInterval = 0.5;
float gSecondsElapsed = 0;
int gCount = 0;

int temp_count = 0;



// for ADSR

ADSR envelope_1_audio; 
ADSR step_triggered_envelope_2;
ADSR sequence_triggered_envelope_3;

float audio_envelope_1_amplitude = 0;
float analog_envelope_2_amplitude = 0;
float analog_sequence_triggered_envelope_3_amplitude = 0;


float envelope_1_attack = 0.0001; // envelope_1_audio attack (seconds)
float envelope_1_decay = 0.1; // envelope_1 decay (seconds)
float envelope_1_sustain = 0.9; // envelope_1 sustain level
float envelope_1_release = 0.5; // envelope_1 release (seconds)

float analog_out_1;
float analog_out_2;
float analog_out_3;
float analog_out_4;


float audio_osc_1_result;
float osc_2_result_analog;
float analog_osc_3_result;

// float envelope_2_attack = 0.0001; // envelope_2 attack (seconds)
// float envelope_2_decay = 0.25; // envelope_2 decay (seconds)
// float envelope_2_sustain = 0.9; // envelope_2 sustain level
// float envelope_2_release = 0.5; // envelope_2 release (seconds)

// float sequence_triggered_envelope_3_attack = 0.0001; // envelope_2 attack (seconds)
// float sequence_triggered_envelope_3_decay = 0.25; // envelope_2 decay (seconds)
// float sequence_triggered_envelope_3_sustain = 0.9; // envelope_2 sustain level
// float sequence_triggered_envelope_3_release = 0.5; // envelope_2 release (seconds)



float gFrequency = 320.0; // Oscillator frequency (Hz)
//float gPhase; // Oscillator phase
//float gInverseSampleRate;

// Oscillator type
enum osc_type
{
	sine,		// 0
	triangle,	// 1
	square,		// 2
	sawtooth,	// 3
	numOscTypes
};




void ResetSequenceCounters(){
  SetTickCountInSequence(0);
  IncrementOrResetBarCount();
  step_count = FIRST_STEP; 
  oscillator_1_analog.setPhase(0.0);
  
  

  sequence_triggered_envelope_3.gate(true);
  //sequence_triggered_envelope_3.gate(false);

  previous_sequence_reset_frame = last_sequence_reset_frame; // The last time the sequence was reset
  last_sequence_reset_frame = frame_timer; // Now
  
  
  // We'll be able to use this, to set delay in frames
  frames_per_sequence = last_sequence_reset_frame - previous_sequence_reset_frame;

  //rt_printf("ResetSequenceCounters Done. current_sequence_length_in_steps is: %d step_count is now: %d \n", current_sequence_length_in_steps, step_count);
}




/// end for ADSR





void printStatus(void*){

    // Might not want to print every time else we overload the CPU
    gCount++;
	
    if(gCount % 1000 == 0) {
      
		rt_printf("======== Hello from printStatus. gCount is: %d ========= \n",gCount);

		  // Global frame timing

		rt_printf("frame_timer is: %llu \n", frame_timer);
    	rt_printf("frames_per_24_ticks is: %llu \n", frames_per_24_ticks);
    	rt_printf("detected_bpm is: %f \n", detected_bpm);
    	rt_printf("frames_per_sequence is: %llu \n", frames_per_sequence);
    
		// Delay Time
		
		rt_printf("DELAY_BUFFER_SIZE is: %d \n", DELAY_BUFFER_SIZE);



		rt_printf("coarse_delay_input is: %d \n", coarse_delay_input);		
		rt_printf("coarse_delay_frames is: %d \n", coarse_delay_frames);
		rt_printf("fine_delay_frames_delta is: %d \n", fine_delay_frames_delta);
		
		
		rt_printf("fine_delay_frames is: %d \n", fine_delay_frames);
		
		
		
		rt_printf("total_delay_frames is: %d \n", total_delay_frames);		
		
		
		// Delay Feedback
		//rt_printf("feedback_delta is: %f \n", feedback_delta);

	    
		rt_printf("delay_feedback_amount is: %f \n", delay_feedback_amount);
		
		
		// Analog / Digital Clock In.
  		rt_printf("last_quarter_note_frame is: %llu \n", last_quarter_note_frame);

		// Other Inputs
    	//rt_printf("sequence_pattern_input_raw is: %f \n", sequence_pattern_input_raw);
		rt_printf("sequence_pattern_input is: %d \n", sequence_pattern_input);
		rt_printf("sequence_length_input_raw is: %f \n", sequence_length_input_raw);
		
		/*
		rt_printf("new_button_1_state is: %d \n", new_button_1_state);
		rt_printf("new_button_2_state is: %d \n", new_button_2_state);
		rt_printf("new_button_3_state is: %d \n", new_button_3_state);
		rt_printf("new_button_4_state is: %d \n", new_button_4_state);
		

		rt_printf("do_button_1_action is: %d \n", do_button_1_action);
		rt_printf("do_button_2_action is: %d \n", do_button_2_action);
		rt_printf("do_button_3_action is: %d \n", do_button_3_action);
		rt_printf("do_button_4_action is: %d \n", do_button_4_action);
		*/


		rt_printf("clock_divide_input is: %d \n", clock_divide_input);
		rt_printf("progression_input is: %d \n", progression_input);

		





	
		/*
		rt_printf("old_both_buttons_pressed_state is: %d \n", old_both_buttons_pressed_state);
		rt_printf("new_both_buttons_pressed_state is: %d \n", new_both_buttons_pressed_state);
		rt_printf("both_buttons_pressed_counter is: %d \n", both_buttons_pressed_counter);
		rt_printf("both_buttons_pressed_even is: %d \n", both_buttons_pressed_even);
		rt_printf("do_both_buttons_action_a is: %d \n", do_both_buttons_action_a);
		rt_printf("do_both_buttons_action_b is: %d \n", do_both_buttons_action_b);
		*/
		
		
        // Sequence derived results 
        /*
    	rt_printf("current_sequence_length_in_steps is: %d \n", current_sequence_length_in_steps);
    	rt_printf("new_sequence_length_in_ticks is: %d \n", new_sequence_length_in_ticks);
    	*/


		/*
    	rt_printf("envelope_1_attack is: %f \n", envelope_1_attack);
    	rt_printf("envelope_1_decay is: %f \n", envelope_1_decay);
    	*/
    	
    	//rt_printf("envelope_1_release is: %f \n", envelope_1_release);
    	
    	/*
    	rt_printf("audio_envelope_1_amplitude is: %f \n", audio_envelope_1_amplitude);
    	rt_printf("analog_envelope_2_amplitude is: %f \n", analog_envelope_2_amplitude);
    	rt_printf("analog_sequence_triggered_envelope_3_amplitude is: %f \n", analog_sequence_triggered_envelope_3_amplitude);
    	*/


		/*
		rt_printf("audio_osc_1_result is: %f \n", audio_osc_1_result);
		rt_printf("osc_2_result_analog is: %f \n", osc_2_result_analog);
		*/
		
		/*
		rt_printf("analog_out_1 is: %f \n", analog_out_1);
		rt_printf("analog_out_2 is: %f \n", analog_out_2);
		rt_printf("analog_out_3 is: %f \n", analog_out_3);
		rt_printf("analog_out_4 is: %f \n", analog_out_4);
		*/
		
		/*
		rt_printf("audio_left_input_raw is: %f \n", audio_left_input_raw);	
		rt_printf("audio_right_input_raw is: %f \n", audio_right_input_raw);
		*/

		// Clock derived values
		/*
    	rt_printf("analog_clock_in_state is: %d \n", analog_clock_in_state);
    	rt_printf("current_digital_clock_in_state is: %d \n", current_digital_clock_in_state);
    	rt_printf("new_digital_clock_in_state is: %d \n", new_digital_clock_in_state);
    	rt_printf("midi_clock_detected is: %d \n", midi_clock_detected);
    	*/

    	//rt_printf("loop_timing.tick_count_in_sequence is: %d \n", loop_timing.tick_count_in_sequence);
    	//rt_printf("loop_timing.tick_count_since_start is: %d \n", loop_timing.tick_count_since_start);

    	

    	
		/*
		rt_printf("gray_code_sequence is: %d \n", gray_code_sequence);
		print_binary(gray_code_sequence);
		rt_printf("%c \n", 'B');
		*/

    	rt_printf("the_sequence is: %d \n", the_sequence);
    	print_binary(the_sequence);
		rt_printf("%c \n", 'B');

		// Sequence state
		
    	rt_printf("bar_count: %d \n", bar_count);
		rt_printf("step_count: %d \n", step_count);
		
    	rt_printf("bar_play: %d \n", bar_play);
		rt_printf("step_play: %d \n", step_play);

		if (step_count == FIRST_STEP) {
    		rt_printf("FIRST_STEP \n");
    	} else {
    		rt_printf("other step \n");
    	}
    	

    	rt_printf("Midi last_note_on: %d \n", last_note_on);
    	rt_printf("Midi last_note_off: %d \n", last_note_off);
    	rt_printf("Midi last_note_disabled: %d \n", last_note_disabled);
    	
    	rt_printf("Midi midi_channel_a: %d \n", midi_channel_a);
    	
    	
    
    	

    rt_printf("sequence_is_running is: %d \n", sequence_is_running);

    // Sequence Outputs 
    rt_printf("target_gate_out_state is: %d \n", target_gate_out_state);
		rt_printf("gate_out_state_set is: %d \n", gate_out_state_set);      


      //std::string message = "$simon!";
      
      //int signal = 1; 
      //float signal2 = 1.23456;
      //int my_result  = myUdpClient.send(&message, message.length()+1);

	  //int my_result  = myUdpClient.send(&message, 16);


	//	int my_result  = myUdpClient.send(&signal2, sizeof(float));

//rt_printf("sent %d  bytes \n", my_result);



	
	



      rt_printf("\n==== Bye from printStatus ======= \n");
      
      



    	
    	
    }




} 


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


void OnMidiNoteInEvent(uint8_t on_off, uint8_t note, uint8_t velocity, uint8_t channel){

  rt_printf("Hi from OnMidiNoteInEvent I got MIDI note Event ON/OFF is %d, Note is %d, Velocity is %d, Channel is %d bar_count is currently %d, step_count is currently %d \n", on_off, note, velocity, channel, bar_count, step_count);
  if (on_off == MIDI_NOTE_ON){

        // A mechanism to clear notes from memory by playing them quietly.
        if (velocity < 7 ){
           // Send Note OFF

           midi.writeNoteOff(channel, note, 0);
           
           // Disable the note on all steps
           //Serial.println(String("DISABLE Note (for all steps) ") + note + String(" because ON velocity is ") + velocity );
           DisableNotes(note);
           
           last_note_disabled = note;

          // Now, when we release this note on the keyboard, the keyboard obviously generates a note off which gets stored in channel_a_midi_note_events
          // and can interfere with subsequent note ONs i.e. cause the note to end earlier than expected.
          // Since velocity of Note OFF is not respected by keyboard manufactuers, we need to find a way remove (or prevent?)
          // these Note OFF events. 
          // One way is to store them here for processing after the note OFF actually happens. 
          channel_a_ghost_events[note].is_active=1;
    
        } else {
          // We want the note on, so set it on.
          rt_printf("Setting MIDI note ON for note %d When step is %d velocity is %d \n", note, step_count, velocity );
          // WRITE MIDI MIDI_DATA
          channel_a_midi_note_events[bar_count][step_count][note][1].tick_count_since_step = loop_timing.tick_count_since_step; // Only one of these per step.
          channel_a_midi_note_events[bar_count][step_count][note][1].velocity = velocity;
          channel_a_midi_note_events[bar_count][step_count][note][1].is_active = 1;
          
          // Echo Midi but only if the sequencer is stopped, else we get double notes because PlayMidi gets called each Tick
          if (sequence_is_running == 0){
          	midi.writeNoteOn(midi_channel_a, note, velocity); // echo midi to the output
          }
          
          
          last_note_on = note;
          
          rt_printf("Done setting MIDI note ON for note %d when step is %d velocity is %d \n", note,  step_count, velocity );

        } 
      
          
        } else {
          
            // Note Off
             rt_printf("Set MIDI note OFF for note %d when bar is %d and step is %d \n", note,  bar_count, step_count );
             
             // WRITE MIDI MIDI_DATA
             channel_a_midi_note_events[bar_count][step_count][note][0].tick_count_since_step = loop_timing.tick_count_since_step;
             channel_a_midi_note_events[bar_count][step_count][note][0].velocity = velocity;
             channel_a_midi_note_events[bar_count][step_count][note][0].is_active = 1;

			 last_note_off = note;
			
			// Echo Midibut only if the sequencer is stopped, else we get double notes because PlayMidi gets called each Tick
			if (sequence_is_running == 0){ 
				midi.writeNoteOff(midi_channel_a, note, 0);
			}
        	rt_printf("Done setting MIDI note OFF (Sent) for note %d when bar is %d and step is %d \n", note,  bar_count, step_count );
  }
  } 


void GateHigh(){
  //rt_printf("Gate HIGH at tick_count_since_start: %d ", loop_timing.tick_count_since_start);
  
  
  target_gate_out_state = true;
  envelope_1_audio.gate(true);
  step_triggered_envelope_2.gate(true);
  

  

}

void GateLow(){
  //rt_printf("Gate LOW");
  
  target_gate_out_state = false;
  
  envelope_1_audio.gate(false);
  step_triggered_envelope_2.gate(false);
  
  sequence_triggered_envelope_3.gate(false); // always reset it here but not trigger it
  
  

  

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
	
  //rt_printf("------SyncAndResetCv-----\n");	
  
  
  //envelope_1.gate(true);
  
}



// Return bth bit of number from https://stackoverflow.com/questions/2249731/how-do-i-get-bit-by-bit-data-from-an-integer-value-in-c
uint8_t ReadBit (int number, int b ){
	(number & ( 1 << b )) >> b;
}






/////////////////////////////////////////////////////////////
// These are the possible beats of the sequence
void OnStep(){
	
	


  

  //rt_printf("Hello from OnStep: %d \n", step_count);
  //rt_printf("the_sequence is: %d \n", the_sequence);
  //print_binary(the_sequence);
  //rt_printf("%c \n", 'B');




  if (step_count > MAX_STEP) {
    rt_printf("----------------------------------------------------------------------------\n");  
    rt_printf("------------------ ERROR! step_count is: %s --- ERROR ---\n", + step_count);
    rt_printf("----------------------------------------------------------------------------\n");    
  }

  

    if (step_count == FIRST_STEP) {
    	//rt_printf("----   -------   YES FIRST_STEP     -------    ------\n");
      SyncAndResetCv();
    } else {
      //rt_printf("----       not first step      step_count is %d FIRST_STEP is %d                  ------\n", step_count, FIRST_STEP ); 
    }
  
  
  step_count = StepCountSanity(step_count);

      // std::string message = "--:OnStep:" + std::to_string(step_count) + "--";
	 // This sends a UDP message 
	 // int my_result  = myUdpClient.send(&message, 32);
  
  
  uint8_t play_note = (the_sequence & ( 1 << step_count )) >> step_count;  
  
  // Why does the line below trigger "Xenomai/cobalt: watchdog triggered" whereas the same logic in this function does not?
  //uint8_t play_note = ReadBit(the_sequence, step_count);
  
   if (play_note){
     //rt_printf("OnStep: %d ****++++++****** PLAY \n", step_count);
    GateHigh(); 
   } else {
    GateLow();
     //rt_printf("OnStep: %d ***-----***** NOT play \n", step_count);
   }
   
   Bela_scheduleAuxiliaryTask(gPrintStatus);	

   //rt_printf("==== End of OnStep: %d \n", step_count);
      
}



// These are ticks which are not steps - so in between possible beats.
void OnNotStep(){
  //rt_printf("NOT step_countIn is: ") + step_countIn  ); 
  // TODO not sure how this worked before. function name? ChangeCvWaveformBAmplitude(); 
  GateLow();
  
}





















float gFreq;
float gPhaseIncrement = 0;
bool gIsNoteOn = 0;
int gVelocity = 0;

int my_note = 0;


float gSamplingPeriod = 0;
//int gSampleCount = 44100; // how often to send out a control change


float gPhase;
float gInverseSampleRate;
int gAudioFramesPerAnalogFrame = 0;



void SetPlayFromCount(){

  if (progression_input == 0){
    bar_play = bar_count;

  } else if (progression_input == 1)  {
      if (bar_count <= 1){
          bar_play = bar_count;  
      } else {
        bar_play = 0;
      }
  } else {
    bar_play = bar_count;
  }
  
  step_play = step_count;

}


// See http://docs.bela.io/classMidi.html for the Bela Midi stuff

void PlayMidi(){
  //rt_printf("midi_note  ") + i + String(" value is ") + channel_a_midi_note_events[step_count][i]  );

			// midi_byte_t statusByte = 0xB0; // control change on channel 0
			// midi_byte_t controller = 30; // controller number 30
			// midi_byte_t value = state * 127; // value : 0 or 127
			// midi_byte_t bytes[3] = {statusByte, controller, value};
			// midi.writeOutput(bytes, 3); // send a control change message



  for (uint8_t n = 0; n <= 127; n++) {
    //rt_printf("** OnStep  ") + step_count + String(" Note ") + n +  String(" ON value is ") + channel_a_midi_note_events[step_count][n][1]);
    
    // READ MIDI sequence
    if (channel_a_midi_note_events[BarCountSanity(bar_play)][StepCountSanity(step_play)][n][1].is_active == 1) { 
           // The note could be on one of 6 ticks in the sequence
           if (channel_a_midi_note_events[BarCountSanity(bar_play)][StepCountSanity(step_play)][n][1].tick_count_since_step == loop_timing.tick_count_since_step){
            	rt_printf("PlayMidi step_play: %d : tick_count_since_step %d Found and will send Note ON for %d \n", step_play, loop_timing.tick_count_since_step, n );
            	midi.writeNoteOn (midi_channel_a, n, channel_a_midi_note_events[BarCountSanity(bar_play)][StepCountSanity(step_count)][n][1].velocity);
           }
    } 

    // READ MIDI MIDI_DATA
    if (channel_a_midi_note_events[BarCountSanity(bar_play)][StepCountSanity(step_count)][n][0].is_active == 1) {
       if (channel_a_midi_note_events[BarCountSanity(bar_play)][StepCountSanity(step_count)][n][0].tick_count_since_step == loop_timing.tick_count_since_step){ 
           //rt_printf("Step:Ticks ") + step_count + String(":") + ticks_after_step +  String(" Found and will send Note OFF for ") + n );
           midi.writeNoteOff(midi_channel_a, n, 0);
       }
    }
  } // End midi note loop

} // End Play Midi





/////////
void AdvanceSequenceChronology(){
  
  // This function advances or resets the sequence powered by the clock.

  // But first check / set the desired sequence length

  //rt_printf("Hello from AdvanceSequenceChronology ");


  // Reverse because we want fully clockwise to be short so we get 1's if sequence is 1.
  //current_sequence_length_in_steps = MAX_SEQUENCE_LENGTH_IN_STEPS - current_sequence_length_in_steps_raw;

  //rt_printf("current_sequence_length_in_steps is: %d ", current_sequence_length_in_steps  );

  if (current_sequence_length_in_steps < MIN_SEQUENCE_LENGTH_IN_STEPS){
    rt_printf("**** ERROR with current_sequence_length_in_steps it WAS: %d but setting it to: %d ", current_sequence_length_in_steps, MIN_SEQUENCE_LENGTH_IN_STEPS );
    current_sequence_length_in_steps = MIN_SEQUENCE_LENGTH_IN_STEPS; 
    
  }
  
  if (current_sequence_length_in_steps > MAX_SEQUENCE_LENGTH_IN_STEPS){
    current_sequence_length_in_steps = MAX_SEQUENCE_LENGTH_IN_STEPS; 
    rt_printf("**** ERROR with current_sequence_length_in_steps but it is NOW: %d ", current_sequence_length_in_steps  );
  }

  new_sequence_length_in_ticks = (current_sequence_length_in_steps) * 6;
  //Serial.println(String("current_sequence_length_in_steps is: ") + current_sequence_length_in_steps  ); 
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
 
 SetPlayFromCount();

  
}



void OnTick(){
// Called on Every MIDI or Analogue clock pulse
// Drives sequencer activity.
// Can be called from Midi Clock and/or Digital Clock In.

 //rt_printf("Hello from OnTick \n");

  last_tick_frame = frame_timer;	

  /////////////////
  // BPM Detection
  if (loop_timing.tick_count_since_start % 24 == 0){
    // 1 Tick (clock pulse) = f Audio Frames
    // 1 Tick = f / 44100 (Audio Sample Rate) seconds
    // There are 44100/f Ticks per seconds
    // There are 44100 * 60 /f Ticks per minute
    // There are 44100 * 60 / (f * 24) Beats per minute 
    // For example 44100 * 60 / (920 * 24) = 120 BPM 

    // Instead of averaging over a few clock cycles and dividing by 24, count frames per 24 ticks 
    // (ticks per quarter note which is midi standard and used by Arturia Beat Step Pro etc.)
    previous_quarter_note_frame = last_quarter_note_frame; // 24 frames ago
    last_quarter_note_frame = frame_timer; // Now
    frames_per_24_ticks = last_quarter_note_frame - previous_quarter_note_frame;
    
    // We never want a zero value 
    if (frames_per_24_ticks == 0){
    	frames_per_24_ticks = INITIAL_FRAMES_PER_24_TICKS;
    }
    
    detected_bpm = audio_sample_rate * 60 / frames_per_24_ticks;
  }
 

  // Decide if we have a "step"
  if (loop_timing.tick_count_in_sequence % 6 == 0){
    //clockShowHigh();
    //rt_printf("loop_timing.tick_count_in_sequence is: ") + loop_timing.tick_count_in_sequence + String(" the first tick of a crotchet or after MIDI Start message") );    
    //////////////////////////////////////////
    OnStep();
    /////////////////////////////////////////   
  } else {
    //clockShowLow();
    // The other ticks which are not "steps".
    OnNotStep();
    //rt_printf("timing.tick_count_in_sequence is: ") + timing.tick_count_in_sequence );
  }

  
  
  // Play any suitable midi in the sequence (note, we read midi using a callback)

// Here we could clock divide 

  PlayMidi();
   
  // Advance and Reset ticks and steps
  AdvanceSequenceChronology();

}

////////////

// read midi loop

// start = 0xfa
// stop = 0xfc
// clock tick = 0xf8 (248 decimal)
// continue = 0xfb


// Nice reference: http://www.giordanobenicchi.it/midi-tech/midispec.htm


// See Midi.h in the Libraries section of the Bela IDE
void readMidiLoop(MidiChannelMessage message, void* arg){

	int MIDI_STATUS_OF_CLOCK = 7; // not  (decimal 248, hex 0xF8) ??
	
	// Read midi loop

	uint8_t type_received = message.getType();
	uint8_t note_received = message.getDataByte(0); // may have different meaning depending on the type
	uint8_t velocity_received = message.getDataByte(1); // may have different meaning depending on the type
	uint8_t channel_received = message.getChannel(); // we ignore the channel


	rt_printf("Midi Message: type: %d, note: %d, velocity: %d, channel: %d  \n", type_received, note_received, velocity_received, channel_received);

	//rt_printf("Note: kmmNoteOn: %d, kmmNoteOff: %d \n", kmmNoteOn, kmmNoteOff);

	// Check for a NOTE ON
    // Some keyboards send velocity 0 for note off instead of sending 0 for type, so we must check for that.
	if(type_received == kmmNoteOn && velocity_received > 0){
			rt_printf("note_received ON: type_received: %d, note_received: %d velocity_received %d Ignoring channel \n", type_received, note_received, velocity_received);
			// Write any note ON into the sequence
			OnMidiNoteInEvent(MIDI_NOTE_ON, note_received, velocity_received, midi_channel_a);
	// Check for a NOTE OFF	 
	} else if (message.getType() == kmmNoteOff || message.getDataByte(1) == 0){
		
			rt_printf("note_received OFF: type_received: %d, note_received: %d velocity_received %d Ignoring channel \n", type_received, note_received, velocity_received);
			
			// Write any note OFF into the sequence
			OnMidiNoteInEvent(MIDI_NOTE_OFF, note_received, velocity_received, midi_channel_a);
			
	// CLOCK but not currently working.
	} else if (message.getType() == MIDI_STATUS_OF_CLOCK) {
			// Midi clock  (decimal 248, hex 0xF8) - for some reason the library returns 7 for clock (kmmSystem ?)
		// int type = message.getType();
		// int byte0 = message.getDataByte(0);
		// int byte1 = message.getDataByte(1);
    	
    	
    	rt_printf("MAYBE CLOCK MIDI Message: type_received: %d, note_received: %d velocity_received %d Ignoring channel \n", type_received, note_received, velocity_received);
    	

		//rt_printf("THINK I GOT MIDI CLOCK - type: %d byte0: %d  byte1 : %d \n", type, byte0, byte1);
		
		// UNRELIABLE AT THE MOMENT
		
		//midi_clock_detected = 1;
		//OnTick();

	// OTHER 
	} else {
			rt_printf("OTHER MIDI Message: type_received: %d, note_received: %d velocity_received %d Ignoring channel \n", type_received, note_received, velocity_received);
	}
}





//////////////

// https://www.codesdope.com/blog/article/set-toggle-and-clear-a-bit-in-c/
// That will clear the nth bit of number. You must invert the bit string with the bitwise NOT operator (~), then AND it.
int BitClear (unsigned int number, unsigned int n) {
	// return number &= ~(1 << n);	
	return number &= ~(1UL << n);
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
  rt_printf("CvStop \n");
  
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


void AllNotesOff(){
	  // All MIDI notes off.
	    uint8_t channel = 0;
  rt_printf("All MIDI notes OFF \n");
  for (uint8_t n = 0; n <= 127; n++) {
     midi.writeNoteOff(channel, n, 0);
  }
  
}



void InitMidiSequence(){

  rt_printf("InitMidiSequence Start \n");

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
  
      //rt_printf("Init Step ") + sc + String(" Note ") + n +  String(" ON ticks value is ") + channel_a_midi_note_events[sc][n][1].is_active);
      //rt_printf("Init Step ") + sc + String(" Note ") + n +  String(" OFF ticks value is ") + channel_a_midi_note_events[sc][n][0].is_active);
      } 
    }
  }


   
  
    for (uint8_t n = 0; n <= 127; n++) {
     channel_a_ghost_events[n].is_active = 0;
     //rt_printf("Init Step with ghost Note: %s is_active false", n );
  }
  


  

	rt_printf("InitMidiSequence Done \n");
}




// Each time we start the sequencer we want to start from the same conditions.
void InitSequencer(){
  GateLow();
  CvStop();
  loop_timing.tick_count_since_start = 0;
  ResetSequenceCounters();
  InitMidiSequence();
  AllNotesOff();
}

void StartSequencer(){
  InitSequencer();
  sequence_is_running = HIGH;
}

void StopSequencer(){
//	auto millisec_since_epoch_2 = duration_cast<milliseconds>(system_clock::now().time_since_epoch()).count();
	
	
//	unsigned long milliseconds_since_epoch = std::chrono::system_clock::now().time_since_epoch();
	

  // Note the format %llu is used to format 64bit unsigned integer. 
  // see https://stackoverflow.com/questions/32112497/how-to-printf-a-64-bit-integer-as-hex
  // https://stackoverflow.com/questions/18107426/printf-format-for-unsigned-int64-on-windows
  //rt_printf("Stop Sequencer at: %llx \n", frame_timer);   // works - hex
  rt_printf("Stop Sequencer at: %llu \n", frame_timer); // works - unsigned
  
  
  // std::cout << millisec_since_epoch_2;
  
  
  InitSequencer();
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













////



void ChangeSequence(void*){
	
	 //rt_printf(" ChangeSequence " );
	
	
	
	uint8_t sequence_pattern_lower_limit = 1;  // Setting to 1 means we never get 0 i.e. a blank sequence especially when we change seq length
    unsigned int sequence_pattern_upper_limit = 1023; 
	
	sequence_pattern_input = static_cast<int>(round(map(sequence_pattern_input_raw, 0, 1, sequence_pattern_lower_limit, sequence_pattern_upper_limit))); 
    //rt_printf("**** NEW value for sequence_pattern_input is: %d ", sequence_pattern_input  );
    



    current_sequence_length_in_steps = static_cast<int>(round(map(sequence_length_input_raw, 0, 1, MIN_SEQUENCE_LENGTH_IN_STEPS, MAX_SEQUENCE_LENGTH_IN_STEPS))); 
    
 
    //////////////////////////////////////////
// Assign values to change the sequencer.
///////////////////////////////////

   last_binary_sequence_result = binary_sequence_result;

 

   // If we have 8 bits, use the range up to 255



  

//binary_sequence_upper_limit = pow(current_sequence_length_in_steps, 2);

// REMEMBER, current_sequence_length_in_steps is ONE indexed (from 1 up to 16) 
// For a 3 step sequence we want to cover all the possibilities of a 3 step sequence which is (2^3) - 1 = 7
// i.e. all bits on of a 3 step sequence is 111 = 7 decimal 
// or (2^current_sequence_length_in_steps) - 1
sequence_pattern_upper_limit = pow(2, current_sequence_length_in_steps) - 1; 



   //rt_printf("binary_sequence_upper_limit is: ") + binary_sequence_upper_limit  );
    


  // Button is in Normal state (not pressed) (HIGH) (button_1_state == HIGH)
   // ***UPPER Pot HIGH Button*** //////////
  // Generally the lowest value from the pot we get is 2 or 3 
  // setting-1
  binary_sequence_result = fscale( 1, 1023, sequence_pattern_lower_limit, sequence_pattern_upper_limit, sequence_pattern_input, 0);

   


   if (binary_sequence_result != last_binary_sequence_result){
    //rt_printf("binary_sequence has changed **");
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

    //the_sequence = BitClear(the_sequence, current_sequence_length_in_steps -1); // current_sequence_length_in_steps is 1 based index. bitClear is zero based index.
    //the_sequence = ~ the_sequence; // Invert

    // So pot fully counter clockwise is 1 on the first beat 
    // if (binary_sequence_result == 1){
    //   the_sequence = 1;
    // }

   //rt_printf("the_sequence is: %s ", the_sequence  );
   //Serial.print("\t");
   //Serial.print(the_sequence, BIN);
   //Serial.println();
   
   
   // TODO only do this if the value changes?
   
        envelope_1_audio.setAttackRate(envelope_1_attack * audio_sample_rate);
        envelope_1_audio.setDecayRate(envelope_1_decay * audio_sample_rate);
        envelope_1_audio.setSustainLevel(envelope_1_sustain);
        envelope_1_audio.setReleaseRate(envelope_1_release * audio_sample_rate);
        
        step_triggered_envelope_2.setAttackRate(envelope_1_attack * analog_sample_rate);
        step_triggered_envelope_2.setDecayRate(envelope_1_decay * analog_sample_rate);
        step_triggered_envelope_2.setReleaseRate(envelope_1_release * analog_sample_rate);
        step_triggered_envelope_2.setSustainLevel(envelope_1_sustain);

        sequence_triggered_envelope_3.setAttackRate(envelope_1_attack * audio_sample_rate);
        sequence_triggered_envelope_3.setDecayRate(envelope_1_decay * audio_sample_rate);
        sequence_triggered_envelope_3.setReleaseRate(envelope_1_release * audio_sample_rate);
        sequence_triggered_envelope_3.setSustainLevel(envelope_1_sustain);
        

	    
	    audio_osc_2_frequency = lfo_osc_1_frequency * 8.0;


    	oscillator_1_analog.setFrequency(lfo_osc_1_frequency); // lower freq
		oscillator_2_audio.setFrequency(audio_osc_2_frequency); // higher freq
		
		
		// We want to Delay Course Dial to span the DELAY_BUFFER_SIZE in jumps of frames_per_24_ticks
		float delay_coarse_dial_factor = DELAY_BUFFER_SIZE / (frames_per_24_ticks * MAX_COARSE_DELAY_TIME_INPUT);
		
		
		// The course delay amount we dial in with the pot
		coarse_delay_frames = rint(frames_per_24_ticks * coarse_delay_input * delay_coarse_dial_factor);	    
	    
		// The amount we increment / decrement the delay using buttons 2 and 1
		fine_delay_frames_delta = rint(frames_per_24_ticks / 24.0);		
		

		// Fine Delay Time
		// Smaller	
		if (do_button_1_action == 1) {
			
			if ((fine_delay_frames - fine_delay_frames_delta) < 0){
				// Skip
			} else { 
				fine_delay_frames = fine_delay_frames - fine_delay_frames_delta;
			}			
			
			do_button_1_action = 0;
		
		// Larger	
		} else if (do_button_2_action == 1) {
			
			if ((coarse_delay_frames + fine_delay_frames + fine_delay_frames_delta) >= DELAY_BUFFER_SIZE){
				// Skip
			} else {
				fine_delay_frames = fine_delay_frames + fine_delay_frames_delta;
			}
			
			do_button_2_action = 0;
		} 
			
		total_delay_frames = coarse_delay_frames + fine_delay_frames;

		// Sanity Checks
	    if (total_delay_frames > DELAY_BUFFER_SIZE){
	    	total_delay_frames = DELAY_BUFFER_SIZE;
		}
		
		if (total_delay_frames < 0){
	    	total_delay_frames = 0;
		}






	
} // end of function






void MaybeOnTick(){
  if (do_tick == true){
    do_tick = false;
    OnTick();
  }
}

////////

// from Bela delay example:

//#define DELAY_BUFFER_SIZE 44100

// 400000 enough for 16 step sequence at 30 bpm
// Then a maximum multiplier of 16

// 400000 * 16 





// Buffer holding previous samples per channel
float gDelayBuffer_l[DELAY_BUFFER_SIZE] = {0};
float gDelayBuffer_r[DELAY_BUFFER_SIZE] = {0};
// Write pointer
int gDelayBufWritePtr = 0;
// Amount of delay
float gDelayAmount = 1.0;

// Level of pre-delay input
float gDelayAmountPre = 0.75;


// Butterworth coefficients for low-pass filter @ 8000Hz
float gDel_a0 = 0.1772443606634904;
float gDel_a1 = 0.3544887213269808;
float gDel_a2 = 0.1772443606634904;
float gDel_a3 = -0.5087156198145868;
float gDel_a4 = 0.2176930624685485;

// Previous two input and output values for each channel (required for applying the filter)
float gDel_x1_l = 0;
float gDel_x2_l = 0;
float gDel_y1_l = 0;
float gDel_y2_l = 0;
float gDel_x1_r = 0;
float gDel_x2_r = 0;
float gDel_y1_r = 0;
float gDel_y2_r = 0;

///////// End of Bela Delay example



/////////////////////////////////////////////////////////

bool setup(BelaContext *context, void *userData){
	
//	rt_printf("Hello from Setup: SimonSaysSeeq on Bela %s:-) \n", version);

	rt_printf("Hello from Setup: SimonSaysSeeq on Bela  \n");

	scope.setup(4, context->audioSampleRate);

	oscillator_1_analog.setup(context->analogSampleRate);	
	oscillator_2_audio.setup(context->audioSampleRate);

	

	

 
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
	midi.enableParser(true);
	midi.setParserCallback(readMidiLoop, (void*) gMidiPort0);
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
        
        //rt_printf("Creating an oscilator in Setup. \n");
        
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
        //if((gFrequencyUpdateTask = Bela_createAuxiliaryTask(&recalculate_frequencies, 85, "bela-update-frequencies")) == 0)
        //        return false;
        
        

        audio_sample_rate = context->audioSampleRate;
        analog_sample_rate = context->analogSampleRate;


        // Set ADSR parameters
        envelope_1_audio.setAttackRate(envelope_1_attack * context->audioSampleRate);
        envelope_1_audio.setDecayRate(envelope_1_decay * context->audioSampleRate);
        envelope_1_audio.setReleaseRate(envelope_1_release * context->audioSampleRate);
        envelope_1_audio.setSustainLevel(envelope_1_sustain);
        
        // This envelope triggers on each step
        step_triggered_envelope_2.setAttackRate(envelope_1_attack  * context->analogSampleRate);
        step_triggered_envelope_2.setDecayRate(envelope_1_decay * context->analogSampleRate);
        step_triggered_envelope_2.setReleaseRate(envelope_1_release * context->analogSampleRate);
        step_triggered_envelope_2.setSustainLevel(envelope_1_sustain);


        sequence_triggered_envelope_3.setAttackRate(envelope_1_attack  * context->analogSampleRate);
        sequence_triggered_envelope_3.setDecayRate(envelope_1_decay * context->analogSampleRate);
        sequence_triggered_envelope_3.setReleaseRate(envelope_1_release * context->analogSampleRate);
        sequence_triggered_envelope_3.setSustainLevel(envelope_1_sustain);


        // Set buttons pins as inputs
        pinMode(context, 0, button_1_PIN, INPUT);
        pinMode(context, 0, button_2_PIN, INPUT);


        // The two LEDS on Salt
        pinMode(context, 0, LED_1_PIN, OUTPUT);
        pinMode(context, 0, LED_2_PIN, OUTPUT);






        if((gChangeSequenceTask = Bela_createAuxiliaryTask(&ChangeSequence, 83, "bela-change-sequence")) == 0)
                return false;
        
        if((gPrintStatus = Bela_createAuxiliaryTask(&printStatus, 80, "bela-print-status")) == 0)
                return false;

    
        
        
        gSampleCount = 0;
        
        //myUdpClient.setup(50002, "18.195.30.76"); 
        
        rt_printf("Bye from Setup \n");

        return true;
}




void render(BelaContext *context, void *userData)
{

  ///////////////////////////////////////////
  // Look for Analogue Clock (24 PPQ)

	// Use this global variable as a timing device
	// Set other time points from this frame_timer
	frame_timer = context->audioFramesElapsed;


  // AUDIO LOOP
	for(unsigned int n = 0; n < context->audioFrames; n++) {
		
		audio_envelope_1_amplitude  = 1.0 * envelope_1_audio.process();
		audio_osc_1_result = oscillator_2_audio.process() * audio_envelope_1_amplitude;
		
        ////////////////////////////////
		    // Begin Bela delay example code
		    float out_l = 0;
        float out_r = 0;
        
        // Read audio inputs
        out_l = audioRead(context,n,0);
        out_r = audioRead(context,n,1);
        
        // Increment delay buffer write pointer
        if(++gDelayBufWritePtr>DELAY_BUFFER_SIZE)
            gDelayBufWritePtr = 0;
        
        // Calculate the sample that will be written into the delay buffer...
        // 1. Multiply the current (dry) sample by the pre-delay gain level (set above)
        // 2. Get the previously delayed sample from the buffer, multiply it by the feedback gain and add it to the current sample
        float del_input_l = (gDelayAmountPre * out_l + gDelayBuffer_l[(gDelayBufWritePtr - total_delay_frames + DELAY_BUFFER_SIZE)%DELAY_BUFFER_SIZE] * delay_feedback_amount);
        float del_input_r = (gDelayAmountPre * out_r + gDelayBuffer_r[(gDelayBufWritePtr - total_delay_frames + DELAY_BUFFER_SIZE)%DELAY_BUFFER_SIZE] * delay_feedback_amount);
        
        // ...but let's not write it into the buffer yet! First we need to apply the low-pass filter!
        
        // Remember these values so that we can update the filter later, as we're about to overwrite it
        float temp_x_l = del_input_l;
        float temp_x_r = del_input_r;
        
        // Apply the butterworth filter (y = a0*x0 + a1*x1 + a2*x2 + a3*y1 + a4*y2)
        del_input_l = gDel_a0*del_input_l
                    + gDel_a1*gDel_x1_l
                    + gDel_a2*gDel_x2_l
                    + gDel_a3*gDelayBuffer_l[(gDelayBufWritePtr-1+DELAY_BUFFER_SIZE)%DELAY_BUFFER_SIZE]
                    + gDel_a4*gDelayBuffer_l[(gDelayBufWritePtr-2+DELAY_BUFFER_SIZE)%DELAY_BUFFER_SIZE];
        
        // Update previous values for next iteration of filter processing
        gDel_x2_l = gDel_x1_l;
        gDel_x1_l = temp_x_l;
        gDel_y2_l = gDel_y1_l;
        gDel_y1_l = del_input_l;
        
        // Repeat process for the right channel
        del_input_r = gDel_a0*del_input_r
                    + gDel_a1*gDel_x1_r
                    + gDel_a2*gDel_x2_r
                    + gDel_a3*gDelayBuffer_r[(gDelayBufWritePtr-1+DELAY_BUFFER_SIZE)%DELAY_BUFFER_SIZE]
                    + gDel_a4*gDelayBuffer_r[(gDelayBufWritePtr-2+DELAY_BUFFER_SIZE)%DELAY_BUFFER_SIZE];
    
        gDel_x2_r = gDel_x1_r;
        gDel_x1_r = temp_x_r;
        gDel_y2_r = gDel_y1_r;
        gDel_y1_r = del_input_r;
        
        //  Now we can write it into the delay buffer
        gDelayBuffer_l[gDelayBufWritePtr] = del_input_l;
        gDelayBuffer_r[gDelayBufWritePtr] = del_input_r;
        
        // Get the delayed sample (by reading `coarse_delay_frames` many samples behind our current write pointer) and add it to our output sample
        out_l += gDelayBuffer_l[(gDelayBufWritePtr - total_delay_frames + DELAY_BUFFER_SIZE)%DELAY_BUFFER_SIZE] * gDelayAmount;
        out_r += gDelayBuffer_r[(gDelayBufWritePtr - total_delay_frames + DELAY_BUFFER_SIZE)%DELAY_BUFFER_SIZE] * gDelayAmount;
        
        // Write the sample into the output buffer -- done!
        audioWrite(context, n, 0, out_l);
        audioWrite(context, n, 1, out_r);
		    // End Bela delay example code
		    //////////////////////////////
		
		
		
		
		
		for(unsigned int ch = 0; ch < gAudioChannelNum; ch++){
			// Pass input to output
			
			
			
			
			// todo create separate vars for oscillator_2_audio_output

			// Disable whilst trying delay things				
			// if (ch == 0){
			// 	audioWrite(context, n, ch, audio_osc_1_result);
			// } else { // ch 1
			// 	audioWrite(context, n, ch, audioRead(context, n, ch));
			// }
			
			if (ch == 0) {
				audio_left_input_raw = audioRead(context, n, ch);
			}
			
			if (ch == 1) {
				audio_right_input_raw = audioRead(context, n, ch);
			}
			
			
		}
	}

// ANALOG LOOP
	for(unsigned int n = 0; n < context->analogFrames; n++) {

		// Process analog oscillator	
		osc_2_result_analog = oscillator_1_analog.process();
		
		// Process step triggered analog envelope
		analog_envelope_2_amplitude  = step_triggered_envelope_2.process();  
		
		// Process the sequence triggered (i.e. every 4 - 16 beats) envelope
		analog_sequence_triggered_envelope_3_amplitude  = env3_amp * sequence_triggered_envelope_3.process();
		

		
		
		// Modulated output
		analog_out_2 = osc_2_result_analog * analog_envelope_2_amplitude; 
		
		// Plain envelope This is like a gate at the start of sequence plus release (so can use it as both a gate and an envelope)
		analog_out_3 = analog_sequence_triggered_envelope_3_amplitude;
		
		// Additive output
		analog_out_4 = ( osc_2_result_analog + analog_envelope_2_amplitude ) / 2.0;
		
		
		
		
		for(unsigned int ch = 0; ch < gAnalogChannelNum; ch++){
			
	      // INPUTS 		
		  if (ch == SEQUENCE_LENGTH_ANALOG_INPUT_PIN){
		  	sequence_length_input_raw = analogRead(context, n, SEQUENCE_LENGTH_ANALOG_INPUT_PIN);
		  }	
	
	
	      // Get the sequence_pattern_input_raw
	      if (ch == SEQUENCE_PATTERN_ANALOG_INPUT_PIN ){
	      	// note this is getting all the frames 
	        sequence_pattern_input_raw = analogRead(context, n, SEQUENCE_PATTERN_ANALOG_INPUT_PIN);
	        
	        
	        //rt_printf("Set sequence_pattern_input_raw %d ", sequence_pattern_input_raw); 
	        
	        //rt_printf("Set sequence_pattern_input_raw %f ", analogRead(context, n, SEQUENCE_PATTERN_ANALOG_INPUT_PIN)); 
	        
	        
	        //sequence_pattern_input = static_cast<double>(round(map(sequence_pattern_input_raw, 0.0, 1.0, 0.0, 255.0))); // GetValue(sequence_pattern_input_raw, sequence_pattern_input, jitter_reduction);
	    	//rt_printf("**** NEW value for sequence_pattern_input is: %d ", sequence_pattern_input  );
	        
	        
	      }
	      
	      if (ch == OSC_FREQUENCY_INPUT_PIN){
	      	
	      	lfo_osc_1_frequency = map(analogRead(context, n, OSC_FREQUENCY_INPUT_PIN), 0, 1, 0.05, 20);
 
	      	
	      	//envelope_1_attack = map(analogRead(context, n, OSC_FREQUENCY_INPUT_PIN), 0, 1, 0.001, 0.5);
	      	//envelope_1_decay = map(analogRead(context, n, OSC_FREQUENCY_INPUT_PIN), 0, 1, 0.5, 3.0);
	      	
		  	 // want range 0 to 3 seconds
		  	//envelope_2_attack = analogRead(context, n, OSC_FREQUENCY_INPUT_PIN);
		  }
		  
		  if (ch == ADSR_RELEASE_INPUT_PIN){
		  	
		  	// TODO use an oscillator here in stead.
		  	envelope_1_release = map(analogRead(context, n, ADSR_RELEASE_INPUT_PIN), 0, 1, 0.01, 5.0);
		  	
		  //	lfo_b_frequency_input_raw = analogRead(context, n, ADSR_RELEASE_INPUT_PIN);
		  }
		  
		  
		 
		  
		  if (ch == COARSE_DELAY_TIME_INPUT_PIN){
		  	coarse_delay_input = map(analogRead(context, n, COARSE_DELAY_TIME_INPUT_PIN), 0, 1, 0, MAX_COARSE_DELAY_TIME_INPUT);
		  	
		  }
		  
		  // > 0.999 leads to distorsion
		  if (ch == DELAY_FEEDBACK_INPUT_PIN){
		  	delay_feedback_amount = map(analogRead(context, n, DELAY_FEEDBACK_INPUT_PIN), 0, 1, 0, 1.8);
		  	
		  }
		  
		  
		  if (ch == CLOCK_DIVIDE_INPUT_PIN){
		  	clock_divide_input = floor(map(analogRead(context, n, CLOCK_DIVIDE_INPUT_PIN), 0, 1, 0, MAX_CLOCK_DIVIDE_INPUT));
		  	
		  }
		  
		  // > 0.999 leads to distorsion
		  if (ch == PROGRESSION_INPUT_PIN){
		  	progression_input = floor(map(analogRead(context, n, PROGRESSION_INPUT_PIN), 0, 1, 0, MAX_PROGRESSION_INPUT));
		  	
		  }
		  
		  
		  
		  
	      
	      // OUTPUTS
	      // CV 1 ** GATE ** 
	      if (ch == SEQUENCE_GATE_OUTPUT_1_PIN){
	      	if (target_gate_out_state == HIGH){
	      		analog_out_1 = 1.0;
	      	} else {
	      		analog_out_1 = -1.0;	
	      	}
	      	analogWrite(context, n, ch, analog_out_1);
	    	
	      }

	      // CV 2
	      if (ch == SEQUENCE_CV_OUTPUT_2_PIN){


	      	//rt_printf("amp is: %f", amp);
	      	analogWrite(context, n, ch, analog_out_2);
	      }


	      // CV 3 - This is below the left hand LED so should be related to the Length of the Sequence (Simple decaying envelope)
	      if (ch == SEQUENCE_CV_OUTPUT_3_PIN){

	      	//rt_printf("amp is: %f", amp);
	      	analogWrite(context, n, ch, analog_out_3);
	      }
	      
	      // CV 4
	      if (ch == SEQUENCE_CV_OUTPUT_4_PIN){


	      	//rt_printf("amp is: %f", amp);
	      	analogWrite(context, n, ch, analog_out_4);
	      }
	      
	      scope.log(analog_out_1, analog_out_2, analog_out_3, analog_out_4);

		}
	}
	

	
	Bela_scheduleAuxiliaryTask(gChangeSequenceTask);
	
	
	// DIGITAL LOOP 
		// Looking at all frames in case the transition happens in these frames. However, as its a clock we could maybe look at only the first frame.
	    for(unsigned int m = 0; m < context->digitalFrames; m++) {
	    
        	// Next state
        	new_digital_clock_in_state = digitalRead(context, m, CLOCK_INPUT_DIGITAL_PIN);
        	
        	
        	
        	old_button_1_state = new_button_1_state;
        	new_button_1_state = digitalRead(context, m, button_1_PIN);

        	old_button_2_state = new_button_2_state;
        	new_button_2_state = digitalRead(context, m, button_2_PIN);

        	old_button_3_state = new_button_3_state;
        	new_button_3_state = digitalRead(context, m, button_3_PIN);

        	old_button_4_state = new_button_4_state;
        	new_button_4_state = digitalRead(context, m, button_4_PIN);




        	old_both_buttons_pressed_state = new_both_buttons_pressed_state;
        	
        	if ((new_button_1_state == 1 && new_button_2_state == 1) && old_both_buttons_pressed_state == 0) {
        		both_buttons_pressed_counter = both_buttons_pressed_counter + 1;
        		new_both_buttons_pressed_state = 1;
        		
        		if ((both_buttons_pressed_counter % 2) == 0){
        			both_buttons_pressed_even = 1;
    
        			do_both_buttons_action_a = 1;
        			do_both_buttons_action_b = 0;
        		} else {
        		    both_buttons_pressed_even = 0;
        		    do_both_buttons_action_a = 0;
        		    do_both_buttons_action_b = 1;
        		}
        		
        		// Reset the buttons becuase we don't want a one button action as well
        		new_button_1_state = 0;
        		new_button_2_state = 0;
        		
        	} else {
        		new_both_buttons_pressed_state = 0;	
 
        	
	        	// Left button newly pressed get smaller
	        	if ((new_button_1_state != old_button_1_state) && new_button_1_state == 1){
	        		do_button_1_action = 1;
	        	}
	        	
	        	 // Right button newly pressed 
	        	if ((new_button_2_state != old_button_2_state) && new_button_2_state == 1){
	        		do_button_2_action = 1;
	        	}
        	
        	
        		if ((new_button_3_state != old_button_3_state) && new_button_3_state == 1){
	        		do_button_3_action = 1;
	        	}
	        	
	        	 // Right button newly pressed 
	        	if ((new_button_4_state != old_button_4_state) && new_button_4_state == 1){
	        		do_button_4_action = 1;
	        	}
        	
        	
        	}// End both buttons pressed check
        	
        	
        
        	// Only set new state if target is changed
        	if (target_gate_out_state != gate_out_state_set){
        		// 0 to 3.3V ? Salt docs says its 0 to 5 V (Eurorack trigger voltage is 0 - 5V)
	        	digitalWrite(context, m, SEQUENCE_OUT_PIN, target_gate_out_state);
	        	gate_out_state_set = target_gate_out_state;
        	}


          // Drive the LEDS. See https://github.com/BelaPlatform/Bela/wiki/Salt#led-and-pwm
          if (gate_out_state_set == HIGH){
            digitalWriteOnce(context, m, LED_1_PIN, LOW);
            
          } else {
            digitalWriteOnce(context, m, LED_1_PIN, HIGH);

          }
        	
        	// Do similar for another PIN for if (step_count == FIRST_STEP)
        	


            // If detect a rising clock edge
            if ((new_digital_clock_in_state == HIGH) && (current_digital_clock_in_state == LOW)){
              
              current_digital_clock_in_state = HIGH;

     


              
              if (sequence_is_running == LOW){
                StartSequencer();
              }
               
              OnTick();
              
            } 
            
            // If detect a Falling clock edge
            if ((new_digital_clock_in_state == LOW) && (current_digital_clock_in_state == HIGH)){
              current_digital_clock_in_state = LOW;
              
              //last_clock_falling_edge = frame_timer;
            	
            	
				// the pulse width of our clock (half actually)
            	//clock_width = last_clock_falling_edge - last_quarter_note_frame;
            	//rt_printf("clock_width is: %llu \n", clock_width);
            	
            	// currently a constant 
            	//clock_patience = clock_width * 100;
            } 
      }
      
     
      
      
      /////////////////////////////////////////////////////////////////////////////////
		// When relying on the analogue / digital (non midi) clock, we don't have a stop as such, so if we don't detect a clock for a while, then assume its stopped.
		// Note that the Beat Step Pro takes a while to kill its clock out after pressing the Stop button.
		if (midi_clock_detected == LOW){
    		elapsed_frames_since_last_tick = frame_timer - last_tick_frame;
			//rt_printf("elapsed_frames_since_last_tick is: %d ", elapsed_frames_since_last_tick);	
		
		
		    // clock patience is a quarter note
		    // We wait for a quarter note before stopping the sequencer (based on the last quarter note time in frames)
			if (elapsed_frames_since_last_tick > frames_per_24_ticks){
		  		if (sequence_is_running == HIGH) {
		    		rt_printf("Stopping sequencer because elapsed_frames_since_last_tick: %llu is greater than frames_per_24_ticks: %llu \n", elapsed_frames_since_last_tick, frames_per_24_ticks);
			    	StopSequencer();
				}
			}
		  

	
		// Temp code until we have clock
	   //temp_count++;
	   // if(temp_count % 1000 == 0) {
	   // 	OnTick();	
	   // }
	


  

        } 
	} // End of render
   
   




   
   


   


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




/*
 * udpClient.h
 *
 *  Created on: 19 May 2015
 *      Author: giulio moro
 */

#ifndef UDPCLIENT_H_
#define UDPCLIENT_H_

#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <netdb.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>

class UdpClient{
	private:
		int port;
		int outSocket;
		struct timeval stTimeOut;
		fd_set stWriteFDS;
		bool enabled = false;
		bool isSetPort = false;
		bool isSetServer = false;
		struct sockaddr_in destinationServer;
	public:
		UdpClient();
		UdpClient(int aPort, const char* aServerName);
		~UdpClient();
		bool setup(int aPort, const char* aServerName);
		void cleanup();
		/**
		 * Sets the port.
		 *
		 * Sets the port on the destination server.
		 * @param aPort the destineation port.
		 */
		void setPort(int aPort);

		/**
		 * Sets the server.
		 *
		 * Sets the IP address of the destinatioon server.
		 * @param aServerName the IP address of the destination server
		 */
		void setServer(const char* aServerName);

		/**
		 * Sends a packet.
		 *
		 * Sends a UPD packet to the destination server on the destination port.
		 * @param message A pointer to the location in memory which contains the message to be sent.
		 * @param size The number of bytes to be read from memory and sent to the destination.
		 * @return the number of bytes sent or -1 if an error occurred.
		 */
		int send(void* message, int size);

		int write(const char* remoteHostname, int remotePortNumber, void* sourceBuffer, int numBytesToWrite);
		int waitUntilReady(bool readyForReading, int timeoutMsecs);
		int setSocketBroadcast(int broadcastEnable);
};



#endif /* UDPCLIENT_H_ */

