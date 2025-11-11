use crate::console::backspace;
use crate::{
    apic::{ioapic_read, ioapic_write},
    print, println,
};
const BUFFER_SIZE: usize = 256;
// PS/2 Scan Code Set 2 to USB HID mapping
static SCANCODE_TO_HID: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, // 0x00-0x07
    0, 0, 0, 0, 0, 0x2B, 0x35, 0, // 0x08-0x0F (Tab=0x0D->0x2B, Backtick=0x0E->0x35)
    0, 0xE2, 0xE1, 0, 0xE0, 0x14, 0x1E,
    0, // 0x10-0x17 (LAlt=0x11->0xE2, LShift=0x12->0xE1, LCtrl=0x14->0xE0, Q=0x15->0x14, 1=0x16->0x1E)
    0, 0, 0x1D, 0x16, 0x04, 0x1A, 0x1F,
    0, // 0x18-0x1F (Z=0x1A->0x1D, S=0x1B->0x16, A=0x1C->0x04, W=0x1D->0x1A, 2=0x1E->0x1F)
    0, 0x06, 0x1B, 0x07, 0x08, 0x21, 0x20,
    0, // 0x20-0x27 (C=0x21->0x06, X=0x22->0x1B, D=0x23->0x07, E=0x24->0x08, 4=0x25->0x21, 3=0x26->0x20)
    0, 0x2C, 0x19, 0x09, 0x17, 0x15, 0x22,
    0, // 0x28-0x2F (Space=0x29->0x2C, V=0x2A->0x19, F=0x2B->0x09, T=0x2C->0x17, R=0x2D->0x15, 5=0x2E->0x22)
    0, 0x11, 0x05, 0x0B, 0x0A, 0x1C, 0x23,
    0, // 0x30-0x37 (N=0x31->0x11, B=0x32->0x05, H=0x33->0x0B, G=0x34->0x0A, Y=0x35->0x1C, 6=0x36->0x23)
    0, 0, 0x10, 0x0D, 0x18, 0x24, 0x25,
    0, // 0x38-0x3F (M=0x3A->0x10, J=0x3B->0x0D, U=0x3C->0x18, 7=0x3D->0x24, 8=0x3E->0x25)
    0, 0x36, 0x0E, 0x0C, 0x12, 0x27, 0x26,
    0, // 0x40-0x47 (Comma=0x41->0x36, K=0x42->0x0E, I=0x43->0x0C, O=0x44->0x12, 0=0x45->0x27, 9=0x46->0x26)
    0, 0x37, 0x38, 0x0F, 0x33, 0x13, 0x2D,
    0, // 0x48-0x4F (Period=0x49->0x37, Slash=0x4A->0x38, L=0x4B->0x0F, Semicolon=0x4C->0x33, P=0x4D->0x13, Minus=0x4E->0x2D)
    0, 0, 0x34, 0, 0x2F, 0x2E, 0,
    0, // 0x50-0x57 (Apostrophe=0x52->0x34, LBracket=0x54->0x2F, Equals=0x55->0x2E)
    0x39, 0xE5, 0x28, 0x30, 0, 0x31, 0,
    0, // 0x58-0x5F (CapsLock=0x58->0x39, RShift=0x59->0xE5, Enter=0x5A->0x28, RBracket=0x5B->0x30, Backslash=0x5D->0x31)
    0, 0, 0, 0, 0, 0, 0x2A, 0, // 0x60-0x67 (Backspace=0x66->0x2A)
    0, 0, 0, 0, 0, 0, 0, 0, // 0x68-0x6F
    0, 0, 0, 0, 0, 0, 0x29, 0, // 0x70-0x77 (Escape=0x76->0x29)
    0, 0, 0, 0, 0, 0, 0, 0, // 0x78-0x7F
    0, 0, 0, 0, 0, 0, 0, 0, // 0x80-0x87
    0, 0, 0, 0, 0, 0, 0, 0, // 0x88-0x8F
    0, 0, 0, 0, 0, 0, 0, 0, // 0x90-0x97
    0, 0, 0, 0, 0, 0, 0, 0, // 0x98-0x9F
    0, 0, 0, 0, 0, 0, 0, 0, // 0xA0-0xA7
    0, 0, 0, 0, 0, 0, 0, 0, // 0xA8-0xAF
    0, 0, 0, 0, 0, 0, 0, 0, // 0xB0-0xB7
    0, 0, 0, 0, 0, 0, 0, 0, // 0xB8-0xBF
    0, 0, 0, 0, 0, 0, 0, 0, // 0xC0-0xC7
    0, 0, 0, 0, 0, 0, 0, 0, // 0xC8-0xCF
    0, 0, 0, 0, 0, 0, 0, 0, // 0xD0-0xD7
    0, 0, 0, 0, 0, 0, 0, 0, // 0xD8-0xDF
    0, 0, 0, 0, 0, 0, 0, 0, // 0xE0-0xE7
    0, 0, 0, 0, 0, 0, 0, 0, // 0xE8-0xEF
    0, 0, 0, 0, 0, 0, 0, 0, // 0xF0-0xF7
    0, 0, 0, 0, 0, 0, 0, 0, // 0xF8-0xFF
];

