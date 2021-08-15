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
pub const PHOTON_VERSION:      & str                   = "vDEV-2021-08-14";          //CURRENT VERSION OF GRAPHICS LIBRARY


// CHARACTER
//Struct
#[derive(Copy, Clone)]
pub struct Character<const PIXEL_WIDTH: usize> {
    codepoint:      char,
    foreground:     [u8; PIXEL_WIDTH],
    background:     [u8; PIXEL_WIDTH],
}

//General Implementation
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

// SCREEN
//Struct
pub struct Screen<
    //Screen Constants
    const SCREEN_H: usize, const SCREEN_W: usize, const SCREEN_P: usize,
    //Character Frame Constants
    const CHARFR_H: usize, const CHARFR_W: usize,
    //Print Result Window Constants
    const PRINTW_M: usize, const PRINTW_H: usize, const PRINTW_W: usize, const PRINTW_Y: usize, const PRINTW_X: usize,
    //Input Window Constants
    const INPUTW_M: usize, const INPUTW_W: usize, const INPUTW_Y: usize, const INPUTW_X: usize,> {
    //Screen variables
        screen:           *mut u8,
    //Character Frame Variables
        charframe:        [[Character<SCREEN_P>; CHARFR_W]; CHARFR_H],
    //Print Result Window Variables
        print_buffer:     [[Character<SCREEN_P>; PRINTW_W]; PRINTW_M],
        print_y:          usize,
        print_x:          usize,
        print_whitespace: Character<SCREEN_P>,
    //Input Window Variables
        input_buffer:     [Character<SCREEN_P>; INPUTW_M],
        input_p:          usize,
}

