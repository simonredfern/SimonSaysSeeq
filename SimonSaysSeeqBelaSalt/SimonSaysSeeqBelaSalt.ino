#include <Bela.h>
#include <libraries/Midi/Midi.h>
#include <stdlib.h>
#include <cmath>


#include <chrono>

// ...

using namespace std::chrono;
milliseconds ms = duration_cast< milliseconds >(
    system_clock::now().time_since_epoch()
);

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



float analogue_clock_level;
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

  bool reset_cv_lfo_at_FIRST_STEP = false;

// Amplitude of the LFO
unsigned int cv_waveform_a_amplitude_raw;
float cv_waveform_a_amplitude;




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



bool midi_clock_detected = LOW;


milliseconds last_clock_pulse=milliseconds();

bool analogue_gate_state = LOW;

bool sequence_is_running = LOW;




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
int gSampleCount = 44100; // how often to send out a control change


float gPhase;
float gInverseSampleRate;
int gAudioFramesPerAnalogFrame = 0;

// Set the analog channels to read from
//int gSensorInputFrequency = 0;
const int CLOCK_INPUT_ANALOGUE_IN_PIN = 0;





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
	}
}



//////////////////////////////////////////////////



bool setup(BelaContext *context, void *userData)
{
	
	midi.readFrom(gMidiPort0);
	midi.writeTo(gMidiPort0);
	midi.enableParser(false);
	midi.setParserCallback(midiMessageCallback, (void*) gMidiPort0);
	gSamplingPeriod = 1 / context->audioSampleRate;
	
	
	
	
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

	
	//rt_printf("Hello \n");
	// one way of getting the midi data is to parse them yourself
//	(you should set midi.enableParser(false) above):

	static midi_byte_t noteOnStatus = 0x90; //on channel 1
	static int noteNumber = 0;
	static int waitingFor = kNoteOn;
	static int playingNote = -1;
	int message;
	
	
	
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
	
	
	/// SimonSaysSeeq first bits_2_1
	
	
// Analog Clock (and left input checking) //////


    ///////////////////////////////////////////
   // Look for Analogue Clock (24 PPQ)
   // Note: We use this input for other things too.
   //if (peak_L.available()){
   
   
   	for(unsigned int n = 0; n < context->audioFrames; n++) {
		if(gAudioFramesPerAnalogFrame && !(n % gAudioFramesPerAnalogFrame)) {
			// read analog inputs and update frequency and amplitude
			// Depending on the sampling rate of the analog inputs, this will
			// happen every audio frame (if it is 44100)
			// or every two audio frames (if it is 22050)
			//frequency = map(analogRead(context, n/gAudioFramesPerAnalogFrame, gSensorInputFrequency), 0, 1, 100, 1000);
			//amplitude = analogRead(context, n/gAudioFramesPerAnalogFrame, CLOCK_INPUT_ANALOGUE_IN_PIN);
			
			
			// This function returns the value of an analog input, at the time indicated by frame. 
			// The returned value ranges from 0 to 1, corresponding to a voltage range of 0 to 4.096V.
			 analogue_clock_level = analogRead(context, n/gAudioFramesPerAnalogFrame, CLOCK_INPUT_ANALOGUE_IN_PIN);
			 
			 
			 //rt_printf("analogue_clock_level is: %f \n", analogue_clock_level);
			
		}

		// float out = amplitude * sinf(gPhase);

		// for(unsigned int channel = 0; channel < context->audioOutChannels; channel++) {
		// 	audioWrite(context, n, channel, out);
		// }

		// Update and wrap phase of sine tone
		// gPhase += 2.0f * (float)M_PI * frequency * gInverseSampleRate;
		// if(gPhase > M_PI)
		// 	gPhase -= 2.0f * (float)M_PI;
			
			
			
			
		        // 1.0; // peak_L.read() * 1.0; // minimum seems to be 0.1 from intelij attenuator
        // Serial.println(String("**** analogue_clock_level: ") + analogue_clock_level) ;

        //Serial.println(String("analogue_gate_state: ") + analogue_gate_state) ;

         // ONLY if no MIDI clock, run the sequencer from the Analogue clock.
        if (midi_clock_detected == LOW) {

          //Serial.println(String(">>>>>NO<<<<<<< Midi Clock Detected midi_clock_detected is: ") + midi_clock_detected) ;
          
          // Only look for this clock if we don't have midi.

            // Rising clock edge? // state-change-1
            if ((analogue_clock_level > 0.5) && (analogue_gate_state == LOW)){
    
              if (sequence_is_running == LOW){
              	// TODO
              	 rt_printf("would StartSequencer because: %f \n", analogue_clock_level);
              	
                //StartSequencer();
              }
              
              analogue_gate_state = HIGH;
              
              rt_printf("Set analogue_gate_state HIGH because: %f \n", analogue_clock_level);
              
              //Serial.println(String("Went HIGH "));
              
              
              // TODO
              //OnTick();
              last_clock_pulse = milliseconds();
              
            } 
    
            // Falling clock edge?
            if ((analogue_clock_level < 0.5) && (analogue_gate_state == HIGH)){
              analogue_gate_state = LOW;
              
              rt_printf("I set analogue_gate_state LOW because: %f \n", analogue_clock_level);
              
              
              //Serial.println(String("Went LOW "));
            } 

        } // 	
			
			
			
			
			
	}
   
   
   
   
   
   
   

        

	
	
	
	
	
	////


	
/////
}

void cleanup(BelaContext *context, void *userData)
{

}