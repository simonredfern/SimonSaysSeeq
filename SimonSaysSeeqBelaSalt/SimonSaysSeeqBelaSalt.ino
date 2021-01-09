 /*

                                   .-'''-.                                                                                                                     
                                  '   _    \                                                                                                        .-''-.     
           .--. __  __   ___    /   /` '.   \    _..._                                                         __.....__           __.....__       //'` `\|    
           |__||  |/  `.'   `. .   |     \  '  .'     '.                    .-.          .-                .-''         '.     .-''         '.    '/'    '|    
           .--.|   .-.  .-.   '|   '      |  '.   .-.   .                    \ \        / /               /     .-''"'-.  `.  /     .-''"'-.  `. |'      '|    
           |  ||  |  |  |  |  |\    \     / / |  '   '  |               __    \ \      / /               /     /________\   \/     /________\   \||     /||    
       _   |  ||  |  |  |  |  | `.   ` ..' /  |  |   |  |       _    .:--.'.   \ \    / /   _         _  |                  ||                  | \'. .'/||    
     .' |  |  ||  |  |  |  |  |    '-...-'`   |  |   |  |     .' |  / |   \ |   \ \  / /  .' |      .' | \    .-------------'\    .-------------'  `--'` ||    
    .   | /|  ||  |  |  |  |  |               |  |   |  |    .   | /`" __ | |    \ `  /  .   | /   .   | /\    '-.____...---. \    '-.____...---.        ||    
  .'.'| |//|__||__|  |__|  |__|               |  |   |  |  .'.'| |// .'.''| |     \  / .'.'| |// .'.'| |// `.             .'   `.             .'         || /> 
.'.'.-'  /                                    |  |   |  |.'.'.-'  / / /   | |_    / /.'.'.-'  /.'.'.-'  /    `''-...... -'       `''-...... -'           ||//  
.'   \_.'                                     |  |   |  |.'   \_.'  \ \._,\ '/|`-' / .'   \_.' .'   \_.'                                                 |'/   
                                              '--'   '--'            `--'  `"  '..'                                                                      |/    

SIMON SAYS SEEQ is released under the AGPL and (c) Simon Redfern 2020, 2021 

This file uses Bela, see below:

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

#include <libraries/ADSR/ADSR.h>





#include <chrono>

#include <libraries/UdpClient/UdpClient.h>

UdpClient myUdpClient;

// ...



uint64_t frame_timer = 0;

uint64_t last_clock_falling_edge = 0; 

uint64_t last_clock_rising_edge = 0;
 
int clock_width = 0;

uint64_t elapsed_since_last_clock_rising_edge = 0;

int clock_patience = 112000;



int audio_sample_rate;
int analog_sample_rate;

// Return now as milliseconds https://en.cppreference.com/w/cpp/chrono/duration/duration_cast
using namespace std::chrono;
milliseconds ms = duration_cast< milliseconds >(
    system_clock::now().time_since_epoch()
);


 








#include <math.h> //sinf
#include <time.h> //time
#include <libraries/Oscillator/Oscillator.h>
#include <libraries/OscillatorBank/OscillatorBank.h>
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

Oscillator oscillator_1_audio;
Oscillator oscillator_2_analog;


int gAudioChannelNum; // number of audio channels to iterate over
int gAnalogChannelNum; // number of analog channels to iterate over


int gTriggerButtonPin = 0; // Digital pin to which gate button should be connected
int gTriggerButtonLastStatus = 0; // Last status of gate button

int gModeButtonPin = 1; // Digital pin to which oscillator selection button should be connected
int gModeButtonLastStatus = 0; // Last status of oscillator selection button


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
const int SEQUENCE_LENGTH_ANALOG_INPUT_PIN = 1; // CV 2 input
const int OSC_FREQUENCY_INPUT_PIN = 2; // CV 3 input
const int ADSR_RELEASE_INPUT_PIN = 3; // CV 4 input



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

const uint8_t MIN_SEQUENCE_LENGTH_IN_STEPS = 4; // ONE INDEXED
const uint8_t MAX_SEQUENCE_LENGTH_IN_STEPS = 16; // ONE INDEXED

// Sequence Length (and default)
uint8_t current_sequence_length_in_steps = 8; 

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
float sequence_pattern_input_raw;
unsigned int sequence_pattern_input = 20;
unsigned int sequence_pattern_input_last;
unsigned int sequence_pattern_input_at_button_change;

//bool lower_pot_high_engaged = true;
float lfo_a_frequency_input_raw;
float frequency_1;
float frequency_2;
float frequency_3;


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


double	wait_time_ms;


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
  //rt_printf("ResetSequenceCounters Done. current_sequence_length_in_steps is: %d step_count is now: %d \n", current_sequence_length_in_steps, step_count);
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







float gInterval = 0.5;
float gSecondsElapsed = 0;
int gCount = 0;

int temp_count = 0;



// for ADSR

ADSR envelope_1_audio; // ADSR envelope_1
ADSR envelope_2_analog;
ADSR envelope_3;

float audio_envelope_1_amplitude = 0;
float analog_envelope_2_amplitude = 0;
float analog_envelope_3_amplitude = 0;


float envelope_1_attack = 0.0001; // envelope_1_audio attack (seconds)
float envelope_1_decay = 0.1; // envelope_1 decay (seconds)
float envelope_1_sustain = 0.9; // envelope_1 sustain level
float envelope_1_release = 0.5; // envelope_1 release (seconds)

float analog_out_1; // not used
float analog_out_2;
float analog_out_3;
float analog_out_4;


float audio_osc_1_result;
float analog_osc_2_result;
float analog_osc_3_result;

// float envelope_2_attack = 0.0001; // envelope_2 attack (seconds)
// float envelope_2_decay = 0.25; // envelope_2 decay (seconds)
// float envelope_2_sustain = 0.9; // envelope_2 sustain level
// float envelope_2_release = 0.5; // envelope_2 release (seconds)

// float envelope_3_attack = 0.0001; // envelope_2 attack (seconds)
// float envelope_3_decay = 0.25; // envelope_2 decay (seconds)
// float envelope_3_sustain = 0.9; // envelope_2 sustain level
// float envelope_3_release = 0.5; // envelope_2 release (seconds)



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









/// end for ADSR






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


void printStatus(void*){

    // Might not want to print every time else we overload the CPU
    gCount++;
	
    if(gCount % 1 == 0) {
      //rt_printf("================ \n");
		rt_printf("======== Hello from printStatus. gCount is: %d ========= \n",gCount);

		// Global frame timing

		rt_printf("frame_timer is: %llu \n", frame_timer);
		
		
		// Analog / Digital Clock In.
  		rt_printf("last_clock_rising_edge is: %llu \n", last_clock_rising_edge);
		rt_printf("last_clock_falling_edge is: %llu \n", last_clock_falling_edge);
		rt_printf("clock_width is: %d \n", clock_width);
		rt_printf("clock_patience is: %d \n", clock_patience);

		// Other Inputs

    	rt_printf("sequence_pattern_input_raw is: %f \n", sequence_pattern_input_raw);
		rt_printf("sequence_pattern_input is: %d \n", sequence_pattern_input);
	  
		rt_printf("sequence_length_input_raw is: %f \n", sequence_length_input_raw);
    	//rt_printf("sequence_length_input is: %d \n", sequence_length_input);
      



    	rt_printf("envelope_1_attack is: %f \n", envelope_1_attack);
    	rt_printf("envelope_1_decay is: %f \n", envelope_1_decay);
    	rt_printf("envelope_1_release is: %f \n", envelope_1_release);
    	
    	rt_printf("audio_envelope_1_amplitude is: %f \n", audio_envelope_1_amplitude);
    	rt_printf("analog_envelope_2_amplitude is: %f \n", analog_envelope_2_amplitude);
    	rt_printf("analog_envelope_3_amplitude is: %f \n", analog_envelope_3_amplitude);
    	

    	rt_printf("lfo_b_frequency_input_raw is: %f \n", lfo_b_frequency_input_raw);
    	
    	rt_printf("frequency_1 is: %f \n", frequency_1);
		rt_printf("frequency_2 is: %f \n", frequency_2);
		rt_printf("frequency_3 is: %f \n", frequency_3);
		
		rt_printf("audio_osc_1_result is: %f \n", audio_osc_1_result);
		rt_printf("analog_osc_2_result is: %f \n", analog_osc_2_result);
		
		
		
		rt_printf("analog_out_1 is: %f \n", analog_out_1);
		rt_printf("analog_out_2 is: %f \n", analog_out_2);
		rt_printf("analog_out_3 is: %f \n", analog_out_3);
		rt_printf("analog_out_4 is: %f \n", analog_out_4);
		

		rt_printf("audio_left_input_raw is: %f \n", audio_left_input_raw);	
		rt_printf("audio_right_input_raw is: %f \n", audio_right_input_raw);

		// Clock derived values

    	rt_printf("analog_clock_in_state is: %d \n", analog_clock_in_state);

    	rt_printf("current_digital_clock_in_state is: %d \n", current_digital_clock_in_state);
    	rt_printf("new_digital_clock_in_state is: %d \n", new_digital_clock_in_state);

    	rt_printf("midi_clock_detected is: %d \n", midi_clock_detected);
    	

    	// rt_printf("loop_timing.tick_count_in_sequence is: %d \n", loop_timing.tick_count_in_sequence);
    	// rt_printf("loop_timing.tick_count_since_start is: %d \n", loop_timing.tick_count_since_start);

    	
    	// Sequence derived results 
    	
    	
    	
    	
    	rt_printf("current_sequence_length_in_steps is: %d \n", current_sequence_length_in_steps);
    	
    	rt_printf("binary_sequence_result is: %d \n", binary_sequence_result);

		rt_printf("gray_code_sequence is: %d \n", gray_code_sequence);
		print_binary(gray_code_sequence);
		rt_printf("%c \n", 'B');


    	rt_printf("the_sequence is: %d \n", the_sequence);
    	print_binary(the_sequence);
		rt_printf("%c \n", 'B');

		// Sequence state
		
		rt_printf("step_count: %d \n", step_count);
		
		if (step_count == FIRST_STEP) {
    		rt_printf("FIRST_STEP \n");
    	} else {
    		rt_printf("other step \n");
    	}


		
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
      
      



///////      
/*
  // can't open a url with this lib it seems libcurl is tricky to link.

    char url[1000] = "https://apisandbox.openbankproject.com/obp/v4.0.0/adapter";

    std::fstream fs;
    fs.open(url); 

// Thanks to: https://gehrcke.de/2011/06/reading-files-in-c-using-ifstream-dealing-correctly-with-badbit-failbit-eofbit-and-perror/

std::string line;
std::string error_opening;
std::ifstream f (url);
if (!f.is_open())
	std::cerr << "Could not open file...\n";
	std::cerr << url << std::endl;
	std::cerr << strerror(errno) << std::endl;
	
while(getline(f, line)) {
    rt_printf("-%c", &line);
    }
    
if (f.bad())
    rt_printf("error reading ");
    fs.close();
*/	
    	
    	
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
           // TODO BELA MIDI.sendNoteOff(note, 0, 1);
           
           // Disable the note on all steps
           //Serial.println(String("DISABLE Note (for all steps) ") + note + String(" because ON velocity is ") + velocity );
           DisableNotes(note);

          // Now, when we release this note on the keyboard, the keyboard obviously generates a note off which gets stored in channel_a_midi_note_events
          // and can interfere with subsequent note ONs i.e. cause the note to end earlier than expected.
          // Since velocity of Note OFF is not respected by keyboard manufactuers, we need to find a way remove (or prevent?)
          // these Note OFF events. 
          // One way is to store them here for processing after the note OFF actually happens. 
          channel_a_ghost_events[note].is_active=1;
    
        } else {
          // We want the note on, so set it on.
          //Serial.println(String("Setting MIDI note ON for note ") + note + String(" when step is ") + step_count + String(" velocity is ") + velocity );
          // WRITE MIDI MIDI_DATA
          channel_a_midi_note_events[step_count][note][1].tick_count_in_sequence = loop_timing.tick_count_in_sequence; // Only one of these per step.
          channel_a_midi_note_events[step_count][note][1].velocity = velocity;
          channel_a_midi_note_events[step_count][note][1].is_active = 1;
          rt_printf("Done setting MIDI note ON for note %d when step is %d velocity is %d \n", note,  step_count, velocity );

        } 
      
          
        } else {
          
            // Note Off
             //Serial.println(String("Setting MIDI note OFF for note ") + note + String(" when step is ") + step_count );
             // WRITE MIDI MIDI_DATA
             channel_a_midi_note_events[step_count][note][0].tick_count_in_sequence = loop_timing.tick_count_in_sequence;
             channel_a_midi_note_events[step_count][note][0].velocity = velocity;
             channel_a_midi_note_events[step_count][note][0].is_active = 1;
             //Serial.println(String("Done setting MIDI note OFF for note ") + note + String(" when step is ") + step_count );

        	rt_printf("Done setting MIDI note OFF for note %d when step is %d \n", note,  step_count );
  }
  } 


