  /*

 ____  _  _      ____  _      ____  ____ ___  _ ____  ____  _____ _____ ____ 
/ ___\/ \/ \__/|/  _ \/ \  /|/ ___\/  _ \\  \/// ___\/ ___\/  __//  __//  _ \
|    \| || |\/||| / \|| |\ |||    \| / \| \  / |    \|    \|  \  |  \  | / \|
\___ || || |  ||| \_/|| | \||\___ || |-|| / /  \___ |\___ ||  /_ |  /_ | \_\|
\____/\_/\_/  \|\____/\_/  \|\____/\_/ \|/_/   \____/\____/\____\\____\\____\

SIMON SAYS SEEQ is released under the AGPL and (c) Simon Redfern 2020 - 2023

This sequencer is dedicated to all those folks working to fight climate change! Whilst you're here, check out https://feedbackloopsclimate.com/introduction/ 

This file uses BELA libraries and example code, see below.

An intro to what this does: https://www.twitch.tv/videos/885185134

*/

const char version[16]= "v0.39-BelaSalt";

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

// Note: Bela might run out of disk space
// du -h --max-depth=1
// #!/bin/bash
// dir=/path/to/directory/you/want/to/check
// num=[number of top largest directories to list)
// du -ah $dir | sort -n -r | head -n $num
// du -hs /var/*

// Can apparently delete /var/cache/apt


// Add this to your /etc/init.d

// root@bela:/etc/init.d# cat bela_startup.sh 
// #!/bin/bash
// rm -f /var/log/*.log || true
// echo > /var/log/syslog
// rm -f /var/log/*.gz || true
// echo > /var/log/syslog.1
// echo $(date -u) "I ran /etc/init.d/bela_startup.sh on startup" >> /var/log/bela_startup.log.keep


// echo > /var/log/*.log


// Make it executable with 
// chmod u+x bela_startup.sh
// chmod +x bela_startup.sh <- Need this else it doesn't run.

// add it to crontab (edited via, for example, crontab -e)
// not sure how successfully this runs
// @reboot sleep 60 && /etc/init.d/bela_startup.sh



//root@bela:/var/log# rm /var/log/*.log
//root@bela:/var/log# rm /var/log/syslog
//root@bela:/var/log# rm /var/log/*.gz
//root@bela:/var/log# rm /var/log/syslog.1 






#include <Bela.h>
#include <libraries/Midi/Midi.h>
#include <stdlib.h>
#include <cmath>

#include <libraries/Scope/Scope.h>

#include <libraries/ADSR/ADSR.h> // See https://www.earlevel.com/main/2013/06/03/envelope-generators-adsr-code/





#include <chrono>

#include <iostream>
#include <fstream>

#include <cstdlib>

#include <libraries/UdpClient/UdpClient.h> 


//#include <curlpp/cURLpp.hpp>
//#include <curlpp/Easy.hpp>
//#include <curlpp/Options.hpp>


//using namespace curlpp::options;


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

// NOTE: It seems there is a timing / loading issue with the USB midi port...
// In order for USB host midi port to work, either: 
// 1) save two copies of this patch under loop_A and loop_B, set the settings to start with loop_* and after booting the Salt, press the left button on Salt for more than two seconds 
// or 
// 2) Save this patch via the IDE after the USB cables are plugged in.
  

//const char* gMidiPort0 = "hw:0,0"; // This is the computer via USB cable
const char* gMidiPort0 = "hw:1,0,0"; // This is the first external USB Midi device. Keyboard is connected to the USB host port on Bela / Salt+
//const char* gMidiPort0 = "hw:0,0,0"; // try me

// Let our delay have 50 different course settings (so the pot doesn't jitter)
const unsigned int MAX_COARSE_DELAY_TIME_INPUT = 50;

// Divde clock - maybe just for midi maybe for sequence too
//const uint8_t MIN_midi_lane_input = 0;
//const uint8_t MAX_midi_lane_input = 16;


uint64_t last_function = 0; // set this in a function to a number

// This is our global frame timer. We count total elapsed frames with this.
uint64_t frame_timer = 0;

uint64_t last_clock_falling_edge = 0; 

uint64_t last_quarter_note_frame = 0;
uint64_t previous_quarter_note_frame = 0;

uint64_t last_sequence_reset_frame = 0;
uint64_t previous_sequence_reset_frame = 0;

uint64_t frames_per_sequence = 0;


uint64_t last_tick_frame = 0;

uint64_t elapsed_frames_since_last_tick = 0;

uint64_t elapsed_frames_since_all_notes_off = 0;
uint64_t last_notes_off_frame = 0;


float detected_bpm = 120.0;

float env3_amp = 0.7f;



uint64_t INITIAL_FRAMES_PER_24_TICKS = 22064;
uint64_t frames_per_24_ticks = 22064; // This must never be zero else we can divide by zero errors

int audio_sample_rate;
int analog_sample_rate;

int draw_buf_write_pointer = 0;

// Amount of delay in samples (needs to be smaller than or equal to the buffer size defined above)
int coarse_delay_frames = 22050;
int total_delay_frames = 22050;

int draw_total_frames = 22050;

// Amount of feedback
float delay_feedback_amount = 0.999; 

float draw_delay_feedback_amount = 0.999; 


uint8_t midi_lane_input = 0; // normal
uint8_t clock_divider_input_value = 1;

#include <math.h> //sinf
#include <time.h> //time
#include <libraries/Oscillator/Oscillator.h>
#include <libraries/OscillatorBank/OscillatorBank.h>


#define DELAY_BUFFER_SIZE 6400000
#define DRAW_BUFFER_SIZE 6400000

const float kMinimumFrequency = 20.0f;
const float kMaximumFrequency = 8000.0f;
int gSampleCount;               // Sample counter for indicating when to update frequencies
float gNewMinFrequency;
float gNewMaxFrequency;
// Task for handling the update of the frequencies using the analog inputs
AuxiliaryTask gFrequencyUpdateTask;

AuxiliaryTask gChangeSequenceTask;

AuxiliaryTask gPrintStatus;

AuxiliaryTask gAllNotesOff;

AuxiliaryTask gWriteSequenceToFiles;

AuxiliaryTask gSendUdpMessage;

AuxiliaryTask gInitMidiSequenceForce;

AuxiliaryTask gInitMidiSequenceNoForce;

// These settings are carried over from main.cpp
// Setting global variables is an alternative approach
// to passing a structure to userData in set up()
int gNumOscillators = 2; // was 500
int gWavetableLength = 1024;
void recalculate_frequencies(void*);
OscillatorBank osc_bank;

//Oscillator oscillator_2_audio;
Oscillator lfo_a_analog;
Oscillator lfo_b_analog;


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

// Salt +

int button_3_PIN = 1; 
int new_button_3_state = 0; 
int old_button_3_state = 0; 


int button_4_PIN = 3;
int new_button_4_state = 0; 
int old_button_4_state = 0; 



/////////////////


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


      int remotePort0 =7000;						// remote IP port
      const char* remoteIP = "192.168.3.1";	    // remote IP, where data will be published


      // Causes: Error while sending to pipe from WSClient_scope_data: (12) Cannot allocate memory (size: 25792)

UdpClient* myUdpClient0; 
      
   //   UdpClient *myUdpClient0 = new UdpClient(remotePort0,remoteIP);



// recursive function to print binary representation of a positive integer
void print_binary(unsigned int number)
{
    if (number >> 1) {
        print_binary(number >> 1);
    }
    rt_printf("%c" , (number & 1) ? '1' : '0' );
}







// Salt Pinouts salt pinouts are here: https://github.com/BelaPlatform/Bela/wiki/Salt
// e.g. CV I/O 1-8	analog channel 0-7	dac~/adc~ 3-10

// T2 (Trigger 2) is Physical Channel / Pin 14 

// T1 in is	digital channel 15
const int CLOCK_INPUT_DIGITAL_PIN = 15;
const int RESET_A_INPUT_DIGITAL_PIN = 14; //T2/SWITCH2 in	digital channel 14
const int RESET_B_INPUT_DIGITAL_PIN = 17; // TODO CONFIRM PIN

////////////////////////////////
// Digital Outputs
// T1 out	digital channel 0
// T2 out	digital channel 5
const int SEQUENCE_A_DIGITAL_OUT_PIN = 0;
const int SEQUENCE_B_DIGITAL_OUT_PIN = 5; // TODO check




// TODO Add a reset (on a digital pin?)

// CV I/O 1-8	ANALOG channel 0-7
const int SEQUENCE_A_PATTERN_ANALOG_INPUT_PIN = 0; // CV 1 input
const int SEQUENCE_A_LENGTH_ANALOG_INPUT_PIN = 2; // CV 3 input

const int LFO_OSC_1_FREQUENCY_INPUT_PIN = 1; // CV 2 input
const int LFO_OSC_2_FREQUENCY_INPUT_PIN = 3; // CV 4 input


const int SEQUENCE_B_PATTERN_ANALOG_INPUT_PIN = 4; // CV 5 (SALT+) was COARSE-_DELAY_TIME_INPUT_PIN
const int SEQUENCE_B_LENGTH_ANALOG_INPUT_PIN = 6; // CV 6 (SALT+)


const uint8_t MIDI_LANE_INPUT_PIN = 5; // CV 7 (SALT+) 
const uint8_t CLOCK_DIVIDER_INPUT_PIN = 7; // CV 8 (SALT+)


bool need_to_reset_draw_buf_pointer = false;


const int SEQUENCE_A_GATE_OUTPUT_1_PIN = 0; // CV (GATE) 1 output
const int SEQUENCE_CV_OUTPUT_2_PIN = 1; // CV 2 output
const int SEQUENCE_CV_OUTPUT_3_PIN = 2; // CV 3 output
const int SEQUENCE_CV_OUTPUT_4_PIN = 3; // CV 4 output


const int SEQUENCE_B_GATE_OUTPUT_5_PIN = 4; // CV (GATE) 5 output
const int SEQUENCE_CV_OUTPUT_6_PIN = 5; // CV 6 output
const int SEQUENCE_CV_OUTPUT_7_PIN = 6; // CV 7 output
const int SEQUENCE_CV_OUTPUT_8_PIN = 7; // CV 8 output



////////////////////////////////////////////////


// const uint8_t BRIGHT_0 = 0;
// const uint8_t BRIGHT_1 = 10;
// const uint8_t BRIGHT_2 = 20;
// const uint8_t BRIGHT_3 = 75;
// const uint8_t BRIGHT_4 = 100;
// const uint8_t BRIGHT_5 = 255;

// Use zero based index for sequencer. i.e. step_a_count for the first step is 0.
const uint8_t FIRST_STEP = 0;
const uint8_t MAX_STEP = 15;

const uint8_t FIRST_BAR = 0;
const uint8_t MAX_BAR = 7; // Memory User!

const uint8_t MIN_LANE = 0;
const uint8_t MAX_LANE = 7; // Memory User!

const uint8_t MIN_CLOCK_DIVIDER_SETTING = 1;
const uint8_t MAX_CLOCK_DIVIDER_SETTING = 8;



uint8_t previous_lane = 0;
uint8_t current_midi_lane = 0;

uint8_t SILENT_MIDI_LANE = 0;



const uint8_t MIN_SEQUENCE_LENGTH_IN_STEPS = 4; // ONE INDEXED
const uint8_t MAX_SEQUENCE_LENGTH_IN_STEPS = 16; // ONE INDEXED

// Sequence Length (and default)
uint8_t current_sequence_a_length_in_steps = 8; 
uint8_t current_sequence_b_length_in_steps = 8; 

///////////////////////

//const int CV_WAVEFORM_B_FREQUENCY_RAW_MAX_INPUT = 1023;


const uint8_t MIDI_NOTE_ON = 1;
const uint8_t MIDI_NOTE_OFF = 0;


// Receive on midi channel 1 (0 internally)
int midi_receive_channel = 0; // Midi channels are between 0 to 15. (Humans call this 1 - 16). So this is channel 1.

uint8_t lowest_midi_note = 0;  // So we can filter out notes by note number
uint8_t highest_midi_note = 127;


uint8_t midi_channel_x = 0; // This is zero indexed. 0 will send on midi channel 1!
uint8_t last_note_on = 0;
uint8_t last_note_off = 0;
uint8_t last_note_disabled = 0;


bool init_midi_sequence_has_run = false;

bool need_to_auto_save_sequence = false;



float sequence_a_length_input_raw;
float sequence_a_pattern_input_raw;
unsigned int sequence_a_pattern_input = 20;
unsigned int sequence_a_pattern_input_last;
unsigned int sequence_a_pattern_input_at_button_change;



float sequence_b_length_input_raw;
float sequence_b_pattern_input_raw;
unsigned int sequence_b_pattern_input = 20;
unsigned int sequence_b_pattern_input_last;
unsigned int sequence_b_pattern_input_at_button_change;



float lfo_a_frequency_input_raw;
float lfo_osc_1_frequency;
float lfo_osc_2_frequency;
float frequency_2;
//float audio_osc_2_frequency;


unsigned int lfo_a_frequency_input = 20;
unsigned int lfo_a_frequency_input_last;
unsigned int lfo_a_frequency_input_at_button_change;




float lfo_b_frequency_input_raw;
unsigned int lfo_b_frequency_input = 20;
unsigned int lfo_b_frequency_input_last;
unsigned int lfo_b_frequency_input_at_button_change;


int new_digital_clock_in_state;
int current_digital_clock_in_state;

int new_reset_a_in_state;
int current_reset_a_in_state;

int new_reset_b_in_state;
int current_reset_b_in_state;


