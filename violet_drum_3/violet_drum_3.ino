/*
 * MIDIUSB_test.ino
 *
 * Created: 4/6/2015 10:47:08 AM
 * Author: gurbrinder grewal
 * Modified by Arduino LLC (2015)
 */ 

#include "MIDIUSB.h"

#define THRESHOLD 400 // Value that drum triggers at
#define EXCLUDED 100 // Time ms to ignore the signal (debounce)

#define DRUM_CHANNEL 2 // MIDI channel
#define NOTE_VELOCITY 90 // Default velocity

#define MAXDRUMS 5 // Number of drums

// Debugging modes
//#define MONITOR
//#define AUTO
int excluded[MAXDRUMS];
unsigned long timeon[MAXDRUMS];

// MIDI notes for each 
uint8_t drums[MAXDRUMS] = {
  60, // C3 - A0
  61,
  62,
  63,
  64
};



int sensorPin;

// First parameter is the event type (0x09 = note on, 0x08 = note off).
// Second parameter is note-on/note-off, combined with the channel.
// Channel can be anything between 0-15. Typically reported to the user as 1-16.
// Third parameter is the note number (48 = middle C).
// Fourth parameter is the velocity (64 = normal, 127 = fastest).

void noteOn(byte channel, byte pitch, byte velocity) {
  digitalWrite(LED_BUILTIN, 1);
  midiEventPacket_t noteOn = {0x09, 0x90 | channel, pitch, velocity};
  MidiUSB.sendMIDI(noteOn);
}

void noteOff(byte channel, byte pitch, byte velocity) {
  digitalWrite(LED_BUILTIN, 0);
  midiEventPacket_t noteOff = {0x08, 0x80 | channel, pitch, velocity};
  MidiUSB.sendMIDI(noteOff);
}

void setup() {
  sensorPin = 0;

  pinMode(LED_BUILTIN, OUTPUT);

  Serial.begin(115200);

  for (int i = 0; i < MAXDRUMS; i++){
    timeon[i] = 0;
  }
}

// First parameter is the event type (0x0B = control change).
// Second parameter is the event type, combined with the channel.
// Third parameter is the control number number (0-119).
// Fourth parameter is the control value (0-127).

void controlChange(byte channel, byte control, byte value) {
  midiEventPacket_t event = {0x0B, 0xB0 | channel, control, value};
  MidiUSB.sendMIDI(event);
}

void sense(){
  // main sensing routine
  int value = analogRead(A0 + sensorPin);

  #ifdef AUTO
    noteOn(DRUM_CHANNEL, 60, 127);
    delay(500);
    noteOff(DRUM_CHANNEL, 60, 127);
    delay(500);
  #endif

  #ifdef MONITOR
    Serial.println("Monitor");
  #endif

  // Check status of all sensors
  for (int sensor = 0; sensor < MAXDRUMS; sensor++){
    if (timeon[sensor] != 0){
      // drum is active
      if (millis() > timeon[sensor] + EXCLUDED){
        // drum has been active for N seconds, send note off
        #ifndef MONITOR
          noteOff(DRUM_CHANNEL, drums[sensor], NOTE_VELOCITY);
        #endif
        timeon[sensor] = 0;
      }
    }
    else{
      // DRUM IS INACTIVE
      #ifdef MONITOR
        Serial.println(value);
      #endif

      // 
      if (sensor == sensorPin){
        if (value > THRESHOLD){
          // sensor is past threshold, turn note on
          timeon[sensor] = millis();
          #ifndef MONITOR
            noteOn(DRUM_CHANNEL, drums[sensor], NOTE_VELOCITY);
          #endif
        }
      }
    }
    

    
    }

    sensorPin = (sensorPin + 1) % MAXDRUMS;

}

void loop() {
  sense();
}