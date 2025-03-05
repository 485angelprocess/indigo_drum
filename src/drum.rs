use core::array;

use crate::midi_device::{MidiReadDevice, UsbMidiController};

use midi_convert::midi_types::{Note, Value7};
use rp_pico::hal::Timer;
use usb_device::bus::UsbBus;


const MAX_PADS: usize = 5;

pub struct Pad{
    pub note: Note,
    pub note_release: Note,
    pub velocity: Value7,
    pub assigned: bool,
    pub active: bool,
    pub len: u64,
    pub timer: u64
}

impl Pad{
    pub fn new() -> Self{
        Self{
            note: Note::A4,
            note_release: Note::A4,
            velocity: Value7::from(0),
            assigned: false,
            active: false,
            len: 100,
            timer: 0
        }
    }
}

pub struct DrumController<'a, B: UsbBus>{
    pad: [Pad; MAX_PADS],
    timer: &'a Timer,
    device:UsbMidiController<'a, B>
}

impl<'a, B> DrumController<'a, B>
    where B: UsbBus
{
    pub fn new(timer: &'a Timer, device: UsbMidiController<'a, B>) -> Self{
        Self{
            pad: array::from_fn(|_| Pad::new()),
            timer: timer,
            device: device
        }
    }
    
    pub fn pad(&mut self, index: usize) -> &mut Pad{
        &mut self.pad[index]
    }
    
    pub fn assign(&mut self, index: usize, note: Note){
        /* Set a note for a pad and flag as active */
        self.pad[index].note = note;
        self.pad[index].assigned = true;
    }
    
    pub fn unassign(&mut self, index: usize){
        /* Flag pad as inactive */
        self.pad[index].assigned = false;
    }
    
    pub fn trigger(&mut self, index: usize, velocity: Value7){
        /* Send note on from assigned pad */
        if self.pad[index].active{
            // Prevent notes from hanging on
            if self.pad[index].note != self.pad[index].note_release{
                self.release(index);
            }
        }
        if self.pad[index].assigned{
            self.device.set_note(self.pad[index].note);
            self.device.set_velocity(velocity);
            
            // Cache to make sure note off matches (checked by some plugins)
            self.pad[index].note_release = self.pad[index].note;
            self.pad[index].velocity = velocity;
            
            self.device.note_on();
            self.pad[index].active = true;
            
            self.schedule_release(index);
        }
    }
    
    pub fn release(&mut self, index: usize){
        /* Send note off from already triggered pad */
        self.device.set_note(self.pad[index].note_release);
        self.device.set_velocity(self.pad[index].velocity);
        self.device.note_off();
        self.pad[index].active = false;
    }
    
    pub fn schedule_release(&mut self, index: usize){
        /* Set a timer to release a pad trigger */
        let now = self.timer.get_counter().ticks();
        self.pad[index].timer = now + self.pad[index].len;
    }
    
    fn release_ready(&self, index: usize, now: u64) -> bool{
        /* Returns true if release is ready */
        self.pad[index].active && now >= self.pad[index].timer
    }
    
    pub fn poll(&mut self){
        /* Run every loop 
            Checks if triggers are done,
            Reads in midi
        */
        // TODO get midi note
        self.device.poll();
        
        let now = self.timer.get_counter().ticks();
        for i in 0..MAX_PADS{
            if self.release_ready(i, now){
                self.release(i);
            }
        }
    }
}