float analog_clock_in_level; // unused?
float right_peak_level;

float external_modulator_object_level;


float audio_left_input_raw;
float audio_right_input_raw;

unsigned int coarse_delay_input = 1;

// Musical parameters that the user can tweak.

// The Primary GATE sequence pattern // Needs to be upto 16 bits. Maybe more later.
unsigned int binary_sequence_a_result;
unsigned int gray_code_sequence_a;
unsigned int the_sequence_a;
unsigned int last_binary_sequence_a_result; // So we can detect changes

unsigned int binary_sequence_b_result;
unsigned int gray_code_sequence_b;
unsigned int the_sequence_b;
unsigned int last_binary_sequence_b_result; // So we can detect changes




bool do_tick = true;

bool do_envelope_1_on = false;

bool target_analog_gate_a_out_state = false;
bool current_analog_gate_a_out_state = false;

bool target_digital_gate_a_out_state = false;
bool current_digital_gate_a_out_state = false;

bool target_analog_gate_b_out_state = false;
bool current_analog_gate_b_out_state = false;

bool target_digital_gate_b_out_state = false;
bool current_digital_gate_b_out_state = false;



uint8_t target_led_1_tri_state = 0;
uint8_t target_led_2_tri_state = 0;
uint8_t target_led_3_tri_state = 0;
uint8_t target_led_4_tri_state = 0;


// Used to control when/how we change sequence length 
uint8_t new_sequence_a_length_in_ticks; 
uint8_t new_sequence_b_length_in_ticks; 


// Just counts 0 to 5 within each step
uint8_t ticks_after_step_a;
uint8_t ticks_after_step_b;


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

int last_flash_change = 0;
float flash_interval = 44000;


uint8_t TriStateToggle(uint8_t tri_state_input){
  uint8_t tri_state_output = 0;


        if (tri_state_input == 0){
          tri_state_output = 1;
        } else if (tri_state_input == 1){
          tri_state_output = 2;
        } else if (tri_state_input == 2){
          tri_state_output = 0;
        } 
        return tri_state_output;
}


void FlashHello(){
	last_function = 238469;
	
// This gets called in the digital loop
// Say hello when this patch starts up.

// What ever happens, we don't want to change the led target state for two long. After 10 seconds we should be done.
if(frame_timer < 440000) {
	
	// When we should next change state
	int next_flash_change = last_flash_change + flash_interval;

	// No point in flashing if the interval is too small
	if (flash_interval > 100){
		if (frame_timer >= next_flash_change){

        target_led_1_tri_state = TriStateToggle(target_led_1_tri_state);
        target_led_2_tri_state = TriStateToggle(target_led_2_tri_state);
        target_led_3_tri_state = TriStateToggle(target_led_3_tri_state);
        target_led_4_tri_state = TriStateToggle(target_led_4_tri_state);

			last_flash_change = frame_timer;
			flash_interval = flash_interval / 1.5;
			//rt_printf("At frame_timer %llu I'm setting last_flash_change to %d and flash_interval to %f  \n" , frame_timer, last_flash_change, flash_interval );
		} 
	// Once we're done..
	} else {
		// In the end we want the led to be off (until something else changes the state)
      target_led_1_tri_state = 0;
      target_led_2_tri_state = 0;
      target_led_3_tri_state = 0;
      target_led_4_tri_state = 0;
	}
  }
}

////////////////////////////////////
// Store extra data about the note (velocity, "exactly" when in a step etc)
// Note name (number) and step information is stored in the array below.         
class NoteInfo
{
 public:
   uint8_t velocity = 0 ;
   uint8_t tick_count_since_step = 0; 
   uint8_t is_active = 0;
   uint8_t tick_count_since_start = 0;
};
/////////


uint8_t test_velocity = 7 ;

int test_int = 8 ;


////////
// For each sequence step / midi note number  / on-or-off we store a NoteInfo (which defines a bit more info)
// Arrays are ZERO INDEXED but here we define the SIZE of each DIMENSION of the Array.)
// This way we can easily access a step and the notes there.
// [step][midi_note][on-or-off]
// [step] will store a digit between 0 and 15 to represent the step of the sequence.
// [midi_note] will store between 0 and 127
// [on-or-off] will store either 1 for MIDI_NOTE_ON or 0 for MIDI_NOTE_OFF
//NoteInfo channel_x_midi_note_events[MAX_STEP+1][128][2]; 

NoteInfo channel_x_midi_note_events[MAX_LANE+1][MAX_BAR+1][MAX_STEP+1][128][2]; 
////////


// uint8_t data[32];
// // fill in the data...
// write_to_file(data, sizeof(data));

// void write_to_file(uint8_t *ptr, size_t len) {
//     ofstream fp;
//     fp.open("somefile.bin",ios::out | ios :: binary );
//     fp.write((char*)ptr, len);
// }


// std::String converter(uint8_t *str){
//     return std::String((char *)str);
// }





void SyncSequenceToFile(bool write_to_file){
	
	last_function = 6123819;
	
	
	rt_printf("Hello from SyncSequenceToFile write_to_file is %d \n", write_to_file);
	
    // Create directory. 
    // Note this directory MUST be *OUT* of the project path, else Node.JS process "node" (which runs the Bela IDE) will grind to a halt. (even if files are marked as hidden node will spend CPU time on them)
 
 
    if (write_to_file == true){
		const int mkdir= system("mkdir -p /var/SimonSaysSeeqConfig");
		// hm what if we get -bash: /bin/rm: Argument list too long ?
		const int remove= system("rm /var/SimonSaysSeeqConfig/_NoteInfo*");
    }
	
	
 int count_of_file_operations = 0;

	uint8_t ln = 0;
	uint8_t sc = 0;
	uint8_t bc = 0;
	uint8_t n = 0;
    uint8_t onoff = 0;
   

	std::string file_name;

	std::ofstream output_file;



	std::string line;
			  

			  
	std::string got;
			 
	long long_integer_from_file;
			  
	std::string::size_type sz;   // alias of size_t			  

for (ln = MIN_LANE; ln <= MAX_LANE; ln++){
   for (bc = FIRST_BAR; bc <= MAX_BAR; bc++){
	    for (sc = FIRST_STEP; sc <= MAX_STEP; sc++){
		      for (n = 0; n <= 127; n++){	
				for (onoff = 0; onoff <= 1; onoff++){

          // TODO optimize loading of files e.g. only load the files that exist
					//rt_printf(".");

				  file_name = "/var/SimonSaysSeeqConfig/_NoteInfo_lane_" + std::to_string(ln) + "_bar_" + std::to_string(bc) + "_step_" + std::to_string(sc) + "_note_" + std::to_string(n) + "_onoff_" + std::to_string(onoff);
		    	
		    
		        if (write_to_file == true) {
		        	// WRITE Sequence to Files
		        	//rt_printf("SyncSequenceToFile will WRITE sequence to Files \n");
		        	

					// Only write active notes. (we deleted all files above so should be ok. )
					if (channel_x_midi_note_events[ln][bc][sc][n][onoff].is_active == 1){
						
						// Open for writing in truncate mode (we will replace the contents)
						output_file.open (file_name, std::ios::out | std::ios::trunc | std::ios::binary);

            			count_of_file_operations = count_of_file_operations + 1;
						
						//rt_printf("SyncSequenceToFile Write Non Zero value is_active: %d \n", channel_x_midi_note_events[ln][bc][sc][n][onoff].is_active);
						//rt_printf("SyncSequenceToFile Write value velocity: %d \n", channel_x_midi_note_events[ln][bc][sc][n][onoff].velocity);
						
						////rt_printf("SyncSequenceToFile Hope to write the value: %d \n", channel_x_midi_note_events[ln][bc][sc][n][1].velocity);
					
						// Must cast the value to int else it won't be written to the file.
						output_file << (int) channel_x_midi_note_events[ln][bc][sc][n][onoff].velocity  << std::endl;
						output_file << (int) channel_x_midi_note_events[ln][bc][sc][n][onoff].tick_count_since_step  << std::endl;
						output_file << (int) channel_x_midi_note_events[ln][bc][sc][n][onoff].is_active << std::endl;

					
						output_file.close();
						
					}
					

					
		        } else {
		        	// READ Sequence from Files
		        	// rt_printf("SyncSequenceToFile will READ sequence from Files \n");
					  std::ifstream read_file(file_name);

            count_of_file_operations = count_of_file_operations + 1;
		
					  if (read_file.is_open()) {
						
						// only want one line
						// Velocity
						getline (read_file, line);	
					
					    try {
					      	// Convert the value to integer
							long_integer_from_file = std::stol (line, &sz);
							if (long_integer_from_file != 0){
								rt_printf("Got Non Zero long_integer_from_file: %d \n", long_integer_from_file);
							}
						} catch(...) {
							long_integer_from_file = 0;
							rt_printf("Exception getting value from file so setting long_integer_from_file to zero \n");
						}
					      
					    // Must cast int to uint8_t
					    channel_x_midi_note_events[ln][bc][sc][n][onoff].velocity = (uint8_t) long_integer_from_file;
					    
					    
					    if (long_integer_from_file != 0){
								rt_printf("Should have set channel_x_midi_note_events[bc][sc][n][onoff].velocity Did I? : %d \n", channel_x_midi_note_events[ln][bc][sc][n][onoff].velocity);
							}
							
						////////////
						// Ticks after Step
						getline (read_file, line);	
					
					    try {
					      	// Convert the value to integer
							long_integer_from_file = std::stol (line, &sz);
							if (long_integer_from_file != 0){
								rt_printf("Got Non Zero long_integer_from_file: %d \n", long_integer_from_file);
							}
						} catch(...) {
							long_integer_from_file = 0;
							rt_printf("Exception getting value from file so setting long_integer_from_file to zero \n");
						}
					      
					    // Must cast int to uint8_t
					    channel_x_midi_note_events[ln][bc][sc][n][onoff].tick_count_since_step = (uint8_t) long_integer_from_file;
					    
					    
					    if (long_integer_from_file != 0){
								rt_printf("Should have set channel_x_midi_note_events[ln][bc][sc][n][onoff].tick_count_since_step Did I? : %d \n", channel_x_midi_note_events[ln][bc][sc][n][onoff].tick_count_since_step);
							}
					    
					    
					    
					    //////////////////////////
					    // Is Active
					    getline (read_file, line);
					    try {
					      	// Convert the value to integer
							long_integer_from_file = std::stol (line, &sz);
							//rt_printf("Got long_integer_from_file: %d \n", long_integer_from_file);
						} catch(...) {
							long_integer_from_file = 0;
							//rt_printf("No value found in file, setting long_integer_from_file to zero \n");
						}
					      
					    // Must cast int to uint8_t
					    channel_x_midi_note_events[ln][bc][sc][n][onoff].is_active = (uint8_t) long_integer_from_file;
					    
					    
			
					    read_file.close();
					  } else {
					  	// This is a common occurance since we only create files for is_active notes, if we don't find a file the Note will stay in its inactive state.
					  	// rt_printf("Unable to open file \n"); 
					  }		
				
		        } // end read/write test
				/////////////////////////////////////

		
				}
		      }
	    }
   }
}
           
    rt_printf("Bye from SyncSequenceToFile. There were %d file operations and write_to_file was %d \n", count_of_file_operations, write_to_file);       
}

///////





void WriteSequenceToFiles(void*){
	
	last_function = 293924;
	
	//if (need_to_auto_save_sequence == true){
		SyncSequenceToFile(true);
	//	need_to_auto_save_sequence = false;
	//}
}

void ReadSequenceFromFiles(){
	last_function = 298897689;
	
	SyncSequenceToFile(false);
}



///////

//void SaveNoteInfos() {
	
	// possible alternative approach using Bela WriteFile

	    // file2.setup("simon_says_test_out.m"); //set the file name to write to
     //   file2.setHeader("myvar=[\n"); //set one or more lines to be printed at the beginning of the file
     //   file2.setFooter("];\n"); //set one or more lines to be printed at the end of the file
     //   file2.setFormat("%.4f %.4f\n"); // set the format that you want to use for your output. Please use %f only (with modifiers)
     //   file2.setFileType(kBinary);
     //   file2.setEchoInterval(1); // only print to the console 1 line every other 10000
        
       
//}



// "Ghost notes" are created to cancel out a note-off in channel_x_midi_note_events that is created  during the note off of low velocity notes.
// class GhostNote
// {
//  public:
//    uint8_t tick_count_in_sequence = 0;
//    uint8_t is_active = 0;
// };

//GhostNote channel_x_ghost_events[128];

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
struct Timing loop_timing_a;
struct Timing loop_timing_b;

// Count of the main pulse i.e. sixteenth notes or eigth notes 
uint8_t step_a_count; // write
uint8_t step_a_play; // read

uint8_t step_b_count; // write
uint8_t step_b_play; // read

// Count of the bar / measure.
uint8_t bar_a_count; // for wrting
uint8_t bar_a_play; // for reading

uint8_t bar_b_count; // for wrting
uint8_t bar_b_play; // for reading


void SetPlayAFromCount(){
	last_function = 6798;
	
	bar_a_play = bar_a_count;
	step_a_play = step_a_count;
}

void SetPlayBFromCount(){
	
	last_function = 79;
	
	bar_b_play = bar_b_count;
	step_b_play = step_b_count;
}


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


void SetTickCountInSequenceA(uint8_t value){
  loop_timing_a.tick_count_in_sequence = value;
  loop_timing_a.tick_count_since_step = value % 6;
}

