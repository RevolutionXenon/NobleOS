// PHOTON
// Photon is the Noble text graphics library:
// Draw to a frame buffer who's size decided at compile-time
// Character struct which holds a codepoint and color
// Screen struct which holds a frame buffer location and backlogs of text


// HEADER
//Flags
#![no_std]
#![feature(const_generics)]
#![feature(const_evaluatable_checked)]

//Modules
pub mod font_handler;

//Imports
use core::fmt::Write;
use core::fmt::Result;
use core::fmt::Arguments;
use core::ptr::write_volatile;
use crate::font_handler::retrieve_font_bitmap;

//Constants
pub const PHOTON_VERSION:      & str                   = "vDEV-2021-08-13";          //CURRENT VERSION OF GRAPHICS LIBRARY


// STRUCTS
//Character
#[derive(Copy, Clone)]
pub struct Character<const PIXEL_WIDTH: usize> {
    codepoint:      char,
    foreground:     [u8; PIXEL_WIDTH],
    background:     [u8; PIXEL_WIDTH],
}
impl<const PIXEL_WIDTH: usize> Character<PIXEL_WIDTH> {
    // CONSTRUCTOR
    pub fn new(codepoint: char, foreground: [u8;PIXEL_WIDTH], background:  [u8;PIXEL_WIDTH]) -> Self {
        Character {
            codepoint,
            foreground,
            background,
        }
    }
}

//Screen
pub struct Screen<
    //Screen Constants
    const SCREEN_H: usize,
    const SCREEN_W: usize,
    const SCREEN_P: usize,
    const CHARFR_H: usize,
    const CHARFR_W: usize,
    //Print Result Window Constants
    const PRINTW_M: usize,
    const PRINTW_H: usize,
    const PRINTW_W: usize,
    const PRINTW_Y: usize,
    const PRINTW_X: usize,
    //Input Window Constants
    const INPUTW_M: usize,
    const INPUTW_H: usize,
    const INPUTW_W: usize,
    const INPUTW_Y: usize,
    const INPUTW_X: usize,
    //Where Bound
    > where [(); CHARFR_H*CHARFR_W]: Sized, [(); PRINTW_M*PRINTW_W]: Sized, [(); INPUTW_M*INPUTW_W]: Sized {
    //Screen variables
    pub screen:       *mut u8,
    pub charframe:        [Character<SCREEN_P>; CHARFR_H*CHARFR_W],
    //Print Result Window Variables
    pub print_buffer:     [Character<SCREEN_P>; PRINTW_M*PRINTW_W],
    pub print_y:           usize,
    pub print_x:           usize,
    pub print_whitespace:  Character<SCREEN_P>,
    //Input Window Variables
    pub input_buffer:     [Character<SCREEN_P>; INPUTW_M*INPUTW_W],
    pub input_p:           usize,
}
impl<
    const SCREEN_H: usize, const SCREEN_W: usize, const SCREEN_P: usize, const CHARFR_H: usize, const CHARFR_W: usize, 
    const PRINTW_M: usize, const PRINTW_H: usize, const PRINTW_W: usize, const PRINTW_Y: usize, const PRINTW_X: usize,
    const INPUTW_M: usize, const INPUTW_H: usize, const INPUTW_W: usize, const INPUTW_Y: usize, const INPUTW_X: usize,
> Screen<
        SCREEN_H, SCREEN_W, SCREEN_P, CHARFR_H, CHARFR_W,
        PRINTW_M, PRINTW_H, PRINTW_W, PRINTW_Y, PRINTW_X,
        INPUTW_M, INPUTW_H, INPUTW_W, INPUTW_Y, INPUTW_X,
