# SimonSaysSeeq

Simon Says Seeq is a *modeless* (one function per knob / button) **[Gray code](https://en.wikipedia.org/wiki/Gray_code) gate sequencer, triggered CV envelope generator / audio delay / VCA and midi looper**.

## Why? 

I wanted to create melodies and rhythms easily (but not randomly).

I wanted to loop midi easily (always on looping).

The gate / CV outputs were created to drive a pitch quantizers via attenuator/offsets and so create repeatable yet dynamic melodies.

## Design Principles

Modeless: A knob only controls one thing. *Holding* a button and turning a knob is just about "ok" for a different function, but pressing a button to create a different mode is not "ok". See the late (Larry Tesler's)[https://en.wikipedia.org/wiki/Larry_Tesler] work on NOMODES.

## Platforms                  

The software (in various forms) runs on (Teensy)[https://www.pjrc.com/store/teensy32.html] based hardware and also [Bela Salt](https://learn.bela.io/products/modular/salt/) (embedded Linux).

All the platforms have the Gray code sequencer and CV envelop outs. Some have midi looping and some have audio looping / VCA.

* [1010Music Euroshield](https://github.com/simonredfern/SimonSaysSeeq/tree/master/SimonSaysSeeqTeensyEuroshieldV0.1) - (note this module is discontinued by 1010Music)

* [Betweener](https://github.com/simonredfern/SimonSaysSeeq/tree/master/SimonSaysSeeqTeensyBetweener)

* [Bela Salt](https://github.com/simonredfern/SimonSaysSeeq/tree/master/SimonSaysSeeqBelaSalt)

See the README.md in each sub folder for more details