void SetTickCountInSequenceB(uint8_t value){
  loop_timing_b.tick_count_in_sequence = value;
  loop_timing_b.tick_count_since_step = value % 6;
}

void SetTotalTickCountA(int value){
  loop_timing_a.tick_count_since_start = value;
}

void SetTotalTickCountB(int value){
  loop_timing_b.tick_count_since_start = value;
}


void Beginning(){
	last_function = 2119892;
	
  SetTickCountInSequenceA(0);
  SetTickCountInSequenceB(0);
  
  step_a_count = FIRST_STEP;
  bar_a_count = FIRST_BAR;
  step_a_play = FIRST_STEP;
  bar_a_play = FIRST_BAR;
  
  step_b_count = FIRST_STEP;
  bar_b_count = FIRST_BAR;
  step_b_play = FIRST_STEP;
  bar_b_play = FIRST_BAR;
}


uint8_t IncrementOrResetBarCountA(){
	
	last_function = 239872;

  // Every time we call this function we advance or reset the bar
  if (bar_a_count == MAX_BAR){
    bar_a_count = FIRST_BAR;
  } else {
    bar_a_count = BarCountSanity(bar_a_count + 1);
  }
  
  //rt_printf("** IncrementOrResetBarCountA bar_a_count is now: %d \n", bar_a_count);
  return BarCountSanity(bar_a_count);
}

uint8_t IncrementOrResetBarCountB(){
	
	last_function = 23962;

  // Every time we call this function we advance or reset the bar
  if (bar_b_count == MAX_BAR){
    bar_b_count = FIRST_BAR;
  } else {
    bar_b_count = BarCountSanity(bar_b_count + 1);
  }
  
  return BarCountSanity(bar_b_count);
}





