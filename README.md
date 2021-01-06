# SimonSaysSeeq
Simon Says Seeq is a simple and hopefully modeless gate sequencer with associated CV envelope generator (and midi looper) that is designed to drive pitch (or other modulation) most likely via a pitch quantizer.

This software (in various forms) is designed to run using Teensy based hardware and more recently on Bela Salt too.

 
Platform                  Status


* 1010Music Euroshield.     - Works (however this module is discontinued by 1010Music)

* Betweener                 - Works but currently the tool chain is broken so I can't compile

* Bela Salt                 - Just starting to work. 

## Simon Says Gray Code Seeq on 1010Music Euroshield

[About Euroshield](https://1010music.com/euroshield-user-guide)

### Upper Pot

Change the gate sequence.

[Gray Code](https://en.wikipedia.org/wiki/Gray_code) is used to provide a gradually changing set of bits (In a Gray Code only one bit or so changes as we count up).
It's slightly modified so that:

Fully clockwise is all ones (dense). Fully counter clockwise is sparse (one step is on).

### Upper Pot whilst button is pressed 

Change the sequence length.

Fully clockwise : Length is 1.
Fully counter clockwise : Length is 16.

The resulting sequence drives Gate Out.

### Lower Pot 

Frequency of Waveform A

### Lower Pot whilst button pressed

Frequency of Waveform B *multiplexed* with Amplitude of Waveform A

Waveform A is multiplied by Waveform B to produce CV Out.

### Midi Looper 

Connect your mini midi cables to the in and out.
Play notes (on your midi keyboard / controller) to loop over the sequence length as set above.
Play *quietly* to cancel notes.
Press the foot pedal to clear the whole midi loop.

### LEDs

LED 1 (top): Gate Out. Pulses on the Sequence.

LED 2: Pulses on first step. If length is 8 it's brighter.

LED 3: Sequence lengths of 16, 8 and 4 are highlighted with different brightness.

LED 4: Shows CV Out amplitude