// Extended PS/2 Scan Code Set 2 (E0 prefix) to USB HID mapping
static SCANCODE_E0_TO_HID: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, // 0x00-0x07
    0, 0, 0, 0, 0, 0, 0, 0, // 0x08-0x0F
    0, 0xE6, 0, 0, 0xE4, 0, 0, 0, // 0x10-0x17 (RAlt=0x11->0xE6, RCtrl=0x14->0xE4)
    0, 0, 0, 0, 0, 0, 0, 0xE3, // 0x18-0x1F (LGui=0x1F->0xE3)
    0, 0, 0, 0, 0, 0, 0, 0xE7, // 0x20-0x27 (RGui=0x27->0xE7)
    0, 0, 0, 0, 0, 0, 0, 0x65, // 0x28-0x2F (Apps=0x2F->0x65)
    0, 0, 0, 0, 0, 0, 0, 0, // 0x30-0x37
    0, 0, 0, 0, 0, 0, 0, 0, // 0x38-0x3F
    0, 0, 0, 0, 0, 0, 0, 0, // 0x40-0x47
    0, 0, 0x54, 0, 0, 0, 0, 0, // 0x48-0x4F (Numpad/=0x4A->0x54)
    0, 0, 0, 0, 0, 0, 0, 0, // 0x50-0x57
    0, 0, 0x58, 0, 0, 0, 0, 0, // 0x58-0x5F (NumpadEnter=0x5A->0x58)
    0, 0, 0, 0, 0, 0, 0, 0, // 0x60-0x67
    0, 0x4D, 0, 0x50, 0x4A, 0, 0,
    0, // 0x68-0x6F (End=0x69->0x4D, Left=0x6B->0x50, Home=0x6C->0x4A)
    0x49, 0x4C, 0x51, 0, 0x4F, 0x52, 0,
    0, // 0x70-0x77 (Insert=0x70->0x49, Delete=0x71->0x4C, Down=0x72->0x51, Right=0x74->0x4F, Up=0x75->0x52)
    0, 0, 0, 0, 0x4E, 0, 0, 0x4B, // 0x78-0x7F (PgDn=0x7A->0x4E, PgUp=0x7D->0x4B)
    0, 0, 0, 0, 0, 0, 0, 0, // 0x80-0x87
    0, 0, 0, 0, 0, 0, 0, 0, // 0x88-0x8F
    0, 0, 0, 0, 0, 0, 0, 0, // 0x90-0x97
    0, 0, 0, 0, 0, 0, 0, 0, // 0x98-0x9F
    0, 0, 0, 0, 0, 0, 0, 0, // 0xA0-0xA7
    0, 0, 0, 0, 0, 0, 0, 0, // 0xA8-0xAF
    0, 0, 0, 0, 0, 0, 0, 0, // 0xB0-0xB7
    0, 0, 0, 0, 0, 0, 0, 0, // 0xB8-0xBF
    0, 0, 0, 0, 0, 0, 0, 0, // 0xC0-0xC7
    0, 0, 0, 0, 0, 0, 0, 0, // 0xC8-0xCF
    0, 0, 0, 0, 0, 0, 0, 0, // 0xD0-0xD7
    0, 0, 0, 0, 0, 0, 0, 0, // 0xD8-0xDF
    0, 0, 0, 0, 0, 0, 0, 0, // 0xE0-0xE7
    0, 0, 0, 0, 0, 0, 0, 0, // 0xE8-0xEF
    0, 0, 0, 0, 0, 0, 0, 0, // 0xF0-0xF7
    0, 0, 0, 0, 0, 0, 0, 0, // 0xF8-0xFF
];