void GateHigh(){
  //rt_printf("Gate HIGH at tick_count_since_start: %d ", loop_timing.tick_count_since_start);
  
  
  target_gate_out_state = true;
  envelope_1_audio.gate(true);
  envelope_2_analog.gate(true);
  

  

}

void GateLow(){
  //rt_printf("Gate LOW");
  
  target_gate_out_state = false;
  
  envelope_1_audio.gate(false);
  envelope_2_analog.gate(false);
  
  

  

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
	
  rt_printf("------SyncAndResetCv-----\n");	
  
  
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
    	rt_printf("----   -------   YES FIRST_STEP     -------    ------\n");
      SyncAndResetCv();
    } else {
      rt_printf("----       not first step      step_count is %d FIRST_STEP is %d                  ------\n", step_count, FIRST_STEP ); 
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

int my_note = 0;


float gSamplingPeriod = 0;
//int gSampleCount = 44100; // how often to send out a control change


float gPhase;
float gInverseSampleRate;
int gAudioFramesPerAnalogFrame = 0;






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

  
}



void OnTick(){
// Called on Every MIDI or Analogue clock pulse
// Drives sequencer activity.
// Can be called from Midi Clock and/or Digital Clock In.

 //rt_printf("Hello from OnTick \n");


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



void readMidiLoop(MidiChannelMessage message, void* arg){

	int MIDI_STATUS_OF_CLOCK = 7; // not  (decimal 248, hex 0xF8) ??

    // TODO Need to check its really On i.e. if velocity is zero its off...
	if(message.getType() == kmmNoteOn){
		if(message.getDataByte(1) > 0){
			uint8_t note = message.getDataByte(0);
			uint8_t velocity = message.getDataByte(1);
			uint8_t channel = message.getChannel();
			rt_printf("note ON: %d type: %d  \n", note, kmmNoteOn);
			
			// Write any note ON into the sequence
			OnMidiNoteInEvent(MIDI_NOTE_ON, note, velocity, channel);
			
		}
	} else if(message.getType() == kmmNoteOff){
		if(message.getDataByte(1) > 0){
			uint8_t note = message.getDataByte(0);
			uint8_t velocity = 0;
			uint8_t channel = message.getChannel();
			rt_printf("note OFF: %d  \n", note);
			
			// Write any note OFF into the sequence
			OnMidiNoteInEvent(MIDI_NOTE_OFF, note, velocity, channel);
			
		}
	} else if(message.getType() == MIDI_STATUS_OF_CLOCK) {
			// Midi clock  (decimal 248, hex 0xF8) - for some reason the library returns 7 for clock (kmmSystem ?)
		int type = message.getType();
		int byte0 = message.getDataByte(0);
		int byte1 = message.getDataByte(1);
    	

		//rt_printf("THINK I GOT MIDI CLOCK - type: %d byte0: %d  byte1 : %d \n", type, byte0, byte1);
		midi_clock_detected = 1;
		
		
		OnTick();

	}
	
	
   // float data = message.getDataByte(1) / 127.0f;

	bool shouldPrint = false;
	if(shouldPrint){
		message.prettyPrint();
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



// Each time we start the sequencer we want to start from the same conditions.
void InitSequencer(){
  GateLow();
  CvStop();
  loop_timing.tick_count_since_start = 0;
  ResetSequenceCounters();
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
    // rt_printf("binary_sequence has changed **");
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
        
        envelope_2_analog.setAttackRate(envelope_1_attack * analog_sample_rate);
        envelope_2_analog.setDecayRate(envelope_1_decay * analog_sample_rate);
        envelope_2_analog.setReleaseRate(envelope_1_release * analog_sample_rate);
        envelope_2_analog.setSustainLevel(envelope_1_sustain);

        // envelope_3.setAttackRate(envelope_1_attack * audio_sample_rate);
        // envelope_3.setDecayRate(envelope_1_decay * audio_sample_rate);
        // envelope_3.setReleaseRate(envelope_1_release * audio_sample_rate);
        // envelope_3.setSustainLevel(envelope_1_sustain);
        

	    frequency_2 = frequency_1 * 8.0;
	    frequency_3 = frequency_1 * 16.0;

		oscillator_1_audio.setFrequency(frequency_3); // higher freq
    	oscillator_2_analog.setFrequency(frequency_1); // lower freq

	
}


/*
//////// SequenceSettings (Everytime we get a midi clock pulse) ////////////////////////////
// This is called from the main loop() function on every Midi Clock message.
// It contains things that we want to check / happen every tick..
void SequenceSettings(){
  // Note we set tick_count_in_sequence to 0 following stop and start midi messages.
  // The midi clock standard sends 24 ticks per crochet. (quarter note).

 //int called_on_step = 0; // not currently used


  ////////////////////////////////////////////////////////////////
  // Read button state
  //int button_1_state = 1; // TODO digitalRead(euroshield_button_pin); // Pressed = LOW, Normal = HIGH
  //rt_printf("button_1_state is: ") + button_1_state);


  // state-change-3
  
  //int button_1_has_changed = 0; // TODO Button1HasChanged(button_1_state);
  //rt_printf("button_1_has_changed is: ") + button_1_has_changed);




  ////////////////////////////////////////////
  // Get the Pot positions. 
  // We will later assign the values dependant on the push button state
  //upper_input_raw = analogRead(upper_pot_pin);
  


  //lower_input_raw = analogRead(lower_pot_pin);
  //rt_printf("*****lower_input_raw *** is: %s ", lower_input_raw  );


//  if ((button_1_state == HIGH) & IsCrossing(sequence_pattern_input, upper_input_raw, FUZZINESS_AMOUNT)) {
   // sequence_pattern_input = static_cast<int>(round(map(sequence_pattern_input_raw, 0.0, 1.0, 0.0, 1023.0))); // GetValue(sequence_pattern_input_raw, sequence_pattern_input, jitter_reduction);
   // rt_printf("**** NEW value for sequence_pattern_input is: %d ", sequence_pattern_input  );
    
  //} else {
  //  rt_printf("NO new value for sequence_pattern_input . Sticking at: %s", sequence_pattern_input  );
  //}
  
  //if ((button_1_state == LOW) & IsCrossing(sequence_length_input, upper_input_raw, FUZZINESS_AMOUNT)) {   
   // sequence_length_input = GetValue(upper_input_raw, sequence_length_input, jitter_reduction);
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


//rt_printf("**** sequence_pattern_input is now: ") + sequence_pattern_input  ); 
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

 //  binary_sequence = (sequence_pattern_input & sequence_bits_8_through_1) + 1;

   // If we have 8 bits, use the range up to 255

   
   uint8_t binary_sequence_lower_limit = 1;  // Setting to 1 means we never get 0 i.e. a blank sequence especially when we change seq length
   // TODO Could probably use a smaller type 
   unsigned int binary_sequence_upper_limit; 


//binary_sequence_upper_limit = pow(current_sequence_length_in_steps, 2);

// REMEMBER, current_sequence_length_in_steps is ONE indexed (from 1 up to 16) 
// For a 3 step sequence we want to cover all the possibilities of a 3 step sequence which is (2^3) - 1 = 7
// i.e. all bits on of a 3 step sequence is 111 = 7 decimal 
// or (2^current_sequence_length_in_steps) - 1
binary_sequence_upper_limit = pow(2, current_sequence_length_in_steps) - 1; 

   //rt_printf("binary_sequence_upper_limit is: ") + binary_sequence_upper_limit  );
    


  // Button is in Normal state (not pressed) (HIGH) (button_1_state == HIGH)
   // ***UPPER Pot HIGH Button*** //////////
  // Generally the lowest value from the pot we get is 2 or 3 
  // setting-1
  binary_sequence_result = fscale( 1, 1023, binary_sequence_lower_limit, binary_sequence_upper_limit, sequence_pattern_input, 0);

   


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

    the_sequence = BitClear(the_sequence, current_sequence_length_in_steps -1); // current_sequence_length_in_steps is 1 based index. bitClear is zero based index.

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
 current_sequence_length_in_steps_raw = fscale( 15, 1023, 0, 15, sequence_length_input, 0);   ;
 // rt_printf("current_sequence_length_in_steps is: ") + current_sequence_length_in_steps  );
   
   //((sequence_length_input & current_sequence_length_in_steps_bits_8_7_6) >> 5) + 1; // We want a range 1 - 8
   

  // Highlight the first step 
  if (step_count == FIRST_STEP) {

    // If the sequence length is 8 (very predictable), make it shine!
    if (current_sequence_length_in_steps == 8){
      Led2Level(BRIGHT_5);
      //Led4Digital(true);
    } else {
      Led2Level(BRIGHT_2);
      //Led2Level(fscale( FIRST_STEP, current_sequence_length_in_steps, 0, BRIGHT_1, current_sequence_length_in_steps, 0));
      //Led4Digital(false);
    }
  
  } else {
      // Else off.
      Led2Level(BRIGHT_0);

  }

// continuous indication of length
    if (current_sequence_length_in_steps == 16){
      Led3Level(BRIGHT_2);     
    } else if (current_sequence_length_in_steps == 8){
      Led3Level(BRIGHT_5);
    } else if (current_sequence_length_in_steps == 4){
      Led3Level(BRIGHT_4);
    } else {
      Led3Level(BRIGHT_0);
    }


  // Led3Level(fscale( MIN_SEQUENCE_LENGTH_IN_STEPS, MAX_SEQUENCE_LENGTH_IN_STEPS, 0, BRIGHT_5, current_sequence_length_in_steps, -1.5));   
   
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


  //return called_on_step;
 
  } // End of SequenceSettings
////////////////////////////////////////////////

*/

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



















void MaybeOnTick(){
  if (do_tick == true){
    do_tick = false;
    OnTick();
  }
}





/////////////////////////////////////////////////////////

bool setup(BelaContext *context, void *userData){
	
	rt_printf("Hello from Setup: SimonSaysSeeq on Bela :-) \n");
	
	oscillator_1_audio.setup(context->audioSampleRate);
	oscillator_2_analog.setup(context->analogSampleRate);
	
	frequency_1 = 110; 
	oscillator_1_audio.setFrequency(frequency_1);
	

 
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
        //if((gFrequencyUpdateTask = Bela_createAuxiliaryTask(&recalculate_frequencies, 85, "bela-update-frequencies")) == 0)
        //        return false;
        
        

        audio_sample_rate = context->audioSampleRate;
        analog_sample_rate = context->analogSampleRate;


        // Set ADSR parameters
        envelope_1_audio.setAttackRate(envelope_1_attack * context->audioSampleRate);
        envelope_1_audio.setDecayRate(envelope_1_decay * context->audioSampleRate);
        envelope_1_audio.setReleaseRate(envelope_1_release * context->audioSampleRate);
        envelope_1_audio.setSustainLevel(envelope_1_sustain);
        
        
        envelope_2_analog.setAttackRate(envelope_1_attack * context->analogSampleRate);
        envelope_2_analog.setDecayRate(envelope_1_decay * context->analogSampleRate);
        envelope_2_analog.setReleaseRate(envelope_1_release * context->analogSampleRate);
        envelope_2_analog.setSustainLevel(envelope_1_sustain);

        // envelope_3.setAttackRate(envelope_1_attack * context->audioSampleRate);
        // envelope_3.setDecayRate(envelope_1_decay * context->audioSampleRate);
        // envelope_3.setReleaseRate(envelope_1_release * context->audioSampleRate);
        // envelope_3.setSustainLevel(envelope_1_sustain);



        // Set buttons pins as inputs
        pinMode(context, 0, gTriggerButtonPin, INPUT);
        pinMode(context, 0, gModeButtonPin, INPUT);




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
	
	
//	std::srand(static_cast<unsigned int>(std::time(nullptr)));

  
	    


    ///////////////////////////////////////////
   // Look for Analogue Clock (24 PPQ)

	// Use this global variable as a timing device
	// Set other time points from this frame_timer
	frame_timer = context->audioFramesElapsed;
	
	


    // AUDIO LOOP
	for(unsigned int n = 0; n < context->audioFrames; n++) {
		
		audio_envelope_1_amplitude  = 1.0 * envelope_1_audio.process();
		audio_osc_1_result = oscillator_1_audio.process() * audio_envelope_1_amplitude;
		
		
		
		for(unsigned int ch = 0; ch < gAudioChannelNum; ch++){
			// Pass input to output
			
			
			
			
			// todo create separate vars for oscillator_1_audio_output
			
			if (ch == 0){
				audioWrite(context, n, ch, audio_osc_1_result);
			} else { // ch 1
				audioWrite(context, n, ch, audioRead(context, n, ch));
			}
			
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
		analog_osc_2_result = oscillator_2_analog.process();
		
		// Process analog envelope
		analog_envelope_2_amplitude  = envelope_2_analog.process();  
		
		// Get an inverse envelope
		analog_envelope_3_amplitude  = 1 - analog_envelope_2_amplitude; // Inverse
		
		// Modulated output
		analog_out_2 = analog_osc_2_result * analog_envelope_2_amplitude;
		
		// Another modulated output
		analog_out_3 = analog_osc_2_result * analog_envelope_3_amplitude;
		
		// Additive output
		analog_out_4 = ( analog_osc_2_result + analog_envelope_3_amplitude ) / 2.0;
		
		
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
	      	
	      	frequency_1 = map(analogRead(context, n, OSC_FREQUENCY_INPUT_PIN), 0, 1, 1, 110);
 
	      	
	      	//envelope_1_attack = map(analogRead(context, n, OSC_FREQUENCY_INPUT_PIN), 0, 1, 0.001, 0.5);
	      	//envelope_1_decay = map(analogRead(context, n, OSC_FREQUENCY_INPUT_PIN), 0, 1, 0.5, 3.0);
	      	
		  	 // want range 0 to 3 seconds
		  	//envelope_2_attack = analogRead(context, n, OSC_FREQUENCY_INPUT_PIN);
		  }
		  
		  if (ch == ADSR_RELEASE_INPUT_PIN){
		  	
		  	envelope_1_release = map(analogRead(context, n, ADSR_RELEASE_INPUT_PIN), 0, 1, 0.01, 5.0);
		  	
		  //	lfo_b_frequency_input_raw = analogRead(context, n, ADSR_RELEASE_INPUT_PIN);
		  }
	      
	      
	      
	      // OUTPUTS
	      // CV 1 (SEQ GATE OUT)
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


	      // CV 3
	      if (ch == SEQUENCE_CV_OUTPUT_3_PIN){

	      	//rt_printf("amp is: %f", amp);
	      	analogWrite(context, n, ch, analog_out_3);
	      }
	      
	      // CV 4
	      if (ch == SEQUENCE_CV_OUTPUT_4_PIN){


	      	//rt_printf("amp is: %f", amp);
	      	analogWrite(context, n, ch, analog_out_4);
	      }
	      

		}
	}
	

	
	Bela_scheduleAuxiliaryTask(gChangeSequenceTask);
	
	
	// DIGITAL LOOP 
		// Looking at all frames in case the transition happens in these frames. However, as its a clock we could maybe look at only the first frame.
	    for(unsigned int m = 0; m < context->digitalFrames; m++) {
	    
        	// Next state
        	new_digital_clock_in_state = digitalRead(context, m, CLOCK_INPUT_DIGITAL_PIN);
        
        	// Only set new state if target is changed
        	if (target_gate_out_state != gate_out_state_set){
        		// 0 to 3.3V ? Salt docs says its 0 to 5 V (Eurorack trigger voltage is 0 - 5V)
	        	digitalWrite(context, m, SEQUENCE_OUT_PIN, target_gate_out_state);
	        	gate_out_state_set = target_gate_out_state;
        	}
        	
        	// Do similar for another PIN for if (step_count == FIRST_STEP)
        	


            // If detect a rising clock edge
            if ((new_digital_clock_in_state == HIGH) && (current_digital_clock_in_state == LOW)){
              
              current_digital_clock_in_state = HIGH;
              last_clock_rising_edge = frame_timer;
              
              if (sequence_is_running == LOW){
                StartSequencer();
              }
               
              OnTick();
              
            } 
            
            // If detect a Falling clock edge
            if ((new_digital_clock_in_state == LOW) && (current_digital_clock_in_state == HIGH)){
              current_digital_clock_in_state = LOW;
              last_clock_falling_edge = frame_timer;
            	
            	
				// the pulse width of our clock (half actually)
            	clock_width = last_clock_falling_edge - last_clock_rising_edge;
            	//rt_printf("clock_width is: %llu \n", clock_width);
            	
            	// currently a constant 
            	//clock_patience = clock_width * 100;
            } 
      }
      
     
      
      
      /////////////////////////////////////////////////////////////////////////////////
		// When relying on the analogue / digital (non midi) clock, we don't have a stop as such, so if we don't detect a clock for a while, then assume its stopped.
		// Note that the Beat Step Pro takes a while to kill its clock out after pressing the Stop button.
		if (midi_clock_detected == LOW){
    		elapsed_since_last_clock_rising_edge = frame_timer - last_clock_rising_edge;
			//rt_printf("elapsed_since_last_clock_rising_edge is: %d ", elapsed_since_last_clock_rising_edge);	
		
			if (elapsed_since_last_clock_rising_edge > clock_patience){
		  		if (sequence_is_running == HIGH) {
		    		rt_printf("Stopping sequencer because elapsed_since_last_clock_rising_edge: %llu is greater than clock_patience: %llu \n", elapsed_since_last_clock_rising_edge, clock_patience);
			    	StopSequencer();
				}
			}
		  
	
	
		// Temp code until we have clock
	   //temp_count++;
	   // if(temp_count % 1000 == 0) {
	   // 	OnTick();	
	   // }
	

        // Only look for this clock if we don't have midi.
   //     if (midi_clock_detected == LOW) {

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
              last_clock_rising_edge = milliseconds();
              
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
              last_clock_rising_edge = milliseconds();
              
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

