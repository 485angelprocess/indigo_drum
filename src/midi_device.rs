// Import necessary types from the usb-device crate
use defmt::*;

use usb_device::bus::{UsbBusAllocator, UsbBus};
use usb_device::prelude::*;
// use usbd_midi::data::usb_midi::constants::*;

use usbd_midi::{
    UsbMidiClass,
    UsbMidiPacketReader,
    UsbMidiEventPacket,
    CableNumber
};

use midi_convert::{midi_types::{Channel, MidiMessage, Note, Value7}, render_slice::MidiRenderSlice};

pub struct MidiMsg{
    pub channel: Channel,
    pub note: Note,
    pub velocity: Value7
}

pub enum PollResp{
    Ready,
    Pass
}

pub trait MidiReadDevice{
    fn poll(&mut self) -> PollResp{
        /* Returns PollResp::Ready is data is available */
        PollResp::Pass
    }
    fn read_data(&mut self, _buffer: &mut [u8], _size: &mut usize) -> PollResp{
        /* Reads data, return PollResp::Pass if data cannot be read*/
        PollResp::Pass
    }
    fn read(&mut self, buffer: &mut [u8], size: &mut usize) -> PollResp{
        /* Wrapper to check if data is ready and can be read */
        match self.poll(){
            PollResp::Pass => PollResp::Pass,
            PollResp::Ready => self.read_data(buffer, size)
        }
    }
    fn packet_reader<'b>(&mut self, buffer: &'b mut [u8; 64]) -> Option<UsbMidiPacketReader<'b>>{
        /* Returns a packet reader object which can be iterated over */
        let mut size: usize = 0;
         
        match self.read(buffer.as_mut_slice(), &mut size){
            PollResp::Ready => Some(UsbMidiPacketReader::new(buffer, size)),
            _ => None
        }
    }
}

impl Default for MidiMsg{
    fn default() -> Self{
        Self{
            channel: Channel::C1,
            note: Note::from(60),
            velocity: Value7::from(100)
        }
    }
}

pub struct UsbMidiController<'a, B>
where B: UsbBus
{
    midi: UsbMidiClass<'a, B>,
    usb_dev: UsbDevice<'a, B>,
    msg: MidiMsg
}

impl<'a, B> MidiReadDevice for UsbMidiController<'a, B> where B: UsbBus{
    fn poll(&mut self) -> PollResp{
        if self.usb_dev.poll(&mut [&mut self.midi]){
            PollResp::Ready
        }
        else{
            PollResp::Pass
        }
    }
    fn read_data(&mut self, buffer: &mut [u8], size: &mut usize) -> PollResp{
        if let Ok(s) = self.midi.read(buffer){
            *size = s;
            PollResp::Ready
        }
        else{
            PollResp::Pass
        }
    }
    
}

impl<'a, B> UsbMidiController<'a, B>
where B: UsbBus
{
    pub fn new(bus: &'a UsbBusAllocator<B>) -> Self where B: UsbBus{
        let descriptor = [StringDescriptors::default()
                            .manufacturer("Angel Process")
                            .product("MIDI Chord Drums")
                            .serial_number("12345678")];
        
        UsbMidiController{
            midi: UsbMidiClass::new(bus, 1, 1).unwrap(),
            usb_dev: UsbDeviceBuilder::new(bus, UsbVidPid(0x16C0, 0x5E4))
                        .device_class(0)
                        .device_sub_class(0)
                        .strings(&descriptor)
                        .unwrap()
                        .build(),
            msg: MidiMsg::default()
        }
    }
    
    pub fn set_channel(&mut self, channel: Channel) -> &mut Self{
        self.msg.channel = channel;
        self
    }
    
    pub fn set_note(&mut self, note: Note) -> &mut Self{
        self.msg.note = note;
        self
    }
    
    pub fn set_velocity(&mut self, velocity: Value7) -> &mut Self{
        self.msg.velocity = velocity;
        self
    }
    
    pub fn note_off(&mut self) -> &mut Self{
        let note_off = MidiMessage::NoteOff(self.msg.channel, self.msg.note, self.msg.velocity);
        self.send(note_off);
        self
    }
    
    pub fn note_on(&mut self) -> &mut Self{
        let note_on = MidiMessage::NoteOn(self.msg.channel, self.msg.note, self.msg.velocity);
        self.send(note_on);
        self
    }
    
    fn send(&mut self, msg: MidiMessage){
        let mut bytes = [0; 3];
        msg.render_slice(&mut bytes);
        let packet = UsbMidiEventPacket::try_from_payload_bytes(CableNumber::Cable0, &bytes).unwrap();
        let _result = self.midi.send_packet(packet);
    }
}