//General Implementation
impl<const SCREEN_H: usize, const SCREEN_W: usize, const SCREEN_P: usize, const CHARFR_H: usize, const CHARFR_W: usize, const PRINTW_M: usize, const PRINTW_H: usize, const PRINTW_W: usize, const PRINTW_Y: usize, const PRINTW_X: usize, const INPUTW_M: usize, const INPUTW_W: usize, const INPUTW_Y: usize, const INPUTW_X: usize,>
Screen<SCREEN_H, SCREEN_W, SCREEN_P, CHARFR_H, CHARFR_W, PRINTW_M, PRINTW_H, PRINTW_W, PRINTW_Y, PRINTW_X, INPUTW_M, INPUTW_W, INPUTW_Y, INPUTW_X,> {
    // CONSTRUCTOR
    //New Screen
    pub fn new(physical_pointer: *mut u8, whitespace: Character<SCREEN_P>) -> Self {
        Screen { 
            screen: physical_pointer,
            charframe: [[whitespace; CHARFR_W]; CHARFR_H],
            print_buffer: [[whitespace; PRINTW_W]; PRINTW_M],
            print_y: PRINTW_M - PRINTW_H,
            print_x: 0,
            print_whitespace: whitespace,
            input_buffer: [whitespace; INPUTW_M],
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
        self.charframe[y][x] = character;
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
                    self.print_buffer[y] = self.print_buffer[y+1];
                }
                //fill last line with spaces
                for x in 0..PRINTW_W {
                    self.print_buffer[PRINTW_M-1][x] = whitespace;
                }
                render = true;
            }
            //move to previous line (beginning of line moving backward)
            if control_backwards && end_backward {
                //reset xbuffer
                self.print_x = PRINTW_W;
                //move buffer down
                for y in (1..PRINTW_M).rev() {
                    self.print_buffer[y] = self.print_buffer[y-1];
                }
                //fill first line with spaces
                for x in 0..PRINTW_W {
                    self.print_buffer[0][x] = whitespace;
                }
                render = true;
            }
            //unprint character (backspace)
            if control_backwards {
                //move xbuffer back one
                self.print_x = self.print_x - 1;
                //remove character
                self.print_buffer[PRINTW_M - 1][self.print_x] = whitespace;
            }
            //print character (not: line Feed, carriage return, backspace)
            if !control {
                //place character
                self.print_buffer[PRINTW_M - 1][self.print_x] = Character::new(codepoint, whitespace.foreground, whitespace.background);
                //move xbuffer right
                self.print_x = self.print_x + 1;
            }
        }
        return render;
    }
    
    //Input character to input stack
    pub fn character_input(&mut self, character: Character<SCREEN_P>, whitespace: Character<SCREEN_P>) -> usize {
        let mut render:usize = 0;
        //control character booleans
        let control:bool = (character.codepoint as u32) < 0x20;                            //Determines if character is a control character
        let control_newline:bool = character.codepoint=='\n' || character.codepoint=='\r'; //Determines if character is a newline character
        let printable:bool = !control || control_newline;                                  //Determines if a character is printable
        let control_backwards:bool = character.codepoint=='\x08';                          //Determines if character types backwards (i.e. backspace)
        let end_forward:bool = self.input_p >= self.input_buffer.len();                    //Determines if the end of the stack is reached and position can no longer move forward
        let end_backward:bool = self.input_p <= 0;                                         //Determines if the beginning of the stack is reached and position can no longer move backwards
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


    // COMPOSITE FUNCTIONS
    //Render entire charframe
    pub fn characterframe_render(&mut self) {
        for y in 0..CHARFR_H {
            for x in 0..CHARFR_W {
                self.character_render(self.charframe[y][x], y, x);
            }
        }
    }

    //Draw character to characterframe screen and render it to physical screen
    pub fn character_draw_render(&mut self, character: Character<SCREEN_P>, y: usize, x: usize) {
        self.character_draw(character, y, x);
        self.character_render(character, y, x);
    }

    //Render entire inputstack
    pub fn inputstack_draw_render(&mut self, whitespace: Character<SCREEN_P>) {
        let overhang: usize = self.input_buffer.len() % INPUTW_W;
        let full: usize = self.input_buffer.len() - overhang;
        let line_width: usize = if self.input_p < full {INPUTW_W} else {overhang};
        for x in 0..line_width {
            self.character_draw_render(self.input_buffer[self.input_p/INPUTW_W+x], INPUTW_Y, INPUTW_X + x);
        }
        for x in line_width..INPUTW_W{
            self.character_draw_render(whitespace, INPUTW_Y, INPUTW_X + x)
        }
        
    }

    //Render entire printbuffer
    pub fn printbuffer_draw_render(&mut self) {
        for y in 0..PRINTW_H {
            for x in 0..PRINTW_W {
                self.character_draw_render(self.print_buffer[self.print_y+y][x], PRINTW_Y + y, PRINTW_X + x);
            }
        }
    }

    //Input character to input stack, draw it to characterframe, and render it to physical screen
    pub fn character_input_draw_render(&mut self, character: Character<SCREEN_P>, whitespace: Character<SCREEN_P>) {
        let p: usize = self.character_input(character, whitespace);
        if p % INPUTW_W == 0 || p % INPUTW_W == INPUTW_W - 1 {
            self.inputstack_draw_render(whitespace);
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
        for i in 0..INPUTW_M {
            self.input_buffer[i] = whitespace;
        }
        self.input_p = 0;
        self.inputstack_draw_render(whitespace);
    }

    //Return contents of the input stack as char array
    pub fn input_as_chararray<'f>(&mut self, buffer: &'f mut [char; INPUTW_M]) -> &'f [char] {
        for i in 0..INPUTW_M {
            buffer[i] = self.input_buffer[i].codepoint;
        }
        return &buffer[0..self.input_p];
    }

    //Return the length of the input stack
    pub fn inputbuffer_len(&self) -> usize {
        return self.input_buffer.len();
    }
    
    // PRINT WINDOW MOVING
    //Line Up
    pub fn print_line_up(&mut self) {
        if self.print_y > 0 {
            self.print_y -= 1;
            self.printbuffer_draw_render();
        }
    }

    //Line Down
    pub fn print_line_down(&mut self) {
        if self.print_y+PRINTW_H < PRINTW_M {
            self.print_y +=1;
            self.printbuffer_draw_render();
        }
    }

    //Page Up
    pub fn print_page_up(&mut self) {
        if self.print_y > PRINTW_H {
            self.print_y -= PRINTW_H;
            self.printbuffer_draw_render();
        }
        else if self.print_y > 0 {
            self.print_y = 0;
            self.printbuffer_draw_render();
        }
    }

    //Page Down
    pub fn print_page_down(&mut self) {
        if self.print_y+PRINTW_H < PRINTW_M - PRINTW_H {
            self.print_y += PRINTW_H;
            self.printbuffer_draw_render();
        }
        else if self.print_y+PRINTW_H < PRINTW_M {
            self.print_y = PRINTW_M - PRINTW_H;
            self.printbuffer_draw_render();
        }
    }

    //End Up
    pub fn print_end_up(&mut self) {
        self.print_y = 0;
        self.printbuffer_draw_render();
    }

    //End Down
    pub fn print_end_down(&mut self) {
        self.print_y = PRINTW_M - PRINTW_H;
        self.printbuffer_draw_render();
    }


    // DRAWING
    //Horizontal Line
    pub fn draw_hline(&mut self, y: usize, x1: usize, x2:usize, whitespace: Character<SCREEN_P>) {
        {
            let check: char = self.charframe[y][x1].codepoint;
            let write = match check {
                '═' => '═', '║' => '╠', '╔' => '╔', '╗' => '╦',
                '╚' => '╚', '╝' => '╩', '╞' => '╞', '╠' => '╠',
                '╡' => '═', '╣' => '╬', '╥' => '╔', '╦' => '╦',
                '╨' => '╚', '╩' => '╩', '╬' => '╬',  _  => '╞',
            };
            self.charframe[y][x1] = Character::new(write, whitespace.foreground, whitespace.background);
        }
        for x in x1+1..x2 {
            let check: char = self.charframe[y][x].codepoint;
            let write = match check {
                '═' => '═', '║' => '╬', '╔' => '╦', '╗' => '╦',
                '╚' => '╩', '╝' => '╩', '╞' => '═', '╠' => '╬',
                '╡' => '═', '╣' => '╬', '╥' => '╦', '╦' => '╦',
                '╨' => '╩', '╩' => '╩', '╬' => '╬',  _  => '═',
            };
            self.charframe[y][x] = Character::new(write, whitespace.foreground, whitespace.background);
        }
        {
            let check: char = self.charframe[y][x2].codepoint;
            let write = match check {
                '═' => '═', '║' => '╣', '╔' => '╦', '╗' => '╗',
                '╚' => '╩', '╝' => '╝', '╞' => '═', '╠' => '╬',
                '╡' => '╡', '╣' => '╣', '╥' => '╗', '╦' => '╦',
                '╨' => '╝', '╩' => '╩', '╬' => '╬',  _  => '╡',
            };
            self.charframe[y][x2] = Character::new(write, whitespace.foreground, whitespace.background);
        }
    }

    //Vertical Line
    pub fn draw_vline(&mut self, x: usize, y1: usize, y2:usize, whitespace: Character<SCREEN_P>) {
        {
            let check: char = self.charframe[y1][x].codepoint;
            let write = match check {
                '═' => '╦', '║' => '║', '╔' => '╔', '╗' => '╗',
                '╚' => '╠', '╝' => '╣', '╞' => '╔', '╠' => '╠',
                '╡' => '╗', '╣' => '╣', '╥' => '╥', '╦' => '╦', 
                '╨' => '║', '╩' => '╬', '╬' => '╬',  _  => '╥',
            };
            self.charframe[y1][x] = Character::new(write, whitespace.foreground, whitespace.background);
        }
        for y in y1+1..y2 {
            let check: char = self.charframe[y][x].codepoint;
            let write = match check {
                '═' => '╬', '║' => '║', '╔' => '╠', '╗' => '╣',
                '╚' => '╠', '╝' => '╣', '╞' => '╠', '╠' => '╠',
                '╡' => '╣', '╣' => '╣', '╥' => '║', '╦' => '╬',
                '╨' => '║', '╩' => '╬', '╬' => '╬',  _  => '║',
            };
            self.charframe[y][x] = Character::new(write, whitespace.foreground, whitespace.background);
        }
        {
            let check: char = self.charframe[y2][x].codepoint;
            let write = match check {
                '═' => '╩', '║' => '║', '╔' => '╠', '╗' => '╣',
                '╚' => '╚', '╝' => '╝', '╞' => '╚', '╠' => '╠',
                '╡' => '╝', '╣' => '╣', '╥' => '║', '╦' => '╬',
                '╨' => '╨', '╩' => '╩', '╬' => '╬',  _  => '╨',
            };
            self.charframe[y2][x] = Character::new(write, whitespace.foreground, whitespace.background);
        }
    }

    //Place string in arbitrary location on screen
    pub fn draw_string(&mut self, string: &str, y: usize, x: usize, whitespace: Character<SCREEN_P>) {
        let mut p = x;
        //Place characters on screen
        for c in string.chars() {
            //Check validity
            if p >= CHARFR_W {return;}
            //Draw
            self.charframe[y][p] = Character::new(c, whitespace.foreground, whitespace.background);
            //Move Position
            p += 1;
        }
    }
}

//Write Implementation
impl<const SCREEN_H: usize, const SCREEN_W: usize, const SCREEN_P: usize, const CHARFR_H: usize, const CHARFR_W: usize, const PRINTW_M: usize, const PRINTW_H: usize, const PRINTW_W: usize, const PRINTW_Y: usize, const PRINTW_X: usize, const INPUTW_M: usize, const INPUTW_W: usize, const INPUTW_Y: usize, const INPUTW_X: usize,> 
Write for Screen< SCREEN_H, SCREEN_W, SCREEN_P, CHARFR_H, CHARFR_W, PRINTW_M, PRINTW_H, PRINTW_W, PRINTW_Y, PRINTW_X, INPUTW_M, INPUTW_W, INPUTW_Y, INPUTW_X,> {
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