uint8_t StepCountSanity(uint8_t step_count_){
	last_function = 672349;
	
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

ADSR per_sequence_adsr_a; 
ADSR per_sequence_adsr_b;
ADSR per_sequence_adsr_c;

float analog_per_sequence_adsr_a_level = 0;
float analog_per_sequence_adsr_b_level = 0;
float analog_per_sequence_adsr_c_level = 0;


float envelope_1_attack = 0.0001; // per_sequence_adsr_a attack (seconds)
float envelope_1_decay = 0.1; // envelope_1 decay (seconds)
float envelope_1_sustain = 0.1; // envelope_1 sustain level
float envelope_1_release = 0.5; 
// A setting that's used to set various attack / decay
float envelope_1_setting = 0.5; // INITIAL value only This is set by a pot. (seconds)


float analog_out_1;
float analog_out_2;
float analog_out_3;
float analog_out_4;
float analog_out_5;
float analog_out_6;
float analog_out_7;
float analog_out_8;



float audio_osc_1_result;

float lfo_a_result_analog;
float lfo_b_result_analog;

float analog_osc_3_result;

float envelope_2_attack = 0.0001; // envelope_2 attack (seconds)
float envelope_2_decay = 0.25; // envelope_2 decay (seconds)
float envelope_2_sustain = 0.9; // envelope_2 sustain level
float envelope_2_release = 0.5; // envelope_2 release (seconds)





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






// This relates to the whole machine.
void ResetSequenceACounters(){
	
	last_function = 3452978;
	
  SetTickCountInSequenceA(0);

  IncrementOrResetBarCountA();

  step_a_count = FIRST_STEP;

  SetPlayAFromCount(); // Note we are not resetting "bar" (not using?)

  lfo_a_analog.setPhase(0.0);
  lfo_b_analog.setPhase(0.0);
  
  per_sequence_adsr_a.reset();
  per_sequence_adsr_a.gate(true);
 
  per_sequence_adsr_b.reset(); 
  per_sequence_adsr_b.gate(true);
  
  per_sequence_adsr_c.reset();
  per_sequence_adsr_c.gate(true);

  previous_sequence_reset_frame = last_sequence_reset_frame; // The last time the sequence A was reset
  last_sequence_reset_frame = frame_timer; // Now
  
  
  // We'll be able to use this, to set delay in frames
  frames_per_sequence = last_sequence_reset_frame - previous_sequence_reset_frame;

 
  
  need_to_reset_draw_buf_pointer = true;

  // target_led_2_tri_state = 1;

  //rt_printf("******************** ResetSequenceACounters Done. current_sequence_a_length_in_steps is: %d step_a_count is now: %d \n", current_sequence_a_length_in_steps, step_a_count);
}


void ResetSequenceBCounters(){
	
	last_function = 766799;
	

  SetTickCountInSequenceB(0);
  

  IncrementOrResetBarCountB();
  

  step_b_count = FIRST_STEP; 

  SetPlayBFromCount(); // Note we are not resetting "bar" (not using?)
  
   //rt_printf("ResetSequenceBCounters Done. current_sequence_b_length_in_steps is: %d step_b_count is now: %d \n", current_sequence_b_length_in_steps, step_b_count);
}







/// end for ADSR





void printStatus(void*){
  // This is used to print periodically. 

    // Might not want to print every time else we overload the CPU
    gCount++;
	
    // By setting this mod we can choose to print less frequently.  
    if(gCount % 1000 == 0) {
      
		rt_printf("======== Hello from printStatus. gCount is: %d ========= \n",gCount);
		
		
		rt_printf("last_function is: %llu \n", last_function);


// last_function_32

		  // Global frame timing

		//rt_printf("frame_timer is: %llu \n", frame_timer);
    //	rt_printf("frames_per_24_ticks is: %llu \n", frames_per_24_ticks);
    // 	rt_printf("detected_bpm is: %f \n", detected_bpm);
    //	rt_printf("frames_per_sequence is: %llu \n", frames_per_sequence);
    
		// Delay Time
		
		//rt_printf("DELAY_BUFFER_SIZE is: %d \n", DELAY_BUFFER_SIZE);





		//rt_printf("coarse_delay_input is: %d \n", coarse_delay_input);		
		//rt_printf("coarse_delay_frames is: %d \n", coarse_delay_frames);
		//rt_printf("fine_delay_frames_delta is: %d \n", fine_delay_frames_delta);
		//rt_printf("fine_delay_frames is: %d \n", fine_delay_frames);
		//rt_printf("total_delay_frames is: %d \n", total_delay_frames);		
		
		
		// Delay Feedback
		//rt_printf("feedback_delta is: %f \n", feedback_delta);

	    
		//rt_printf("delay_feedback_amount is: %f \n", delay_feedback_amount);
		
		
		// Analog / Digital Clock In.
  	//	rt_printf("last_quarter_note_frame is: %llu \n", last_quarter_note_frame);

		// Other Inputs
		//rt_printf("sequence_a_pattern_input is: %d \n", sequence_a_pattern_input);
		//rt_printf("sequence_a_length_input_raw is: %f \n", sequence_a_length_input_raw);
		
		
		// rt_printf("new_button_1_state is: %d \n", new_button_1_state);
		//rt_printf("new_button_2_state is: %d \n", new_button_2_state);
	//	rt_printf("new_button_3_state is: %d \n", new_button_3_state);
		// rt_printf("new_button_4_state is: %d \n", new_button_4_state);
		

		// rt_printf("do_button_1_action is: %d \n", do_button_1_action);
		// rt_printf("do_button_2_action is: %d \n", do_button_2_action);
		// rt_printf("do_button_3_action is: %d \n", do_button_3_action);
		// rt_printf("do_button_4_action is: %d \n", do_button_4_action);
		
    //rt_printf("\n==== MIDI ======= \n");

	  //rt_printf("current_midi_lane is: %d \n", current_midi_lane);

		
        // Sequence derived results 
        

    	
    	//rt_printf("new_sequence_a_length_in_ticks is: %d \n", new_sequence_a_length_in_ticks);
    	


		/*
    	rt_printf("envelope_1_attack is: %f \n", envelope_1_attack);
    	rt_printf("envelope_1_decay is: %f \n", envelope_1_decay);
	
    	*/
    	
    	//rt_printf("\n==== Sequence A ======= \n");
    	
    	//rt_printf("sequence_a_length_input_raw is: %f \n", sequence_a_length_input_raw);
    	//rt_printf("sequence_a_pattern_input_raw is: %f \n", sequence_a_pattern_input_raw);

    	//rt_printf("current_sequence_a_length_in_steps is: %d \n", current_sequence_a_length_in_steps);
    	//rt_printf("new_sequence_a_length_in_ticks is: %d \n", new_sequence_a_length_in_ticks);
    	
    	
    	//rt_printf("the_sequence_a is: %d \n", the_sequence_a);
	  //  print_binary(the_sequence_a);
		// rt_printf("%c \n", 'B');


    //	rt_printf("loop_timing_a.tick_count_in_sequence is: %d \n", loop_timing_a.tick_count_in_sequence);
    //	rt_printf("loop_timing_a.tick_count_since_start is: %d \n", loop_timing_a.tick_count_since_start);
    	
    	rt_printf("step_a_count is: %d \n", step_a_count);
    	rt_printf("clock_divider_input_value is: %d \n", clock_divider_input_value);


    	/*
    	if (step_a_count == FIRST_STEP) {
    		rt_printf("FIRST_STEP A \n");
    	} else {
    		rt_printf("other step A \n");
    	}
		*/

		// rt_printf("\n==== Sequence B ======= \n");

    	//rt_printf("sequence_b_length_input_raw is: %f \n", sequence_b_length_input_raw);
    	//rt_printf("sequence_b_pattern_input_raw is: %f \n", sequence_b_pattern_input_raw);
    	
		//rt_printf("current_sequence_b_length_in_steps is: %d \n", current_sequence_b_length_in_steps);
		//rt_printf("new_sequence_b_length_in_ticks is: %d \n", new_sequence_b_length_in_ticks);
		//rt_printf("the_sequence_b is: %d \n", the_sequence_b);
	  //  print_binary(the_sequence_b);
	//	rt_printf("%c \n", 'B');
		
		
		//rt_printf("loop_timing_b.tick_count_in_sequence is: %d \n", loop_timing_b.tick_count_in_sequence);
    //	rt_printf("loop_timing_b.tick_count_since_start is: %d \n", loop_timing_b.tick_count_since_start);

		//rt_printf("step_b_count is: %d \n", step_b_count);

/*
      if (step_b_count == FIRST_STEP) {
    		rt_printf("FIRST_STEP B \n");
    	} else {
    		rt_printf("other step B \n");
    	}
  */  	
    	



/*
		rt_printf("envelope_1_setting is: %f \n", envelope_1_setting);

    	rt_printf("analog_per_sequence_adsr_a_level is: %f \n", analog_per_sequence_adsr_a_level);
    	rt_printf("analog_per_sequence_adsr_b_level is: %f \n", analog_per_sequence_adsr_b_level);
    	rt_printf("analog_per_sequence_adsr_c_level is: %f \n", analog_per_sequence_adsr_c_level);
*/

    	/*
    	rt_printf("envelope_1_release is: %f \n", envelope_1_release);
    	
    	

    	*/


		/*
		rt_printf("audio_osc_1_result is: %f \n", audio_osc_1_result);
		rt_printf("lfo_a_result_analog is: %f \n", lfo_a_result_analog);
		*/
		
		/*
		rt_printf("analog_out_1 is: %f \n", analog_out_1);
		rt_printf("analog_out_2 is: %f \n", analog_out_2);
		rt_printf("analog_out_3 is: %f \n", analog_out_3);
		rt_printf("analog_out_4 is: %f \n", analog_out_4);
		*/
		
		
		//rt_printf("delay_feedback_amount is: %f \n", delay_feedback_amount);
		
		
		
		
	//	rt_printf("analog_out_5 is: %f \n", analog_out_5);
	//	rt_printf("analog_out_6 is: %f \n", analog_out_6);
	//	rt_printf("analog_out_7 is: %f \n", analog_out_7);
	//	rt_printf("analog_out_8 is: %f \n", analog_out_8);
		
		
	//		rt_printf("draw_buf_write_pointer is: %d \n", draw_buf_write_pointer);
					
		

		
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

/*




    	
*/
    	
		/*
		rt_printf("gray_code_sequence_a is: %d \n", gray_code_sequence_a);
		print_binary(gray_code_sequence_a);
		rt_printf("%c \n", 'B');
		*/



		// Sequence state
		
    	//rt_printf("bar_a_count: %d \n", bar_a_count);
		//rt_printf("step_a_count: %d \n", step_a_count);
		//rt_printf("step_b_count: %d \n", step_b_count);
		
    	//rt_printf("bar_a_play: %d \n", bar_a_play);
		//rt_printf("step_a_play: %d \n", step_a_play);


    	
    	


    //	rt_printf("Midi last_note_on: %d \n", last_note_on);
    // 	rt_printf("Midi last_note_off: %d \n", last_note_off);
    //	rt_printf("Midi last_note_disabled: %d \n", last_note_disabled);
    	
    //	rt_printf("Midi midi_channel_x: %d \n", midi_channel_x);
    	
    	
    
    	

    //rt_printf("sequence_is_running is: %d \n", sequence_is_running);

    // Sequence Outputs 
    //rt_printf("target_analog_gate_a_out_state is: %d \n", target_analog_gate_a_out_state);
		//rt_printf("current_analog_gate_a_out_state is: %d \n", current_analog_gate_a_out_state);      


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



void DisableMidiNotes(uint8_t note){
	
	last_function = 28749;
	
             // Disable that note for all steps.
           uint8_t sc = 0;
           uint8_t bc = 0;
           for (bc = FIRST_BAR; bc <= MAX_BAR; bc++){
            for (sc = FIRST_STEP; sc <= MAX_STEP; sc++){
              // WRITE MIDI MIDI_DATA
              channel_x_midi_note_events[current_midi_lane][bc][sc][note][1].velocity = 0;
              channel_x_midi_note_events[current_midi_lane][bc][sc][note][1].is_active = 0;
              channel_x_midi_note_events[current_midi_lane][bc][sc][note][0].velocity = 0;
              channel_x_midi_note_events[current_midi_lane][bc][sc][note][0].is_active = 0;         
            }
           }
}


void OnMidiNoteInEvent(uint8_t on_off, uint8_t note, uint8_t velocity, int channel){
	// This function writes midi notes into channel_x_midi_note_events based on the current step_a_count

	last_function = 466942;

  rt_printf("Hi from OnMidiNoteInEvent I got MIDI note Event ON/OFF is %d, Note is %d, Velocity is %d, Channel is: %d - AND - bar_a_count is currently %d, step_a_count is currently %d \n", on_off, note, velocity, channel, bar_a_count, step_a_count);
  
  if (channel == midi_receive_channel){
  
    //rt_printf("***** I AM ACCEPTING the midi event because it is on channel %d and midi_receive_channel is: %d \n", channel, midi_receive_channel);


    if (note >= lowest_midi_note && note <= highest_midi_note) {

        //rt_printf("***** I AM PROCESSING the midi event because note %d is within lowest_midi_note %d and highest_midi_note %d \n", note, lowest_midi_note,  highest_midi_note);

    
      if (on_off == MIDI_NOTE_ON){

        // A mechanism to clear notes from memory by playing them quietly.
        if (velocity < 40 ){
          // Send Note OFF

          rt_printf("*** I GOT A LOW VELOCITY %d so will remove note %d from the sequence ***  \n", velocity, note);

          // Set to yellow
          target_led_4_tri_state = 2;

          midi.writeNoteOff(channel, note, 0);
          
          // Disable the note on all steps
          //Serial.println(String("DISABLE Note (for all steps) ") + note + String(" because ON velocity is ") + velocity );
          DisableMidiNotes(note);
          
          last_note_disabled = note;

          // Now, when we release this note on the keyboard, the keyboard obviously generates a note off which gets stored in channel_x_midi_note_events
          // and can interfere with subsequent note ONs i.e. cause the note to end earlier than expected.
          // Since velocity of Note OFF is not respected by keyboard manufactuers, we need to find a way remove (or prevent?)
          // these Note OFF events. 
          // One way is to store them here for processing after the note OFF actually happens. 
          // channel_x_ghost_events[note].is_active=1;
    
        } else {
          // We want the note on, so set it on. Note On
          if (current_midi_lane != SILENT_MIDI_LANE){ 
            rt_printf("Setting MIDI note ON for note %d When step is %d velocity is %d \n", note, step_a_count, velocity );
            // WRITE MIDI MIDI_DATA
            channel_x_midi_note_events[current_midi_lane][bar_a_count][step_a_count][note][1].tick_count_since_step = loop_timing_a.tick_count_since_step; // Only one of these per step.
            channel_x_midi_note_events[current_midi_lane][bar_a_count][step_a_count][note][1].velocity = velocity;
            channel_x_midi_note_events[current_midi_lane][bar_a_count][step_a_count][note][1].is_active = 1;

            
          } else {
            rt_printf("SILENT lane so NOT Writing note %d When step is %d velocity is %d \n", note, step_a_count, velocity );
          }

        } // End velocity check


        // Echo Midi but only if the sequencer is stopped, else we get double notes because PlayMidi gets called each Tick
        if (sequence_is_running == 0){

          // note this when we actually send the midi note out
          channel_x_midi_note_events[current_midi_lane][bar_a_count][step_a_count][note][1].tick_count_since_start = loop_timing_a.tick_count_since_start;
          channel_x_midi_note_events[current_midi_lane][bar_a_count][step_a_count][note][0].tick_count_since_start = 0;

          midi.writeNoteOn(channel, note, velocity); // echo midi to the output
        }
          
              
        last_note_on = note;
              
        rt_printf("Done setting MIDI note ON for note %d when step is %d velocity is %d \n", note,  step_a_count, velocity );

        } else { // End of Note ON   
          // MIDI Note OFF
          rt_printf("Set MIDI note OFF for note %d when bar is %d and step is %d \n", note,  bar_a_count, step_a_count );
          
          // WRITE MIDI MIDI_DATA (unless on silent lane) Note Off
          if (current_midi_lane != SILENT_MIDI_LANE){ 
            channel_x_midi_note_events[current_midi_lane][bar_a_count][step_a_count][note][0].tick_count_since_step = loop_timing_a.tick_count_since_step;
            channel_x_midi_note_events[current_midi_lane][bar_a_count][step_a_count][note][0].velocity = velocity;
            channel_x_midi_note_events[current_midi_lane][bar_a_count][step_a_count][note][0].is_active = 1;
          
          } else {
            rt_printf("SILENT lane so NOT setting MIDI note OFF for note %d when bar is %d and step is %d \n", note,  bar_a_count, step_a_count );
          }

          last_note_off = note;

          // Echo Midi but only if the sequencer is stopped, else we get double notes because PlayMidi gets called each Tick
          if (sequence_is_running == 0){ 

            channel_x_midi_note_events[current_midi_lane][bar_a_count][step_a_count][note][1].tick_count_since_start = 0;
            channel_x_midi_note_events[current_midi_lane][bar_a_count][step_a_count][note][0].tick_count_since_start = loop_timing_a.tick_count_since_start;


            midi.writeNoteOff(channel, note, 0);
          }
              
          rt_printf("Done setting MIDI note OFF (Sent) for note %d when bar is %d and step is %d \n", note,  bar_a_count, step_a_count );
      
        } // End of Note Off

    } else {
      	 // rt_printf("###### I am NOT processing the midi because the note %d was out of range - lowest note allowed: %d highest note allowed: %d \n", note, lowest_midi_note, highest_midi_note);

    }




  } else { // End correct receive_midi_channel
 	 // rt_printf("###### I am NOT ACCEPTING the midi because it was on channel %d and midi_receive_channel is set to: %d \n", channel, midi_receive_channel);
  } // End of receive channel check
} // End of function


void GateAHigh(){
	
   last_function = 982934;	
  //rt_printf("Gate HIGH at tick_count_since_start: %d ", loop_timing_a.tick_count_since_start);
  
  target_analog_gate_a_out_state = true;
  target_digital_gate_a_out_state = true;
  target_led_1_tri_state = 1; 


  

  Bela_scheduleAuxiliaryTask(gSendUdpMessage);


}

void GateALow(){
	
	last_function = 9849;
  //rt_printf("Gate LOW");
  
  target_analog_gate_a_out_state = false;
  target_digital_gate_a_out_state = false;
  
  target_led_1_tri_state = 0;
  
  //per_sequence_adsr_a.gate(false); // always reset it here but not trigger it
  //per_sequence_adsr_b.gate(false); // always reset it here but not trigger it
  //per_sequence_adsr_c.gate(false); // always reset it here but not trigger it
  
}

void GateBHigh(){
	
   last_function = 969636;	
	
  //rt_printf("Gate HIGH at tick_count_since_start: %d ", loop_timing_a.tick_count_since_start);
  
  target_analog_gate_b_out_state = true;
  
  target_digital_gate_b_out_state = true;

  target_led_3_tri_state = 1; 
    

}


void GateBLow(){
  //rt_printf("Gate LOW");
  
  last_function = 17469;
  
  target_analog_gate_b_out_state = false;

  target_digital_gate_b_out_state = false;

  target_led_3_tri_state = 0;
  
  
}




bool RampIsPositive(){
	last_function = 6684268;
	
	// TODO BELA
	return false;
  
}

// Kind of syncs the CV 
void SyncAndResetCv(){
	
  //rt_printf("------SyncAndResetCv-----\n");	
  
  
  //envelope_1.gate(true);
  
}



// Return bth bit of number from https://stackoverflow.com/questions/2249731/how-do-i-get-bit-by-bit-data-from-an-integer-value-in-c
uint8_t ReadBit (int number, int b ){
	
	last_function = 28642;
	(number & ( 1 << b )) >> b;
}






/////////////////////////////////////////////////////////////
// These are the possible beats of the sequence
void OnStepA(){
	
	last_function = 929732;
	
  //rt_printf("Hello from OnStepA: %d \n", step_a_count);
  //rt_printf("the_sequence_a is: %d \n", the_sequence_a);
  //print_binary(the_sequence_a);
  //rt_printf("%c \n", 'B');

  if (step_a_count > MAX_STEP) {
    rt_printf("----------------------------------------------------------------------------\n");  
    rt_printf("------------------ ERROR! step_a_count is: %s --- ERROR ---\n", + step_a_count);
    rt_printf("----------------------------------------------------------------------------\n");    
  }

  

    if (step_a_count == FIRST_STEP) {
    	//rt_printf("----   -------   YES FIRST_STEP     -------    ------\n");
      SyncAndResetCv();
    } else {
      //rt_printf("----       not first step      step_a_count is %d FIRST_STEP is %d                  ------\n", step_a_count, FIRST_STEP ); 
    }
  
  
  step_a_count = StepCountSanity(step_a_count);

 
  
  
  uint8_t play_a_note = (the_sequence_a & ( 1 << step_a_count )) >> step_a_count;  
  
  // Why does the line below trigger "Xenomai/cobalt: watchdog triggered" whereas the same logic in this function does not?
  //uint8_t play_note = ReadBit(the_sequence_a, step_a_count);
  
   if (play_a_note){
     //rt_printf("OnStepA: %d ****++++++****** PLAY \n", step_a_count);
    GateAHigh(); 
   } else {
    GateALow();
     //rt_printf("OnStepA: %d ***-----***** NOT play \n", step_a_count);
   }
   
   Bela_scheduleAuxiliaryTask(gPrintStatus);	

   //rt_printf("==== End of OnStepA: %d \n", step_a_count);
      
}



// These are ticks which are not steps - so in between possible beats.
void OnNotStepA(){
	
	last_function = 843;
	
  //rt_printf("NOT step_a_countIn is: ") + step_a_countIn  ); 
  // TODO not sure how this worked before. function name? ChangeCvWaveformBAmplitude(); 
  GateALow();
  
}

//////

/////////////////////////////////////////////////////////////
// These are the possible beats of the sequence
void OnStepB(){
	
	last_function = 296;
	
  //rt_printf("Hello from OnStepB: %d \n", step_a_count);
  //rt_printf("the_sequence_a is: %d \n", the_sequence_a);
  //print_binary(the_sequence_a);
  //rt_printf("%c \n", 'B');

  if (step_b_count > MAX_STEP) {
    rt_printf("----------------------------------------------------------------------------\n");  
    rt_printf("------------------ ERROR! step_b_count is: %s --- ERROR ---\n", + step_b_count);
    rt_printf("----------------------------------------------------------------------------\n");    
  }

  

    if (step_b_count == FIRST_STEP) {
    	//rt_printf("----   -------   YES FIRST_STEP     -------    ------\n");
      //SyncAndResetCv(); // TODOAB
    } else {
      //rt_printf("----       not first step      step_a_count is %d FIRST_STEP is %d                  ------\n", step_a_count, FIRST_STEP ); 
    }
  
  
  step_b_count = StepCountSanity(step_b_count);


  
  
  uint8_t play_b_note = (the_sequence_b & ( 1 << step_b_count )) >> step_b_count;  
  
  // Why does the line below trigger "Xenomai/cobalt: watchdog triggered" whereas the same logic in this function does not?
  //uint8_t play_note = ReadBit(the_sequence_a, step_a_count);
  
   if (play_b_note){
     //rt_printf("OnStepA: %d ****++++++****** PLAY \n", step_a_count);
    GateBHigh(); 
   } else {
    GateBLow();
     //rt_printf("OnStepA: %d ***-----***** NOT play \n", step_a_count);
   }
   
   Bela_scheduleAuxiliaryTask(gPrintStatus);	

   //rt_printf("==== End of OnStepA: %d \n", step_a_count);
      
}



// These are ticks which are not steps - so in between possible beats.
void OnNotStepB(){
	
	last_function = 69469;
	
  //rt_printf("NOT step_a_countIn is: ") + step_a_countIn  ); 
  // TODO not sure how this worked before. function name? ChangeCvWaveformBAmplitude(); 
  GateBLow();
  
}





//////

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





// See http://docs.bela.io/classMidi.html for the Bela Midi stuff

void PlayMidi(){

  // This function only plays midi based on the current state of step_a_play and whatever is in the channel_x_midi_note_events array.
	
	last_function = 364892;
	
  //rt_printf("midi_note  ") + i + String(" value is ") + channel_x_midi_note_events[step_a_count][i]  );

			// midi_byte_t statusByte = 0xB0; // control change on channel 0
			// midi_byte_t controller = 30; // controller number 30
			// midi_byte_t value = state * 127; // value : 0 or 127
			// midi_byte_t bytes[3] = {statusByte, controller, value};
			// midi.writeOutput(bytes, 3); // send a control change message


  // n represents each MIDI note from 0 to 127
  for (uint8_t n = 0; n <= 127; n++) {
    //rt_printf("** OnStepA  ") + step_a_count + String(" Note ") + n +  String(" ON value is ") + channel_x_midi_note_events[step_a_count][n][1]);
    
    // READ MIDI sequence (note ONs at the current global step_a_play)
    if (channel_x_midi_note_events[current_midi_lane][BarCountSanity(bar_a_play)][StepCountSanity(step_a_play)][n][1].is_active == 1) { 
           // The note could be on one of 6 ticks in the sequence
           if (channel_x_midi_note_events[current_midi_lane][BarCountSanity(bar_a_play)][StepCountSanity(step_a_play)][n][1].tick_count_since_step == loop_timing_a.tick_count_since_step){
            	//rt_printf("PlayMidi step_a_play: %d : tick_count_since_step %d Found and will send Note ON for %d \n", step_a_play, loop_timing_a.tick_count_since_step, n );
            	
              // Set LED 4 high
              target_led_4_tri_state = 1;

              
              // Note the fact that we sent a midi note ON (any record of a note OFF is now invalid)
              channel_x_midi_note_events[current_midi_lane][bar_a_count][step_a_count][n][1].tick_count_since_start = loop_timing_a.tick_count_since_start;
              channel_x_midi_note_events[current_midi_lane][bar_a_count][step_a_count][n][0].tick_count_since_start = 0;
              
              midi.writeNoteOn (midi_channel_x, n, channel_x_midi_note_events[current_midi_lane][BarCountSanity(bar_a_play)][StepCountSanity(step_a_count)][n][1].velocity);
           }
    } 

    // READ MIDI MIDI_DATA (note OFFs)
    if (channel_x_midi_note_events[current_midi_lane][BarCountSanity(bar_a_play)][StepCountSanity(step_a_count)][n][0].is_active == 1) {
       if (channel_x_midi_note_events[current_midi_lane][BarCountSanity(bar_a_play)][StepCountSanity(step_a_count)][n][0].tick_count_since_step == loop_timing_a.tick_count_since_step){ 
           //rt_printf("Step:Ticks ") + step_a_count + String(":") + ticks_after_step_a +  String(" Found and will send Note OFF for ") + n );

           // Set LED 4 low
           target_led_4_tri_state = 0;

          // Note the fact that we sent a midi note OFF (any record of a note ON is now invalid)
          channel_x_midi_note_events[current_midi_lane][bar_a_count][step_a_count][n][1].tick_count_since_start = 0;
          channel_x_midi_note_events[current_midi_lane][bar_a_count][step_a_count][n][0].tick_count_since_start = loop_timing_a.tick_count_since_start;

          midi.writeNoteOff(midi_channel_x, n, 0);
       }
    }
  } // End midi note loop

} // End Play Midi





/////////
void AdvanceSequenceAChronology(){
	
	last_function = 2386;
  
  // This function advances or resets the sequence powered by the clock.

  // But first check / set the desired sequence length

  //rt_printf("Hello from AdvanceSequenceAChronology ");


  // Reverse because we want fully clockwise to be short so we get 1's if sequence is 1.
  //current_sequence_a_length_in_steps = MAX_SEQUENCE_LENGTH_IN_STEPS - current_sequence_a_length_in_steps_raw;

  //rt_printf("current_sequence_a_length_in_steps is: %d ", current_sequence_a_length_in_steps  );

  if (current_sequence_a_length_in_steps < MIN_SEQUENCE_LENGTH_IN_STEPS){
    rt_printf("**** ERROR with current_sequence_a_length_in_steps it WAS: %d but setting it to: %d ", current_sequence_a_length_in_steps, MIN_SEQUENCE_LENGTH_IN_STEPS );
    current_sequence_a_length_in_steps = MIN_SEQUENCE_LENGTH_IN_STEPS; 
    
  }
  
  if (current_sequence_a_length_in_steps > MAX_SEQUENCE_LENGTH_IN_STEPS){
    current_sequence_a_length_in_steps = MAX_SEQUENCE_LENGTH_IN_STEPS; 
    rt_printf("**** ERROR with current_sequence_a_length_in_steps but it is NOW: %d ", current_sequence_a_length_in_steps  );
  }

  new_sequence_a_length_in_ticks = (current_sequence_a_length_in_steps) * 6;
  //Serial.println(String("current_sequence_a_length_in_steps is: ") + current_sequence_a_length_in_steps  ); 
  //Serial.println(String("new_sequence_a_length_in_ticks is: ") + new_sequence_a_length_in_ticks  );  

  // Always advance the ticks SINCE START
  SetTotalTickCountA(loop_timing_a.tick_count_since_start += 1);




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
  // new_sequence_a_length_in_ticks is one indexed
  // 

  // Should we Reset or Increment?
  // If we're at the end of the sequence
  if (
    (loop_timing_a.tick_count_in_sequence + 1 == new_sequence_a_length_in_ticks )
  // this causes multiple resets on bela
  // or we past the end and we're at new beat  
  // WE NEED this check else we crash when sequence length gets reduced because we overshoot 
  ||
  (loop_timing_a.tick_count_in_sequence + 1  >= new_sequence_a_length_in_ticks 
      && 
      // loop_timing_a.tick_count_since_start % new_sequence_a_length_in_ticks == 0 
      // If somehow we overshot (because pot was being turned whilst sequence running), only 
      loop_timing_a.tick_count_since_start % 6 == 0 
  )
  // or we're past 16 beats worth of ticks. (this could happen if the sequence length gets changed during run-time)
  || 
  loop_timing_a.tick_count_in_sequence >= 16 * 6
  ) { // Reset
    ResetSequenceACounters();
  } else {
    SetTickCountInSequenceA(loop_timing_a.tick_count_in_sequence += 1); // Else increment.
  }

  // Update Step Count (this could also be a function but probably makes sense to store it)
  // An integer operation - we just want the quotient.
  step_a_count = loop_timing_a.tick_count_in_sequence / 6;

  // Just to show the tick progress  
  ticks_after_step_a = loop_timing_a.tick_count_in_sequence % 6;

 //Serial.println(String("step_a_count is ") + step_a_count  + String(" ticks_after_step_a is ") + ticks_after_step_a  );
 
 SetPlayAFromCount();

  
}


void AdvanceSequenceBChronology(){
	
	last_function = 585635;
  
 //rt_printf("**** AdvanceSequenceBChronology");
 
 
 
  if (current_sequence_b_length_in_steps < MIN_SEQUENCE_LENGTH_IN_STEPS){
    rt_printf("**** ERROR with current_sequence_b_length_in_steps it WAS: %d but setting it to: %d ", current_sequence_b_length_in_steps, MIN_SEQUENCE_LENGTH_IN_STEPS );
    current_sequence_b_length_in_steps = MIN_SEQUENCE_LENGTH_IN_STEPS; 
    
  }
  
  if (current_sequence_b_length_in_steps > MAX_SEQUENCE_LENGTH_IN_STEPS){
    current_sequence_b_length_in_steps = MAX_SEQUENCE_LENGTH_IN_STEPS; 
    rt_printf("**** ERROR with current_sequence_b_length_in_steps but it is NOW: %d ", current_sequence_b_length_in_steps  );
  }

  new_sequence_b_length_in_ticks = (current_sequence_b_length_in_steps) * 6;
  //Serial.println(String("current_sequence_b_length_in_steps is: ") + current_sequence_b_length_in_steps  ); 
  //Serial.println(String("new_sequence_b_length_in_ticks is: ") + new_sequence_b_length_in_ticks  );  

  // Always advance the ticks SINCE START
  SetTotalTickCountB(loop_timing_b.tick_count_since_start += 1);


  // If we're at the end of the sequence
  if (
    (loop_timing_b.tick_count_in_sequence + 1 == new_sequence_b_length_in_ticks )

  // or we past the end and we're at new beat  
  ||
  (loop_timing_b.tick_count_in_sequence + 1  >= new_sequence_b_length_in_ticks 
      && 
      // loop_timing_b.tick_count_since_start % new_sequence_b_length_in_ticks == 0 
      // If somehow we overshot (because pot was being turned whilst sequence running), only 
      loop_timing_b.tick_count_since_start % 6 == 0 
  )
  // or we're past 16 beats worth of ticks. (this could happen if the sequence length gets changed during run-time)
  || 
  loop_timing_b.tick_count_in_sequence >= 16 * 6
  ) { // Reset
    ResetSequenceBCounters();
  } else {
    SetTickCountInSequenceB(loop_timing_b.tick_count_in_sequence += 1); // Else increment.
  }

  // Update Step Count (this could also be a function but probably makes sense to store it)
  // An integer operation - we just want the quotient.
  step_b_count = loop_timing_b.tick_count_in_sequence / 6;

  // Just to show the tick progress  
  ticks_after_step_b = loop_timing_b.tick_count_in_sequence % 6;

 //Serial.println(String("step_b_count is ") + step_b_count  + String(" ticks_after_step_a is ") + ticks_after_step_a  );
 
 SetPlayBFromCount();

  
}


void InitMidiSequence(bool force){
	
	last_function = 32786;
	
  if (init_midi_sequence_has_run == false || force == true){	

  rt_printf("InitMidiSequence Start \n");


  // Loop through lanes
  for (uint8_t ln = MIN_LANE; ln <= MAX_LANE; ln++) {

  // Loop through bars
  for (uint8_t bc = FIRST_BAR; bc <= MAX_BAR; bc++) {

    // Loop through steps
    for (uint8_t sc = FIRST_STEP; sc <= MAX_STEP; sc++) {
      //Serial.println(String("Step ") + sc );
    
      // Loop through notes
      for (uint8_t n = 0; n <= 127; n++) {

        if (channel_x_midi_note_events[ln][bc][sc][n][1].is_active == 1){	
          rt_printf("ACTIVE Init Step is: %d Note is: %d Note ON is_active is: %d tick_count_since_start is %d \n", sc, n, channel_x_midi_note_events[ln][bc][sc][n][1].is_active, channel_x_midi_note_events[ln][bc][sc][n][1].tick_count_since_start);
        } else {
          rt_printf(".");
        }

        // Initialise and print Note on (1) and Off (2) contents of the array.
        // WRITE MIDI MIDI_DATA
        channel_x_midi_note_events[ln][bc][sc][n][1].is_active = 0;
        channel_x_midi_note_events[ln][bc][sc][n][0].is_active = 0;

// HERE
       // rt_printf("Init Step ") + %d sc + " Note " + n +  " OFF ticks value is " + channel_x_midi_note_events[sc][n][0].is_active);




//rt_printf("Init Step ") + sc + " Note " + n +  " OFF ticks value is " + channel_x_midi_note_events[sc][n][0].is_active);

  
      //rt_printf("Init Step ") + sc + String(" Note ") + n +  String(" ON ticks value is ") + channel_x_midi_note_events[sc][n][1].is_active);
      //rt_printf("Init Step ") + sc + String(" Note ") + n +  String(" OFF ticks value is ") + channel_x_midi_note_events[sc][n][0].is_active);
      } 
    }
  }
}
    //for (uint8_t n = 0; n <= 127; n++) {
    // channel_x_ghost_events[n].is_active = 0;
     //rt_printf("Init Step with ghost Note: %s is_active false", n );
  //}
  


    init_midi_sequence_has_run = true;

	rt_printf("InitMidiSequence Done \n");
	
  } else {
  	rt_printf("InitMidiSequence Skipped because already done \n");
  }
	
	
} // End of InitMidiSequence


void InitMidiSequenceForce(void*){
  last_function = 56811;
  InitMidiSequence(true);
}

void InitMidiSequenceNoForce(void*){
  last_function = 78911;
  InitMidiSequence(false);
}


void OnTick(){
	
	last_function = 55411;
	
// Called on Every MIDI or Analogue clock pulse
// Drives sequencer activity.
// Can be called from Midi Clock and/or Digital Clock In.

 //rt_printf("Hello from OnTick \n");

  last_tick_frame = frame_timer;	

  /////////////////
  // BPM Detection (we use A)
  if (loop_timing_a.tick_count_since_start % 24 == 0){
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
  
  // clock divide here?

  // This is how we define when a step happens.
  if (loop_timing_a.tick_count_in_sequence % (6 * clock_divider_input_value) == 0){
    //clockShowHigh();
    //rt_printf("loop_timing_a.tick_count_in_sequence is: ") + loop_timing_a.tick_count_in_sequence + String(" the first tick of a crotchet or after MIDI Start message") );    
    //////////////////////////////////////////
    OnStepA();
    /////////////////////////////////////////   
  } else {
    //clockShowLow();
    // The other ticks which are not "steps".
    OnNotStepA();
    //rt_printf("timing.tick_count_in_sequence is: ") + timing.tick_count_in_sequence );
  }


  // Decide if we have a "step"
  if (loop_timing_b.tick_count_in_sequence % (6 * clock_divider_input_value) == 0){
    //clockShowHigh();
    //rt_printf("loop_timing_a.tick_count_in_sequence is: ") + loop_timing_a.tick_count_in_sequence + String(" the first tick of a crotchet or after MIDI Start message") );    
    //////////////////////////////////////////
    OnStepB();
    /////////////////////////////////////////   
  } else {
    //clockShowLow();
    // The other ticks which are not "steps".
    OnNotStepB();
    //rt_printf("timing.tick_count_in_sequence is: ") + timing.tick_count_in_sequence );
  }

  
  
  // Play any suitable midi in the sequence (note, we read midi using a callback)

// Here we could clock divide 

// Do we call this too often?
  PlayMidi();
   
  // Advance and Reset ticks and steps
  AdvanceSequenceAChronology();
  AdvanceSequenceBChronology();

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
	
	last_function = 9945443;

	int MIDI_STATUS_OF_CLOCK = 7; // not  (decimal 248, hex 0xF8) ??
	
  uint8_t MIDI_SUSTAIN_PEDAL_TYPE = 3; 
  uint8_t MIDI_SUSTAIN_PEDAL_NOTE_NUMBER = 64;
  uint8_t MIDI_SUSTAIN_PEDAL_VELOCITY = 127;
  uint8_t MIDI_SUSTAIN_PEDAL_CHANNEL = 0; 



	// Read midi loop

	uint8_t type_received = message.getType();
	uint8_t note_received = message.getDataByte(0); // may have different meaning depending on the type
	uint8_t velocity_received = message.getDataByte(1); // may have different meaning depending on the type
	int channel_received = message.getChannel(); // only recently we use the channel

//rt_printf("B4 PrettyPrint type: %d, note: %d, velocity: %d, channel: %d  \n", type_received, note_received, velocity_received, channel_received);

//  message.prettyPrint();

// rt_printf("\n");  

	//rt_printf("Midi Message: type: %d, note: %d, velocity: %d, channel: %d  \n", type_received, note_received, velocity_received, channel_received);

	//rt_printf("Note: kmmNoteOn: %d, kmmNoteOff: %d \n", kmmNoteOn, kmmNoteOff);

	// Check for a NOTE ON
    // Some keyboards send velocity 0 for note off instead of sending 0 for type, so we must check for that.
	if(type_received == kmmNoteOn && velocity_received > 0){
			rt_printf("note_received ON: type_received: %d, note_received: %d velocity_received: %d channel: %d \n", type_received, note_received, velocity_received, channel_received);
			// Write any note ON into the sequence
			OnMidiNoteInEvent(MIDI_NOTE_ON, note_received, velocity_received, channel_received);
	// Check for a NOTE OFF	 
	} else if (message.getType() == kmmNoteOff || message.getDataByte(1) == 0){
		
			rt_printf("note_received OFF: type_received: %d, note_received: %d velocity_received: %d channel: \n", type_received, note_received, velocity_received, channel_received);
			
			// Write any note OFF into the sequence
			OnMidiNoteInEvent(MIDI_NOTE_OFF, note_received, velocity_received, channel_received);
	} else if (message.getType() == MIDI_SUSTAIN_PEDAL_TYPE && note_received == MIDI_SUSTAIN_PEDAL_NOTE_NUMBER && velocity_received == MIDI_SUSTAIN_PEDAL_VELOCITY && channel_received == MIDI_SUSTAIN_PEDAL_CHANNEL){
    target_led_4_tri_state = 2; // yellow
    //InitMidiSequence(true);

    Bela_scheduleAuxiliaryTask(gInitMidiSequenceForce);
    Bela_scheduleAuxiliaryTask(gAllNotesOff);
	// CLOCK but not currently working.
  } else if (message.getType() == MIDI_STATUS_OF_CLOCK) {
			// Midi clock  (decimal 248, hex 0xF8) - for some reason the library returns 7 for clock (kmmSystem ?)
		// int type = message.getType();
		// int byte0 = message.getDataByte(0);
		// int byte1 = message.getDataByte(1);
    	
    	
    //	rt_printf("MAYBE CLOCK MIDI Message: type_received: %d, note_received: %d velocity_received: %d channel: %d \n", type_received, note_received, velocity_received, channel_received);
    	

		//rt_printf("THINK I GOT MIDI CLOCK - type: %d byte0: %d  byte1 : %d \n", type, byte0, byte1);
		
		// UNRELIABLE AT THE MOMENT
		
		//midi_clock_detected = 1;
		//OnTick();

	// OTHER 
	} else {
			rt_printf("***** OTHER MIDI Message *******: type_received: %d, note_received: %d velocity_received: %d channel: %d \n", type_received, note_received, velocity_received, channel_received);
	}
}





//////////////

// https://www.codesdope.com/blog/article/set-toggle-and-clear-a-bit-in-c/
// That will clear the nth bit of number. You must invert the bit string with the bitwise NOT operator (~), then AND it.
int BitClear (unsigned int number, unsigned int n) {
	
	last_function = 54543232;
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
	
	last_function = 4534;

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
 	last_function = 561122;
 	
 	//rt_printf("======== Hello from Binary2Gray data is: %d ========= \n",data);
 	
   unsigned int n_data = (data>>1);
   unsigned int gray_code = (data ^ n_data);
   
   //rt_printf("======== Bye from Binary2Gray gray_code is: %d ========= \n",gray_code);

  return gray_code;
 }
///////////////////////////////////////////////////////////////


// linear conversion https://stackoverflow.com/questions/929103/convert-a-number-range-to-another-range-maintaining-ratio

float linearScale( float original_range_min, float original_range_max, float new_range_min, float new_range_max, float original_value){
	
	last_function = 990088;

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
	
	last_function = 948462;
  
  // change by an amount (might go up or down)
  cv_waveform_b_amplitude += cv_waveform_b_amplitude_delta;

  // TODO do something with bela
  // SetWaveformBObjectAmplitude ();
 
}

void SetWaveformBObjectAmplitude (){
	
	last_function = 296689;
  
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
	
	last_function = 883355;
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

// look here
void AllNotesOff(void*){
	
	last_function = 99883352;
	
	// Don't do this too often
	if (frame_timer - last_notes_off_frame > 11000){
		  // All MIDI notes off.
		  rt_printf("All MIDI notes OFF \n");
		  for (uint8_t n = 0; n <= 127; n++) {
		     midi.writeNoteOff(midi_channel_x, n, 0);
         target_led_4_tri_state = 0;
		  }
		  last_notes_off_frame = frame_timer;
	} else {
		rt_printf("SKIPPING All MIDI notes OFF \n");
	}
  
}

void SetLane(uint8_t lane_in){
	
	last_function = 5533889;

	if (lane_in > MAX_LANE){
		current_midi_lane = MAX_LANE;
	} else if (lane_in < MIN_LANE){
		current_midi_lane = MIN_LANE;
	} else {
		current_midi_lane = lane_in;
	}

  if (previous_lane != current_midi_lane){
    // Don't want to call this too often.
    // Need to limit this in case we have CV input making current_midi_lane change fast

    if (current_midi_lane == SILENT_MIDI_LANE){

// TODO bug here since this seems to be called too often.

       rt_printf("Just changed to SILENT_MIDI_LANE \n"); 
       Bela_scheduleAuxiliaryTask(gAllNotesOff);
    }


    //Bela_scheduleAuxiliaryTask(gAllNotesOff);
    previous_lane = current_midi_lane;
  }
}






// Each time we start the sequencer we want to start from the same conditions.
void InitSequencer(){
	
  last_function = 9876542;	
	
  GateALow();
  GateBLow();

  CvStop();


  loop_timing_a.tick_count_since_start = 0;
  ResetSequenceACounters();

  loop_timing_b.tick_count_since_start = 0;
  ResetSequenceBCounters();


}

void StartSequencer(){
	
	last_function = 6782341;
	
  InitSequencer();
  sequence_is_running = HIGH;
  need_to_auto_save_sequence = true;
}

void StopSequencer(){
	
	last_function = 82558;
//	auto millisec_since_epoch_2 = duration_cast<milliseconds>(system_clock::now().time_since_epoch()).count();
	
	
//	unsigned long milliseconds_since_epoch = std::chrono::system_clock::now().time_since_epoch();
	

  // Note the format %llu is used to format 64bit unsigned integer. 
  // see https://stackoverflow.com/questions/32112497/how-to-printf-a-64-bit-integer-as-hex
  // https://stackoverflow.com/questions/18107426/printf-format-for-unsigned-int64-on-windows
  //rt_printf("Stop Sequencer at: %llx \n", frame_timer);   // works - hex
  rt_printf("Stop Sequencer at: %llu \n", frame_timer); // works - unsigned
  

  InitSequencer();
  sequence_is_running = LOW;
  
  Bela_scheduleAuxiliaryTask(gAllNotesOff);

}






int GetValue(int raw, int last, int jitter_reduction){
	
	last_function = 2542;
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
	
	last_function = 5375;
  // Return true if the two values are close
  if (abs(value_1 - value_2) <= fuzzyness){
    return true;
  } else {
    return false;
  }
}





float gDelayBuffer_l[DELAY_BUFFER_SIZE] = {0};
float gDelayBuffer_r[DELAY_BUFFER_SIZE] = {0};


float draw_buffer[DELAY_BUFFER_SIZE] = {0};
float draw_gDelayBuffer_r[DELAY_BUFFER_SIZE] = {0};


// Write pointer
int gDelayBufWritePtr = 0;



// Amount of delay
float gDelayAmount = 1.0;


float draw_feedback_ratio = 1.0;


// Level of pre-delay input
float gDelayAmountPre = 0.75;

float draw_feedback_ratioPre = 0.75;


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

///









////
void InitAudioBuffer(){
	
	last_function = 25852;
	// Buffer holding previous samples per channel
	gDelayBuffer_l[DELAY_BUFFER_SIZE] = {0};
	gDelayBuffer_r[DELAY_BUFFER_SIZE] = {0};
	// Write pointer
	gDelayBufWritePtr = 0;

}


void ChangeSequence(void*){
	
	last_function = 24964222;
	
	 //rt_printf(" ChangeSequence " );
	
	
	
	uint8_t sequence_pattern_lower_limit = 1;  // Setting to 1 means we never get 0 i.e. a blank sequence especially when we change seq length
  unsigned int sequence_a_pattern_upper_limit = 1023; 
	unsigned int sequence_b_pattern_upper_limit = 1023;
	
	
	sequence_a_pattern_input = static_cast<int>(round(map(sequence_a_pattern_input_raw, 0, 1, sequence_pattern_lower_limit, sequence_a_pattern_upper_limit))); 
    //rt_printf("**** NEW value for sequence_a_pattern_input is: %d ", sequence_a_pattern_input  );
    
	sequence_b_pattern_input = static_cast<int>(round(map(sequence_b_pattern_input_raw, 0, 1, sequence_pattern_lower_limit, sequence_a_pattern_upper_limit))); 
    //rt_printf("**** NEW value for sequence_a_pattern_input is: %d ", sequence_a_pattern_input  );



    current_sequence_a_length_in_steps = static_cast<int>(round(map(sequence_a_length_input_raw, 0, 1, MIN_SEQUENCE_LENGTH_IN_STEPS, MAX_SEQUENCE_LENGTH_IN_STEPS))); 
    
	current_sequence_b_length_in_steps = static_cast<int>(round(map(sequence_b_length_input_raw, 0, 1, MIN_SEQUENCE_LENGTH_IN_STEPS, MAX_SEQUENCE_LENGTH_IN_STEPS))); 


 
    //////////////////////////////////////////
// Assign values to change the sequencer.
///////////////////////////////////

   last_binary_sequence_a_result = binary_sequence_a_result;
   last_binary_sequence_b_result = binary_sequence_b_result;
 

   // If we have 8 bits, use the range up to 255



  

//binary_sequence_upper_limit = pow(current_sequence_a_length_in_steps, 2);

// REMEMBER, current_sequence_a_length_in_steps is ONE indexed (from 1 up to 16) 
// For a 3 step sequence we want to cover all the possibilities of a 3 step sequence which is (2^3) - 1 = 7
// i.e. all bits on of a 3 step sequence is 111 = 7 decimal 
// or (2^current_sequence_a_length_in_steps) - 1
sequence_a_pattern_upper_limit = pow(2, current_sequence_a_length_in_steps) - 1; 
sequence_b_pattern_upper_limit = pow(2, current_sequence_b_length_in_steps) - 1; 


   //rt_printf("binary_sequence_upper_limit is: ") + binary_sequence_upper_limit  );
    


  // Button is in Normal state (not pressed) (HIGH) (button_1_state == HIGH)
   // ***UPPER Pot HIGH Button*** //////////
  // Generally the lowest value from the pot we get is 2 or 3 
  // setting-1
  binary_sequence_a_result = fscale( 1, 1023, sequence_pattern_lower_limit, sequence_a_pattern_upper_limit, sequence_a_pattern_input, 0);
  binary_sequence_b_result = fscale( 1, 1023, sequence_pattern_lower_limit, sequence_b_pattern_upper_limit, sequence_b_pattern_input, 0);
   


   if (binary_sequence_a_result != last_binary_sequence_a_result){
    //rt_printf("binary_sequence has changed **");
   }


   //rt_printf("binary_sequence_a_result is: ") + binary_sequence_a_result  );
   //Serial.print("\t");
   //Serial.print(binary_sequence_a_result, BIN);
   //Serial.println();

   gray_code_sequence_a = Binary2Gray(binary_sequence_a_result);
   //rt_printf("gray_code_sequence_a is: ") + gray_code_sequence_a  );
   //Serial.print("\t");
   //Serial.print(gray_code_sequence_a, BIN);
   //Serial.println();
   gray_code_sequence_b = Binary2Gray(binary_sequence_b_result);


    the_sequence_a = gray_code_sequence_a;
    the_sequence_b = gray_code_sequence_b;

    //the_sequence_a = BitClear(the_sequence_a, current_sequence_a_length_in_steps -1); // current_sequence_a_length_in_steps is 1 based index. bitClear is zero based index.
    //the_sequence_a = ~ the_sequence_a; // Invert

    // So pot fully counter clockwise is 1 on the first beat 
    // if (binary_sequence_a_result == 1){
    //   the_sequence_a = 1;
    // }

   //rt_printf("the_sequence_a is: %s ", the_sequence_a  );
   //Serial.print("\t");
   //Serial.print(the_sequence_a, BIN);
   //Serial.println();
   
   

   // Modify Attack
   per_sequence_adsr_a.setAttackRate(envelope_1_setting * analog_sample_rate);

   // Modifiy Attack and Decay
   per_sequence_adsr_b.setAttackRate(envelope_1_setting * analog_sample_rate);
   per_sequence_adsr_b.setDecayRate(envelope_1_setting * analog_sample_rate);

   // Modify Decay
   per_sequence_adsr_c.setDecayRate(envelope_1_setting * analog_sample_rate);


      lfo_a_analog.setFrequency(lfo_osc_1_frequency); 
      lfo_b_analog.setFrequency(lfo_osc_2_frequency);
		
		
		// We want to Delay Course Dial to span the DELAY_BUFFER_SIZE in jumps of frames_per_24_ticks
		float delay_coarse_dial_factor = DELAY_BUFFER_SIZE / (frames_per_24_ticks * MAX_COARSE_DELAY_TIME_INPUT);
		
		
		// The course delay amount we dial in with the pot
		coarse_delay_frames = rint(frames_per_24_ticks * coarse_delay_input * delay_coarse_dial_factor);	    
	    
		// The amount we increment / decrement the delay using buttons 2 and 1
		//fine_delay_frames_delta = rint(frames_per_24_ticks / 24.0);	
		
		fine_delay_frames_delta = rint(frames_per_24_ticks / 48.0);	
		

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



		// Clear Audio Buffer (Delay)
		if (do_button_3_action == 1) {
			InitAudioBuffer();
			do_button_3_action = 0;
		}



		// Clear Midi sequence
		if (do_button_4_action == 1) {
      Bela_scheduleAuxiliaryTask(gInitMidiSequenceForce);
			Bela_scheduleAuxiliaryTask(gAllNotesOff);
      target_led_4_tri_state = 2; // yellow
			do_button_4_action = 0;
		}


	
} // end of function






void MaybeOnTick(){
	
	last_function = 6644;
  if (do_tick == true){
    do_tick = false;
    OnTick();
  }
}



#include <libraries/WriteFile/WriteFile.h>
WriteFile file1;
WriteFile file2;



void SendUdpMessage(void*){
	
	last_function = 686587;

     // std::string message2 = ">:" + std::to_string(analog_out_2) + ":<";


int32_t test = 123;

rt_printf("*********** BEFORE UDP ******************* \n");
      
	 // This sends a UDP message 
	 int my_result  = myUdpClient0->send(&test, 32);

rt_printf("******** after UDP *******. \n");

}   




/////////////////////////////////////////////////////////

bool setup(BelaContext *context, void *userData){
	
	last_function = 42396;
	
	//main();
	
	
	rt_printf("Hello from Setup: SimonSaysSeeq on Bela %s:-) \n", version);
	
  InitAudioBuffer();

	scope.setup(4, context->analogSampleRate);

	lfo_a_analog.setup(context->analogSampleRate);
  lfo_b_analog.setup(context->analogSampleRate);


	//oscillator_2_audio.setup(context->audioSampleRate);

	

	

 
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
        
        osc_bank.setup(context->audioSampleRate, gWavetableLength, gNumOscillators);
        // Fill in the wavetable with one period of your waveform
        float* wavetable = osc_bank.getWavetable();
        for(int n = 0; n < osc_bank.getWavetableLength() + 1; n++){
                wavetable[n] = sinf(2.0 * M_PI * (float)n / (float)osc_bank.getWavetableLength());
        }
        
        // Initialise frequency and amplitude
        float freq = kMinimumFrequency;
        float increment = (kMaximumFrequency - kMinimumFrequency) / (float)gNumOscillators;
        for(int n = 0; n < gNumOscillators; n++) {
                if(context->analogFrames == 0) {
                        // Random frequencies when used without analogInputs
                        osc_bank.setFrequency(n, kMinimumFrequency + (kMaximumFrequency - kMinimumFrequency) * ((float)random() / (float)RAND_MAX));
                }
                else {
                        // Constant spread of frequencies when used with analogInputs
                        osc_bank.setFrequency(n, freq);
                        freq += increment;
                }
                osc_bank.setAmplitude(n, (float)random() / (float)RAND_MAX / (float)gNumOscillators);
        }
        increment = 0;
        freq = 440.0;
        for(int n = 0; n < gNumOscillators; n++) {
                // Update the frequencies to a regular spread, plus a small amount of randomness
                // to avoid weird phase effects
                float randScale = 0.99 + .02 * (float)random() / (float)RAND_MAX;
                float newFreq = freq * randScale;
                // For efficiency, frequency is expressed in change in wavetable position per sample, not Hz or radians
                osc_bank.setFrequency(n, newFreq);
                freq += increment;
        }
        // Initialise auxiliary tasks
        //if((gFrequencyUpdateTask = Bela_createAuxiliaryTask(&recalculate_frequencies, 85, "bela-update-frequencies")) == 0)
        //        return false;
        
        

        audio_sample_rate = context->audioSampleRate;
        analog_sample_rate = context->analogSampleRate;


        // SETUP ADSR (these are modified in Change Sequence)
         // This envelope triggers at the start of each sequence
         // Attack can be modified
        per_sequence_adsr_a.setAttackRate(envelope_1_setting * context->analogSampleRate); // resulting times are in seconds
        per_sequence_adsr_a.setDecayRate(envelope_1_decay * context->analogSampleRate);
        per_sequence_adsr_a.setSustainLevel(envelope_1_sustain); // Level not time
        per_sequence_adsr_a.setReleaseRate(envelope_1_release * context->analogSampleRate);
       
        
        // This envelope triggers at the start of each sequence
        // Attack and Decay can be modified
        per_sequence_adsr_b.setAttackRate(envelope_1_setting * context->analogSampleRate);
        per_sequence_adsr_b.setDecayRate(envelope_1_setting * context->analogSampleRate);
        per_sequence_adsr_b.setSustainLevel(envelope_1_sustain); // Level not time
        per_sequence_adsr_b.setReleaseRate(envelope_1_release * context->analogSampleRate);
        

        // This envelope triggers at the start of each sequence
        // Decay can be modified
        per_sequence_adsr_c.setAttackRate(envelope_1_attack  * context->analogSampleRate);
        per_sequence_adsr_c.setDecayRate(envelope_1_setting * context->analogSampleRate);
        per_sequence_adsr_c.setSustainLevel(envelope_1_sustain); // Level not time
        per_sequence_adsr_c.setReleaseRate(envelope_1_release * context->analogSampleRate);
        


        // The two buttons on Salt as inputs
        pinMode(context, 0, button_1_PIN, INPUT);
        pinMode(context, 0, button_2_PIN, INPUT);


        // The two buttons on Salt + as inputs
        pinMode(context, 0, button_3_PIN, INPUT);
        pinMode(context, 0, button_4_PIN, INPUT);


        // The two LEDS on Salt
        pinMode(context, 0, LED_1_PIN, OUTPUT); // RED
        //pinMode(context, 0, LED_1_PIN, INPUT); // YELLOW - doesn't work




        pinMode(context, 0, LED_2_PIN, OUTPUT);
        
        // The two LEDS on Salt + 
        pinMode(context, 0, LED_3_PIN, OUTPUT);
        pinMode(context, 0, LED_4_PIN, OUTPUT);

        // Get different colours out of LEDS? See https://github.com/BelaPlatform/Bela/blob/master/examples/Digital/bicolor-LEDs/render.cpp


// UPD setup UDP

myUdpClient0 = new UdpClient(remotePort0,remoteIP);





        if((gChangeSequenceTask = Bela_createAuxiliaryTask(&ChangeSequence, 83, "bela-change-sequence")) == 0)
                return false;

        if((gInitMidiSequenceForce = Bela_createAuxiliaryTask(&InitMidiSequenceForce, 82, "bela-init-midi-sequence-force")) == 0)
                return false;

        if((gInitMidiSequenceNoForce = Bela_createAuxiliaryTask(&InitMidiSequenceNoForce, 82, "bela-init-midi-sequence-no-force")) == 0)
                return false;


        if((gPrintStatus = Bela_createAuxiliaryTask(&printStatus, 80, "bela-print-status")) == 0)
                return false;

        if((gAllNotesOff = Bela_createAuxiliaryTask(&AllNotesOff, 75, "bela-all-notes-off")) == 0)
                return false;   
                
        if((gWriteSequenceToFiles = Bela_createAuxiliaryTask(&WriteSequenceToFiles, 70, "bela-write-sequence-to-files")) == 0)
                return false;
                



        if((gSendUdpMessage = Bela_createAuxiliaryTask(&SendUdpMessage, 60, "bela-send-udp-message")) == 0)
                return false;
                
                
                

    
        
        
        gSampleCount = 0;
        
        //rt_printf("Before myUdpClient.setup in Setup. \n");

        //myUdpClient.setup(50002, "18.195.30.76"); 

        //rt_printf("After myUdpClient.setup in Setup. \n");
        
    // Create Midi Sequence in memory Structure
    // InitMidiSequence(false);

    Bela_scheduleAuxiliaryTask(gInitMidiSequenceNoForce);
        
	// Now that we have created the structure in memory above, we can populate it from files stored last time we shut down the sequencer nicely.
	ReadSequenceFromFiles();
        
        rt_printf("Bye from Setup. - I hope you will send me a clock :-) \n");

        return true;
}

// Thanks to giuliomoro
void drivePwm(BelaContext* context, int pwmPin)
{
    static unsigned int count = 0;
    pinMode(context, 0, pwmPin, OUTPUT);
    for(unsigned int n = 0; n < context->digitalFrames; ++n)
    {
        digitalWriteOnce(context, n, pwmPin, count > 8);
        count++;
        if(32 == count)
                count = 0;
    }
}

// Thanks to giuliomoro
void setLed(BelaContext* context, int ledPin,  int color)
{
    switch(color)
    {
        case 0: // off
            pinMode(context, 0, ledPin, INPUT);
            break;
        case 1: // red 
        case 2: // yellow 
            pinMode(context, 0, ledPin, OUTPUT);
            digitalWrite(context, 0, ledPin, color - 1); // note the -1 so that it's 0 or 1
            break;
        case 4: // mix (orange?)
            pinMode(context, 0, ledPin, OUTPUT);
           // mix the two colors in equal proportion
            for(unsigned int n = 0; n < context->digitalFrames; ++n)
                digitalWrite(context, 0, ledPin, n & 1); // this is low on even frames and high on odd frames
            break;
    }
}




// Render runs for each block of samples: https://learn.bela.io/using-bela/languages/c-plus-plus/#render
void render(BelaContext *context, void *userData)
{
	
	last_function = 886653;


  drivePwm(context, LED_PWM_PIN);

  ///////////////////////////////////////////
  // Look for Analogue Clock (24 PPQ)

	// Use this global variable as a timing device
	// Set other time points from this frame_timer
	frame_timer = context->audioFramesElapsed;


  // AUDIO LOOP
	for(unsigned int n = 0; n < context->audioFrames; n++) {
		
	
		
    

		
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
        
        // Get the delayed sample (by reading `total_delay_frames` many samples behind our current write pointer) and add it to our output sample
        out_l += gDelayBuffer_l[(gDelayBufWritePtr - total_delay_frames + DELAY_BUFFER_SIZE)%DELAY_BUFFER_SIZE] * gDelayAmount;
        out_r += gDelayBuffer_r[(gDelayBufWritePtr - total_delay_frames + DELAY_BUFFER_SIZE)%DELAY_BUFFER_SIZE] * gDelayAmount;
        
        // Write the sample into the output buffer -- done!
        // Apply "VCA" to the output.  



        audioWrite(context, n, 0, out_l);
        audioWrite(context, n, 1, out_r);
		    // End Bela delay example code
		    //////////////////////////////
		



	}

// ANALOG LOOP
	for(unsigned int n = 0; n < context->analogFrames; n++) {

		// Process analog oscillator	
	//	lfo_a_result_analog = lfo_a_analog.process();
	//	lfo_b_result_analog = lfo_b_analog.process();


	//  analog_per_sequence_adsr_a_level  = 1.0 * per_sequence_adsr_a.process();
	//	analog_per_sequence_adsr_b_level  = 1.0 * per_sequence_adsr_b.process();  
	//	analog_per_sequence_adsr_c_level  = 1.0 * per_sequence_adsr_c.process();
				
		// Modulated outputs
		//analog_out_2 = lfo_a_result_analog; // * analog_per_sequence_adsr_a_level;
		//analog_out_2 = per_sequence_adsr_a.process();
    	//analog_out_3 = analog_per_sequence_adsr_b_level; 
		//analog_out_4 = analog_per_sequence_adsr_c_level;

		
    	//analog_out_3 = lfo_a_result_analog * analog_per_sequence_adsr_b_level; 
		//analog_out_4 = lfo_a_result_analog * analog_per_sequence_adsr_c_level;
		
		
		
		

  //  analog_out_6 = lfo_b_result_analog * analog_per_sequence_adsr_a_level;
  //  analog_out_7 = lfo_b_result_analog * analog_per_sequence_adsr_b_level; 
  //  analog_out_8 = lfo_b_result_analog * analog_per_sequence_adsr_c_level;
		
		// ANALOG INPUTS
		for(unsigned int ch = 0; ch < gAnalogChannelNum; ch++){
			
	      // Sequence A INPUTS 		
		  if (ch == SEQUENCE_A_LENGTH_ANALOG_INPUT_PIN){
		  	sequence_a_length_input_raw = analogRead(context, n, SEQUENCE_A_LENGTH_ANALOG_INPUT_PIN);
        
        	// May be a hack to set two params from one knob.
        	coarse_delay_input = map(analogRead(context, n, SEQUENCE_A_PATTERN_ANALOG_INPUT_PIN), 0, 1, 0, MAX_COARSE_DELAY_TIME_INPUT);
		  }	
	    
	    if (ch == SEQUENCE_A_PATTERN_ANALOG_INPUT_PIN ){
	      	// note this is getting all the frames 
	        sequence_a_pattern_input_raw = analogRead(context, n, SEQUENCE_A_PATTERN_ANALOG_INPUT_PIN);
	        
	        //rt_printf("Set sequence_a_pattern_input_raw %d ", sequence_a_pattern_input_raw); 
	        //rt_printf("Set sequence_a_pattern_input_raw %f ", analogRead(context, n, SEQUENCE_A_PATTERN_ANALOG_INPUT_PIN)); 
	        //sequence_a_pattern_input = static_cast<double>(round(map(sequence_a_pattern_input_raw, 0.0, 1.0, 0.0, 255.0))); // GetValue(sequence_a_pattern_input_raw, sequence_a_pattern_input, jitter_reduction);
	    	  //rt_printf("**** NEW value for sequence_a_pattern_input is: %d ", sequence_a_pattern_input  );
	    }

    	// Sequence B INPUTS 
    	if (ch == SEQUENCE_B_LENGTH_ANALOG_INPUT_PIN){
 	      sequence_b_length_input_raw = analogRead(context, n, SEQUENCE_B_LENGTH_ANALOG_INPUT_PIN);
		 }	

		 if (ch == SEQUENCE_B_PATTERN_ANALOG_INPUT_PIN){
        	sequence_b_pattern_input_raw = analogRead(context, n, SEQUENCE_B_PATTERN_ANALOG_INPUT_PIN);
		 }



	    if (ch == LFO_OSC_1_FREQUENCY_INPUT_PIN){
	      	lfo_osc_1_frequency = map(analogRead(context, n, LFO_OSC_1_FREQUENCY_INPUT_PIN), 0, 1, 0.01, 1); // up to 1 Hz
		  }

	  if (ch == LFO_OSC_2_FREQUENCY_INPUT_PIN){
		  	// used for various ADSR settings
		  	//envelope_1_setting = map(analogRead(context, n, LFO_OSC_2_FREQUENCY_INPUT_PIN), 0, 1, 0.01, 8.0); // Up to 8 seconds (when multipled by sample rate)
		    	lfo_osc_2_frequency = map(analogRead(context, n, LFO_OSC_2_FREQUENCY_INPUT_PIN), 0, 1, 0.005, 1); // up to 1 Hz
      }
		    
		  
      // Changes the Midi Lane. 
		  if (ch == MIDI_LANE_INPUT_PIN){
		  	midi_lane_input = floor(map(analogRead(context, n, MIDI_LANE_INPUT_PIN), 0, 1, MIN_LANE, MAX_LANE));
		  	SetLane(midi_lane_input);
		  }
		  
		  if (ch == CLOCK_DIVIDER_INPUT_PIN){

        clock_divider_input_value = floor(map(analogRead(context, n, CLOCK_DIVIDER_INPUT_PIN), 0, 1, MIN_CLOCK_DIVIDER_SETTING, MAX_CLOCK_DIVIDER_SETTING));

// Now using this pot for clock devider

		        // // Increment draw buffer write pointer
		        // if(++draw_buf_write_pointer > DRAW_BUFFER_SIZE){
		        //     draw_buf_write_pointer = 0;
		        // }
		        
		        // // this might be set when we reset sequence (get to FIRST_STEP)
		        // if (need_to_reset_draw_buf_pointer == true){
		        // 	draw_buf_write_pointer = 0;
		        // 	need_to_reset_draw_buf_pointer = false;
		        // }
		
		        // // If button 3 is pressed, mirror the input to the output and write the value to the buffer for later use.
		        // if (new_button_3_state == 1){
		  		  //     analog_out_8 = analogRead(context,n,CLOCK_DIVIDER_INPUT_PIN);
		        //     draw_buffer[draw_buf_write_pointer] = analog_out_8;
		        // } else {
		        // 	// Else use the buffer value
		        //     // analog_out_8 = draw_buffer[(draw_buf_write_pointer - draw_total_frames + DRAW_BUFFER_SIZE) % DRAW_BUFFER_SIZE] * 1; // feedback gain is 1.
		        // 	analog_out_8 = draw_buffer[draw_buf_write_pointer];
		        	
		        	
		        // }     
		
				    // // Write output Draw Output
		        // analogWrite(context, n, SEQUENCE_CV_OUTPUT_8_PIN, analog_out_8);


		  }
		  
		  
		  
		  
	      
	      // ANALOG OUTPUTS
	      // CV 1 ** GATE ** 
	      if (ch == SEQUENCE_A_GATE_OUTPUT_1_PIN){
	      	if (target_analog_gate_a_out_state == HIGH){
	      		analog_out_1 = 1.0;
	      	} else {
	      		analog_out_1 = -1.0;	
	      	}
	      	analogWrite(context, n, ch, analog_out_1);
	      }

	      // CV 2
	      if (ch == SEQUENCE_CV_OUTPUT_2_PIN){
	      	//rt_printf("amp is: %f", amp);
	      	//////
	      	

	      	analog_per_sequence_adsr_a_level = per_sequence_adsr_a.process();
	      	
	      	lfo_a_result_analog = lfo_a_analog.process();
          lfo_b_result_analog = lfo_b_analog.process();
	      	
	      	// Sum
	      	analog_out_2 = (lfo_a_result_analog + lfo_b_result_analog) / 2.0;
	 
	      	analogWrite(context, n, ch, analog_out_2);
	      }


	      // CV 3 - This is below the left hand LED so should be related to the Length of the Sequence (Simple decaying envelope)
	      if (ch == SEQUENCE_CV_OUTPUT_3_PIN){
	      	//rt_printf("amp is: %f", amp);
	      	
	      	
	      	// Difference 
	      	analog_out_3 = (lfo_a_result_analog - lfo_b_result_analog) / 2.0;
	      	
	      	
	      	
	      	analogWrite(context, n, ch, analog_out_3);
	      }
	      
	      // CV 4
	      if (ch == SEQUENCE_CV_OUTPUT_4_PIN){
	      	//rt_printf("amp is: %f", amp);
	      	
	      	 analog_per_sequence_adsr_b_level = per_sequence_adsr_b.process();
	      	
	      	// Produect
	      	analog_out_4 = lfo_a_result_analog * lfo_b_result_analog;
	      	
	      	analogWrite(context, n, ch, analog_out_4);
	      }
	      
	      if (ch == SEQUENCE_B_GATE_OUTPUT_5_PIN){
	      	if (target_analog_gate_b_out_state == HIGH){
	      		analog_out_5 = 1.0;
	      	} else {
	      		analog_out_5 = -1.0;	
	      	}
	      	analogWrite(context, n, ch, analog_out_5);
	      }



	      // CV 6
	      if (ch == SEQUENCE_CV_OUTPUT_6_PIN){
	      	//rt_printf("amp is: %f", amp);
	      	

	      	
	      	lfo_b_result_analog = lfo_b_analog.process();
	      	
	      	analog_out_6 = (lfo_a_result_analog - lfo_b_result_analog) * analog_per_sequence_adsr_b_level;
	      	
	      	
	      	analogWrite(context, n, ch, analog_out_6);
	      }


	      // CV 7 
	      if (ch == SEQUENCE_CV_OUTPUT_7_PIN){
	      	//rt_printf("amp is: %f", amp);
	      	
	      	
	      	 analog_per_sequence_adsr_c_level = per_sequence_adsr_c.process();
	      	
	      	analog_out_7 = (lfo_a_result_analog + lfo_b_result_analog) * analog_per_sequence_adsr_c_level / 3.0;
	      	
	      	
	      	analogWrite(context, n, ch, analog_out_7);
	      }
	      
	      // CV 8 -- See above, Draw CV
	      //if (ch == SEQUENCE_CV_OUTPUT_8_PIN){
	      //	//rt_printf("amp is: %f", amp);
	      //	analogWrite(context, n, ch, analog_out_8);
	      //}

 
	      scope.log(analog_out_1, analog_out_2, analog_out_3, analog_out_4);

		}
	}
	

	// How often does this run? 
	Bela_scheduleAuxiliaryTask(gChangeSequenceTask);
	
	
	// DIGITAL LOOP 
		// Looking at all frames in case the transition happens in these frames. However, as its a clock we could maybe look at only the first frame.
	    for(unsigned int m = 0; m < context->digitalFrames; m++) {
	    	
	    	// we call this often but it only acts at the start
	    	FlashHello();
	    
        	// Next state
        	new_digital_clock_in_state = digitalRead(context, m, CLOCK_INPUT_DIGITAL_PIN);
        	
        	new_reset_a_in_state = digitalRead(context, m, RESET_A_INPUT_DIGITAL_PIN);
        	new_reset_b_in_state = digitalRead(context, m, RESET_B_INPUT_DIGITAL_PIN);
        	
        	
        	old_button_1_state = new_button_1_state;
        	new_button_1_state = digitalRead(context, m, button_1_PIN);

        	old_button_2_state = new_button_2_state;
        	new_button_2_state = digitalRead(context, m, button_2_PIN);

        	old_button_3_state = new_button_3_state;
        	new_button_3_state = digitalRead(context, m, button_3_PIN);

        	old_button_4_state = new_button_4_state;
        	new_button_4_state = digitalRead(context, m, button_4_PIN);

        	
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
        	
        	        	
  
        	// Only set new state if target is changed
        	if (target_digital_gate_a_out_state != current_digital_gate_a_out_state){
        		// 0 to 3.3V ? Salt docs says its 0 to 5 V (Eurorack trigger voltage is 0 - 5V)
	        	digitalWrite(context, m, SEQUENCE_A_DIGITAL_OUT_PIN, target_digital_gate_a_out_state);
	        	current_digital_gate_a_out_state = target_digital_gate_a_out_state;
        	}

        	if (target_digital_gate_b_out_state != current_digital_gate_b_out_state){
        		// 0 to 3.3V ? Salt docs says its 0 to 5 V (Eurorack trigger voltage is 0 - 5V)
	        	digitalWrite(context, m, SEQUENCE_B_DIGITAL_OUT_PIN, target_digital_gate_b_out_state);
	        	current_digital_gate_b_out_state = target_digital_gate_b_out_state;
        	}



          // Drive the LEDS. See https://github.com/BelaPlatform/Bela/wiki/Salt#led-and-pwm
          // Also set by flash

          setLed(context, LED_1_PIN, target_led_1_tri_state);
          setLed(context, LED_2_PIN, target_led_2_tri_state);
          setLed(context, LED_3_PIN, target_led_3_tri_state);
          setLed(context, LED_4_PIN, target_led_4_tri_state);

        
            // CLOCK RISE
            // If detect a rising clock edge
            if ((new_digital_clock_in_state == HIGH) && (current_digital_clock_in_state == LOW)){
              
              current_digital_clock_in_state = HIGH;
   
              if (sequence_is_running == LOW){
                StartSequencer();
              }
               
              OnTick();
              
            } 
            
            
            // CLOCK FALL
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
            
            
            // RESET A RISE
            if ((new_reset_a_in_state == HIGH) && (current_reset_a_in_state == LOW)){
              current_reset_a_in_state = HIGH;
              // Do reset A
              ResetSequenceACounters();
              // show it
              target_led_2_tri_state = 2;
            }
            
            // RESET A FALL
            if ((new_reset_a_in_state == LOW) && (current_reset_a_in_state == HIGH)){
              current_reset_a_in_state = LOW;
              // show off
              target_led_2_tri_state = 0;
            }
            
            // RESET B RISE
            if ((new_reset_b_in_state == HIGH) && (current_reset_b_in_state == LOW)){
              current_reset_b_in_state = HIGH;
              // Do reset B
              ResetSequenceBCounters();
              // show it
              target_led_4_tri_state = 2;
            }
            
            // RESET B FALL
            if ((new_reset_b_in_state == LOW) && (current_reset_b_in_state == HIGH)){
              current_reset_b_in_state = LOW;
              // show off
              target_led_4_tri_state = 0;
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
			
			
			if (elapsed_frames_since_last_tick > frames_per_24_ticks * 10){
		  		if (sequence_is_running == LOW) {
		    		// TODO make this a low prioity task

			    	
			    	if (need_to_auto_save_sequence == true){
			    		rt_printf("Saving Sequence to Files because elapsed_frames_since_last_tick: %llu is greater than frames_per_24_ticks * 10 : %llu \n", elapsed_frames_since_last_tick, frames_per_24_ticks);
						  Bela_scheduleAuxiliaryTask(gWriteSequenceToFiles);
						  need_to_auto_save_sequence = false;
					  }
			    	
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
	last_function = 588773;
	
	Bela_scheduleAuxiliaryTask(gAllNotesOff);
}


// BELA LGPL
// This is a lower-priority call to update the frequencies which will happen
// periodically when the analog inputs are enabled. By placing it at a lower priority,
// it has minimal effect on the audio performance but it will take longer to
// complete if the system is under heavy audio load.
void recalculate_frequencies(void*)
{
		last_function = 628497;
	
        float freq = gNewMinFrequency;
        float increment = (gNewMaxFrequency - gNewMinFrequency) / (float)gNumOscillators;
        for(int n = 0; n < gNumOscillators; n++) {
                // Update the frequencies to a regular spread, plus a small amount of randomness
                // to avoid weird phase effects
                float randScale = 0.99 + .02 * (float)random() / (float)RAND_MAX;
                float newFreq = freq * randScale;
                osc_bank.setFrequency(n, newFreq);
                freq += increment;
        }
}





  