// USB HID keycode to ASCII mapping (unshifted)
static HID_TO_ASCII: [u8; 256] = [
    0, 0, 0, 0, // 0x00-0x03
    b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', // 0x04-0x0B (a-h)
    b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', // 0x0C-0x13 (i-p)
    b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', // 0x14-0x1B (q-x)
    b'y', b'z', // 0x1C-0x1D (y-z)
    b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', // 0x1E-0x25 (1-8)
    b'9', b'0',    // 0x26-0x27 (9-0)
    b'\n',   // 0x28 (Enter)
    0x1B,    // 0x29 (Escape)
    b'\x08', // 0x2A (Backspace)
    b'\t',   // 0x2B (Tab)
    b' ',    // 0x2C (Space)
    b'-', b'=', // 0x2D-0x2E (- =)
    b'[', b']',  // 0x2F-0x30 ([ ])
    b'\\', // 0x31 (\)
    0,     // 0x32 (Non-US # and ~)
    b';',  // 0x33 (;)
    b'\'', // 0x34 (')
    b'`',  // 0x35 (`)
    b',', b'.', b'/', // 0x36-0x38 (, . /)
    0,    // 0x39 (Caps Lock)
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x3A-0x45 (F1-F12)
    0, 0, 0, // 0x46-0x48 (Print Screen, Scroll Lock, Pause)
    0, 0, 0, 0, 0, 0, // 0x49-0x4E (Insert, Home, PgUp, Del, End, PgDn)
    0, 0, 0, 0, // 0x4F-0x52 (Right, Left, Down, Up arrows)
    0, // 0x53 (Num Lock)
    b'/', b'*', b'-', b'+',  // 0x54-0x57 (Keypad / * - +)
    b'\n', // 0x58 (Keypad Enter)
    b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', // 0x59-0x60 (Keypad 1-8)
    b'9', b'0', // 0x61-0x62 (Keypad 9-0)
    b'.', // 0x63 (Keypad .)
    0,    // 0x64 (Non-US \ and |)
    0,    // 0x65 (Application)
    0,    // 0x66 (Power)
    b'=', // 0x67 (Keypad =)
    // Fill rest with zeros
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x68-0x77
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x78-0x87
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x88-0x97
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x98-0xA7
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0xA8-0xB7
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0xB8-0xC7
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0xC8-0xD7
    0, 0, 0, 0, 0, 0, 0, 0, // 0xD8-0xDF
    0, 0, 0, 0, 0, 0, 0, 0, // 0xE0-0xE7 (Modifier keys - handled separately)
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0xE8-0xF7
    0, 0, 0, 0, 0, 0, 0, 0, // 0xF8-0xFF
];

// USB HID keycode to ASCII mapping (shifted)
static HID_TO_ASCII_SHIFT: [u8; 256] = [
    0, 0, 0, 0, // 0x00-0x03
    b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', // 0x04-0x0B (A-H)
    b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P', // 0x0C-0x13 (I-P)
    b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', // 0x14-0x1B (Q-X)
    b'Y', b'Z', // 0x1C-0x1D (Y-Z)
    b'!', b'@', b'#', b'$', b'%', b'^', b'&', b'*', // 0x1E-0x25 (! @ # $ % ^ & *)
    b'(', b')',    // 0x26-0x27 (() ))
    b'\n',   // 0x28 (Enter)
    0x1B,    // 0x29 (Escape)
    b'\x08', // 0x2A (Backspace)
    b'\t',   // 0x2B (Tab)
    b' ',    // 0x2C (Space)
    b'_', b'+', // 0x2D-0x2E (_ +)
    b'{', b'}', // 0x2F-0x30 ({ })
    b'|', // 0x31 (|)
    0,    // 0x32 (Non-US # and ~)
    b':', // 0x33 (:)
    b'"', // 0x34 (")
    b'~', // 0x35 (~)
    b'<', b'>', b'?', // 0x36-0x38 (< > ?)
    0,    // 0x39 (Caps Lock)
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x3A-0x45 (F1-F12)
    0, 0, 0, // 0x46-0x48 (Print Screen, Scroll Lock, Pause)
    0, 0, 0, 0, 0, 0, // 0x49-0x4E (Insert, Home, PgUp, Del, End, PgDn)
    0, 0, 0, 0, // 0x4F-0x52 (Right, Left, Down, Up arrows)
    0, // 0x53 (Num Lock)
    b'/', b'*', b'-', b'+',  // 0x54-0x57 (Keypad / * - +)
    b'\n', // 0x58 (Keypad Enter)
    b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', // 0x59-0x60 (Keypad 1-8)
    b'9', b'0', // 0x61-0x62 (Keypad 9-0)
    b'.', // 0x63 (Keypad .)
    0,    // 0x64 (Non-US \ and |)
    0,    // 0x65 (Application)
    0,    // 0x66 (Power)
    b'=', // 0x67 (Keypad =)
    // Fill rest with zeros
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x68-0x77
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x78-0x87
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x88-0x97
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x98-0xA7
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0xA8-0xB7
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0xB8-0xC7
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0xC8-0xD7
    0, 0, 0, 0, 0, 0, 0, 0, // 0xD8-0xDF
    0, 0, 0, 0, 0, 0, 0, 0, // 0xE0-0xE7 (Modifier keys - handled separately)
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0xE8-0xF7
    0, 0, 0, 0, 0, 0, 0, 0, // 0xF8-0xFF
];