> where [(); CHARFR_H*CHARFR_W]:, [(); PRINTW_M*PRINTW_W]:, [(); INPUTW_M*INPUTW_W]: {
    // CONSTRUCTOR
    pub fn new(physical_pointer: *mut u8, whitespace: Character<SCREEN_P>) -> Self {
        Screen { 
            screen: physical_pointer,
            charframe: [whitespace; CHARFR_H*CHARFR_W],
            print_buffer: [whitespace; PRINTW_M*PRINTW_W],
            print_y: PRINTW_M - 1,
            print_x: 0,
            print_whitespace: whitespace,
            input_buffer: [whitespace; INPUTW_M*INPUTW_W],
            input_p: 0
        }
    }

    // BASIC FUNCTIONS
    //Render character to physical screen
    pub fn character_render(&mut self, character: Character<SCREEN_P>, y: usize, x: usize) {
        //Check valid character position
        if y >= CHARFR_H || x >= CHARFR_W {
            return;
        }
        //Find bitmap
        let bitmap = retrieve_font_bitmap(&character.codepoint);
        //Loop through bitmap
        for byte_row in 0..16 {
            for byte_column in 0..2 {
                for bit in 0..8 {
                    //Get position of of pixel in width*height space
                    let pixel_index = (y*16 + byte_row)*SCREEN_W + (x*16 + byte_column*8 + bit);
                    //Get color from bitmap bit and Character
                    let color: [u8; SCREEN_P] = 
                        if bitmap[byte_row*2 + byte_column] & (1 << bit) != 0{character.foreground}
                        else {character.background};
                    //Write color to screen
                    unsafe{
                        for i in 0..SCREEN_P {
                            write_volatile(
                                self.screen.add(pixel_index*SCREEN_P + i),
                                color[i]
                            );
                        }
                    }
                }
            }
        }
    }

    //Draw character to characterframe screen
    pub fn character_draw(&mut self, character: Character<SCREEN_P>, y: usize, x: usize) {
        //Check valid character position
        if y >= CHARFR_H || x >= CHARFR_W {
            return;
        }
        //Set character
        self.charframe[y*CHARFR_W + x] = character;
    }

    //Input character to input stack
    pub fn character_input(&mut self, character: Character<SCREEN_P>, whitespace: Character<SCREEN_P>) -> usize {
        let mut render:usize = 0;
        //control character booleans
        let control:bool = (character.codepoint as u32) < 0x20;                        //Determines if character is a control character
        let control_newline:bool = character.codepoint=='\n' || character.codepoint=='\r'; //Determines if character is a newline character
        let printable:bool = !control || control_newline;                          //Determines if a character is printable
        let control_backwards:bool = character.codepoint=='\x08';                      //Determines if character types backwards (i.e. backspace)
        let end_forward:bool = self.input_p >= self.input_buffer.len();           //Determines if the end of the stack is reached and position can no longer move forward
        let end_backward:bool = self.input_p <= 0;                                //Determines if the beginning of the stack is reached and position can no longer move backwards
        //add printable character to stack
        if printable && !end_forward {
            self.input_buffer[self.input_p] = character;
            render = self.input_p;
            self.input_p = self.input_p + 1;
        }
        //backspace handling
        else if control_backwards && !end_backward {
            self.input_p = self.input_p - 1;
            render = self.input_p;
            self.input_buffer[self.input_p] = whitespace;
        }
        return render;
    }

    //Input string to print buffer
    pub fn string_print(&mut self, string: &str, whitespace: Character<SCREEN_P>) -> bool {
        let mut render:bool = false;
        //begin writing to the buffer
        for codepoint in string.chars() {
            let control = (codepoint as u32) < 0x20;
            let control_newline = codepoint=='\n' || codepoint=='\r';
            let control_backwards = codepoint=='\x08';
            let end_forward = self.print_x >= PRINTW_W;
            let end_backward = self.print_x <= 0;
            //move to next line (line feed, carriage return, end of line moving forward)
            if (control_newline || end_forward) &! control_backwards {
                //reset xbuffer
                self.print_x=0;
                //move buffer up
                for y in 0..PRINTW_M-1 {
                    for x in 0..PRINTW_W {
                        self.print_buffer[y*PRINTW_W+x] = self.print_buffer[(y+1)*PRINTW_W+x];
                    }
                }
                //fill last line with spaces
                for x in 0..PRINTW_W {
                    self.print_buffer[(PRINTW_M-1)*PRINTW_W+x] = whitespace;
                }
                render = true;
            }
            //move to previous line (beginning of line moving backward)
            if control_backwards && end_backward {
                //reset xbuffer
                self.print_x = PRINTW_W;
                //move buffer down
                for y in (1..PRINTW_M).rev() {
                    for x in 0..PRINTW_W{
                        self.print_buffer[y*PRINTW_W+x] = self.print_buffer[(y-1)*PRINTW_W+x];
                    }
                }
                //fill first line with spaces
                for x in 0..PRINTW_W {
                    self.print_buffer[x] = whitespace;
                }
                render = true;
            }
            //unprint character (backspace)
            if control_backwards {
                //move xbuffer back one
                self.print_x = self.print_x - 1;
                //remove character
                self.print_buffer[(PRINTW_M - 1) * PRINTW_W + self.print_x] = whitespace;
            }
            //print character (not: line Feed, carriage return, backspace)
            if !control {
                //place character
                self.print_buffer[(PRINTW_M - 1) * PRINTW_W + self.print_x] = Character::new(codepoint, whitespace.foreground, whitespace.background);
                //move xbuffer right
                self.print_x = self.print_x + 1;
            }
        }
        return render;
    }


    // COMPOSITE FUNCTIONS
    //Render entire charframe
    pub fn characterframe_render(&mut self) {
        for y in 0..CHARFR_H {
            for x in 0..CHARFR_W {
                self.character_render(self.charframe[y*CHARFR_W + x], y, x);
            }
        }
    }

    //Draw character to characterframe screen and render it to physical screen
    pub fn character_draw_render(&mut self, character: Character<SCREEN_P>, y: usize, x: usize) {
        self.character_draw(character, y, x);
        self.character_render(character, y, x);
    }

    //Render entire inputstack
    pub fn inputstack_draw_render(&mut self) {
        for x in 0..INPUTW_W {
            self.character_draw_render(self.input_buffer[self.input_p/INPUTW_W+x], INPUTW_Y, INPUTW_X + x);
        }
    }

    //Render entire printbuffer
    pub fn printbuffer_draw_render(&mut self) {
        for y in 0..PRINTW_H {
            for x in 0..PRINTW_W {
                self.character_draw_render(self.print_buffer[(self.print_y + y) * PRINTW_W + x], y + PRINTW_Y, x + PRINTW_X);
            }
        }
    }

    //Input character to input stack, draw it to characterframe, and render it to physical screen
    pub fn character_input_draw_render(&mut self, character: Character<SCREEN_P>, whitespace: Character<SCREEN_P>) {
        let p: usize = self.character_input(character, whitespace);
        if p % INPUTW_W == 0 || p % INPUTW_W == INPUTW_W - 1 {
            self.inputstack_draw_render();
        }
        else {
            self.character_draw_render(self.input_buffer[p%INPUTW_W], INPUTW_Y, INPUTW_X + p);
        }
    }

    //Input string to print buffer, draw it to characterframe, and render it to physical screen
    pub fn string_print_draw_render(&mut self, character: &str, whitespace: Character<SCREEN_P>) {
        self.string_print(character, whitespace);
        self.printbuffer_draw_render();
    }

    //Remove all information from input stack and replace with Character given
    pub fn input_flush(&mut self, whitespace: Character<SCREEN_P>) {
        for i in 0..INPUTW_M*INPUTW_W {
            self.input_buffer[i] = whitespace;
        }
        self.input_p = 0;
        self.inputstack_draw_render();
    }

    //Return input characterstack as char array
    pub fn input_as_chararray(&mut self, blankpoint: char) -> [char;INPUTW_M*INPUTW_W] {
        let mut buffer: [char; INPUTW_M*INPUTW_W] = [blankpoint; INPUTW_M*INPUTW_W];
        for i in 0..INPUTW_M*INPUTW_W {
            buffer[i] = self.input_buffer[i].codepoint;
        }
        return buffer;
    }


    // DRAWING
    //Horizontal Line
    pub fn draw_hline(&mut self, y: usize, x1: usize, x2:usize, whitespace: Character<SCREEN_P>){
        {
            let pos: usize = y*CHARFR_W + x1;
            let check: char = self.charframe[pos].codepoint;
            let write = match check {
                '═' => '═', '║' => '╠', '╔' => '╔', '╗' => '╦',
                '╚' => '╚', '╝' => '╩', '╞' => '╞', '╠' => '╠',
                '╡' => '═', '╣' => '╬', '╥' => '╔', '╦' => '╦',
                '╨' => '╚', '╩' => '╩', '╬' => '╬',  _  => '╞',
            };
            self.charframe[pos] = Character::new(write, whitespace.foreground, whitespace.background);
        }
        for x in x1+1..x2 {
            let pos: usize = y*CHARFR_W + x;
            let check: char = self.charframe[pos].codepoint;
            let write = match check {
                '═' => '═', '║' => '╬', '╔' => '╦', '╗' => '╦',
                '╚' => '╩', '╝' => '╩', '╞' => '═', '╠' => '╬',
                '╡' => '═', '╣' => '╬', '╥' => '╦', '╦' => '╦',
                '╨' => '╩', '╩' => '╩', '╬' => '╬',  _  => '═',
            };
            self.charframe[pos] = Character::new(write, whitespace.foreground, whitespace.background);
        }
        {
            let pos: usize = y*CHARFR_W + x2;
            let check: char = self.charframe[pos].codepoint;
            let write = match check {
                '═' => '═', '║' => '╣', '╔' => '╦', '╗' => '╗',
                '╚' => '╩', '╝' => '╝', '╞' => '═', '╠' => '╬',
                '╡' => '╡', '╣' => '╣', '╥' => '╗', '╦' => '╦',
                '╨' => '╝', '╩' => '╩', '╬' => '╬',  _  => '╡',
            };
            self.charframe[pos] = Character::new(write, whitespace.foreground, whitespace.background);
        }
    }

    //Vertical Line
    pub fn draw_vline(&mut self, x: usize, y1: usize, y2:usize, whitespace: Character<SCREEN_P>) {
        {
            let pos: usize = y1*CHARFR_W + x;
            let check: char = self.charframe[pos].codepoint;
            let write = match check {
                '═' => '╦', '║' => '║', '╔' => '╔', '╗' => '╗',
                '╚' => '╠', '╝' => '╣', '╞' => '╔', '╠' => '╠',
                '╡' => '╗', '╣' => '╣', '╥' => '╥', '╦' => '╦', 
                '╨' => '║', '╩' => '╬', '╬' => '╬',  _  => '╥',
            };
            self.charframe[pos] = Character::new(write, whitespace.foreground, whitespace.background);
        }
        for y in y1+1..y2 {
            let pos: usize = y*CHARFR_W + x;
            let check: char = self.charframe[pos].codepoint;
            let write = match check {
                '═' => '╬', '║' => '║', '╔' => '╠', '╗' => '╣',
                '╚' => '╠', '╝' => '╣', '╞' => '╠', '╠' => '╠',
                '╡' => '╣', '╣' => '╣', '╥' => '║', '╦' => '╬',
                '╨' => '║', '╩' => '╬', '╬' => '╬',  _  => '║',
            };
            self.charframe[pos] = Character::new(write, whitespace.foreground, whitespace.background);
        }
        {
            let pos: usize = y2*CHARFR_W + x;
            let check: char = self.charframe[pos].codepoint;
            let write = match check {
                '═' => '╩', '║' => '║', '╔' => '╠', '╗' => '╣',
                '╚' => '╚', '╝' => '╝', '╞' => '╚', '╠' => '╠',
                '╡' => '╝', '╣' => '╣', '╥' => '║', '╦' => '╬',
                '╨' => '╨', '╩' => '╩', '╬' => '╬',  _  => '╨',
            };
            self.charframe[pos] = Character::new(write, whitespace.foreground, whitespace.background);
        }
    }

    //Place string in arbitrary location on screen
    pub fn draw_string(&mut self, string: &str, y: usize, x: usize, whitespace: Character<SCREEN_P>) {
        let mut p = y*CHARFR_W + x;
        //Check validity
        if p + string.len() >= self.charframe.len() {return;}
        //Place characters on screen
        for c in string.chars() {
            self.charframe[p] = Character::new(c, whitespace.foreground, whitespace.background);
            p += 1;
        }
    }
}
impl<
    const SCREEN_H: usize, const SCREEN_W: usize, const SCREEN_P: usize, const CHARFR_H: usize, const CHARFR_W: usize, 
    const PRINTW_M: usize, const PRINTW_H: usize, const PRINTW_W: usize, const PRINTW_Y: usize, const PRINTW_X: usize,
    const INPUTW_M: usize, const INPUTW_H: usize, const INPUTW_W: usize, const INPUTW_Y: usize, const INPUTW_X: usize,> 
    Write for Screen< 
        SCREEN_H, SCREEN_W, SCREEN_P, CHARFR_H, CHARFR_W,
        PRINTW_M, PRINTW_H, PRINTW_W, PRINTW_Y, PRINTW_X,
        INPUTW_M, INPUTW_H, INPUTW_W, INPUTW_Y, INPUTW_X,
> where [(); CHARFR_H*CHARFR_W]:, [(); PRINTW_M*PRINTW_W]:, [(); INPUTW_M*INPUTW_W]: {
    fn write_str(&mut self, string: &str) -> Result {
        self.string_print(string, self.print_whitespace);
        return Ok(());
    }

    fn write_char(&mut self, codepoint: char) -> Result {
        self.string_print(codepoint.encode_utf8(&mut [0; 4]), self.print_whitespace);
        return Ok(());
    }

    fn write_fmt(mut self: &mut Self, arguments: Arguments<'_>) -> Result {
        let r = core::fmt::write(&mut self, arguments);
        self.printbuffer_draw_render();
        return r;
    }
}
