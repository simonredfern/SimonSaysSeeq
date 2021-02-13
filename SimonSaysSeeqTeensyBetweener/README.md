# SimonSaysSeeq for the Betweener

# Instructions for compiling (if you need to)

The recommended place (according to Arduino https://www.arduino.cc/en/guide/libraries) to put libraries is in your own "Sketchbook Location" e.g. /your-home/Local/Documents/Arduino

This document kind of assumes you are using the Teensyduino flavour of the Arduino IDE. Teensydunino probably comes with some libraries already installed e.g. Audio and Midi and possibly Bounce2.

See / Set the sketchbook location using the Arduino / Teensyduino IDE -> Preferences -> Settings -> Sketchbook Locaiton.

You can copy the contents of the (unusual) libraries e.g. Betweener and whatever that uses in libraries folder of this project to your sketchbook location. 

Then *see the important note about DODINMIDI below* and then try and compile. Good luck!


Note: To add other unusual libraries you'll do via zip files in the IDE: Goto Sketch -> Add Library -> Add .ZIP Library and choose the Betweener.zip file in the same folder as this file. This will probably create a data folder with the same file in it.



// Note: We want to use https://github.com/FortySevenEffects/arduino_midi_library
// However even with (a recent verion of) this library we get compile errors, so we comment out the offending MIDI over DIN in Betweener.cpp
// NOTE In case of compile errors with Betweener and the MIDI library used,

Important! 
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



// Note: It can take 2-3 mins for the device to come back up after uploading!!

//****NOTE: Because the Betweener library uses USB midi, you must set up the Arduino IDE to expect USB MIDI even if you don’t use it in your sketch.
// To do this, you simply go to the main menu bar in Arduino and select Tools->USB Type->[any that includes MIDI]. I usually select “Serial + MIDI”. If you do not see this option on the menu, make sure that you have selected “Teensy 3.1/3.2 from the Tools->Board menu.
// Also Turn off DODINMIDI in Betweener.h (see below)



// We use : https://github.com/PaulStoffregen/Audio
// See https://www.pjrc.com/teensy/gui/index.html

// Betweener library https://github.com/jkrame1/Betweener/wiki/03_Library-Reference

// Libraries seem to be in /Users/simonredfern/Documents/Arduino/libraries/


// Sketchbook location is: /Users/simonredfern/Documents/Arduino



// 1) Install Arduino version 1.8.13
// 2) Then apply the Teensyduino, Version 1.53 dmg to the Arduino installation.
// 3) Then put Audio 1.03 from 2015 // https://github.com/PaulStoffregen/Audio/releases/tag/v1.03 into Documents/libraries.
// These are in Documents/libraries
// Audio-1.03
// MIDIUSB
// readme.txt
// Betweener
// MIDI_Library
// Bounce2
// ResponsiveAnalogRead
// 4) Set Tools Board to Teensy 3.2 (else it won't compile!)
// 5) Set Tools USB type to MIDI 

# Instructions for running

Use the Tools -> Serial Monitor to view the debug messages.