use spin::mutex::Mutex;
use x86_64::instructions::port::Port;
static KEYBOARD_BUFFER: Mutex<KeyboardBuffer> = Mutex::new(KeyboardBuffer::new());
pub static KEYBOARDKEY_STATE: Mutex<Option<KeyboardKeyState>> = Mutex::new(None);

pub struct Keyboard {
    data_port: Port<u8>,      //r/w
    status_registr: Port<u8>, //r
    command_port: Port<u8>,   //w
}
impl Keyboard {
    pub fn new() -> Self {
        Keyboard {
            data_port: Port::new(0x60),
            status_registr: Port::new(0x64),
            command_port: Port::new(0x64),
        }
    }
    pub fn init(&mut self) {
        unsafe {
            //disable PS/2
            self.command_port.write(0xAD);
            self.command_port.write(0xA7);
            //Flush out
            while self.status_registr.read() & 0x01 != 0 {
                self.data_port.read();
            }
            //Setup Controller byte
            self.command_port.write(0x20);
            self.poll_for_ouput();
            let byte = self.data_port.read();
            let mask = (1 << 0) | (1 << 4) | (1 << 6);
            self.command_port.write(0x60);
            self.poll_for_input();
            self.data_port.write(byte & (!mask));
            //self test
            self.command_port.write(0xAA);
            self.poll_for_ouput();
            let byte = self.data_port.read();
            if byte != 0x55 {
                panic!("[Error] Keyboard Failed to Init");
            }
            //detemine if dual channel
            self.command_port.write(0xA8);
            self.command_port.write(0x20);
            self.poll_for_ouput();
            let has_dual = (self.data_port.read() & (1 << 5)) >> 5;
            //perform interface test
            self.command_port.write(0xAB);
            self.poll_for_ouput();
            let byte = self.data_port.read();
            //enable devices
            self.command_port.write(0xAE);
            self.command_port.write(0x20);
            self.poll_for_ouput();
            let new_byte = (self.data_port.read() | 1);
            self.command_port.write(0x60);
            self.poll_for_input();
            self.data_port.write(new_byte);
            //reset devices
            self.poll_for_input();
            self.data_port.write(0xFF);
            self.poll_for_ouput();
            self.poll_for_ouput();
            //enable scanning
            self.poll_for_input();
            self.data_port.write(0xF4);
            self.poll_for_ouput();
            self.data_port.read();
            KeyboardKeyState::init();
            println!("[OK] Keyboard Driver is active")
        }
    }
    fn poll_for_ouput(&mut self) {
        unsafe { while self.status_registr.read() & 0x01 == 0 {} }
    }
    fn poll_for_input(&mut self) {
        unsafe { while self.status_registr.read() & 0x02 != 0 {} }
    }
}
pub struct KeyboardKeyState {
    left_shift: bool,
    right_shift: bool,
    left_ctrl: bool,
    right_ctrl: bool,
    left_alt: bool,
    right_alt: bool,
    caps_lock: bool,
    num_lock: bool,
    extended: bool,
    break_next: bool,
}
impl KeyboardKeyState {
    pub fn new() -> Self {
        KeyboardKeyState {
            left_shift: false,
            right_shift: false,
            left_ctrl: false,
            right_ctrl: false,
            left_alt: false,
            right_alt: false,
            caps_lock: false,
            num_lock: false,
            extended: false,
            break_next: false,
        }
    }
    pub fn init() {
        let mut keyboard_state = Self::new();
        *KEYBOARDKEY_STATE.lock() = Some(keyboard_state);
    }
    pub fn shift_pressed(&self) -> bool {
        self.left_shift || self.right_shift
    }
    pub fn ctrl_pressed(&self) -> bool {
        self.left_ctrl || self.right_ctrl
    }
    pub fn alt_pressed(&self) -> bool {
        self.left_alt || self.right_alt
    }
}
pub fn handle_scancode(code: u8) {
    if let Some(ref mut state) = *KEYBOARDKEY_STATE.lock() {
        // PS/2 Set 2: 0xF0 indicates next scancode is a key release
        if code == 0xF0 {
            state.break_next = true;
            return;
        }

        // PS/2 Set 2: 0xE0 indicates extended scancode
        if code == 0xE0 {
            state.extended = true;
            return;
        }

        let is_released = state.break_next;

        let key_code = {
            if state.extended {
                SCANCODE_E0_TO_HID[code as usize]
            } else {
                SCANCODE_TO_HID[code as usize]
            }
        };
        match key_code {
            0xE1 => state.left_shift = !is_released,
            0xE5 => state.right_shift = !is_released,
            0xE0 => state.left_ctrl = !is_released,
            0xE4 => state.right_ctrl = !is_released,
            0xE2 => state.left_alt = !is_released,
            0xE6 => state.right_alt = !is_released,
            0x39 if !is_released => state.caps_lock = !state.caps_lock,
            _ => {
                if !is_released {
                    if let Some(ch) = keycode_to_char(key_code, &state) {
                        handle_key_press(ch)
                    }
                }
            }
        }
        state.extended = false;
        state.break_next = false;
    }
}

fn keycode_to_char(keycode: u8, state: &KeyboardKeyState) -> Option<char> {
    let shift_pressed = state.shift_pressed();
    let caps_lock = state.caps_lock;

    let ascii = if shift_pressed {
        HID_TO_ASCII_SHIFT[keycode as usize]
    } else {
        HID_TO_ASCII[keycode as usize]
    };

    if ascii == 0 {
        return None;
    }

    if keycode >= 0x04 && keycode <= 0x1D {
        let char = ascii as char;
        if caps_lock {
            if shift_pressed {
                return Some(char.to_ascii_lowercase());
            } else {
                return Some(char.to_ascii_uppercase());
            }
        }
    }

    Some(ascii as char)
}

fn handle_key_press(ch: char) {
    if ch == '\x08' {
        backspace();
        KEYBOARD_BUFFER.lock().pop();
        return;
    }
    KEYBOARD_BUFFER.lock().push(ch as u8);
    print!("{}", ch);
}
pub struct KeyboardBuffer {
    buffer: [u8; BUFFER_SIZE],
    read_pos: usize,
    write_pos: usize,
}
impl KeyboardBuffer {
    pub const fn new() -> Self {
        KeyboardBuffer {
            buffer: [0; BUFFER_SIZE],
            read_pos: 0,
            write_pos: 0,
        }
    }
    pub fn push(&mut self, ch: u8) -> bool {
        let next = (self.write_pos + 1) % BUFFER_SIZE;
        if next == self.read_pos {
            return false;
        }
        self.buffer[self.write_pos] = ch;
        self.write_pos = next;
        true
    }
    pub fn pop(&mut self) -> Option<u8> {
        if self.read_pos == self.write_pos {
            return None;
        }
        let ch = self.buffer[self.read_pos];
        self.read_pos = (self.read_pos + 1) % BUFFER_SIZE;
        Some(ch)
    }
}
pub fn read_key_from_buffer() -> Option<char> {
    KEYBOARD_BUFFER.lock().pop().map(|b| b as char)
}
pub fn read_line_from_buffer_into_buffer(buffer: &mut [u8]) -> usize {
    let mut i = 0;
    loop {
        if let Some(ch) = read_key_from_buffer() {
            if ch == '\n' {
                buffer[i] = b'\n';
                return i + 1;
            }
            if i < buffer.len() {
                buffer[i] = ch as u8;
                i += 1;
            }
        }
    }
}
