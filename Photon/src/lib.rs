// PHOTON
// Photon is the Noble text graphics library:
// Draw to a frame buffer who's size decided at compile-time
// Character struct which holds a codepoint and color
// Screen struct which holds a frame buffer location and backlogs of text


// HEADER
//Flags
#![no_std]
#![allow(incomplete_features)]
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
pub const PHOTON_VERSION: & str = "vDEV-2021-08-15"; //CURRENT VERSION OF GRAPHICS LIBRARY


// CHARACTER
//Struct
#[derive(Copy, Clone)]
pub struct Character<const DEPTH: usize> {
    codepoint:      char,
    foreground:     [u8; DEPTH],
    background:     [u8; DEPTH],
}
//General Implementation
impl<const DEPTH: usize> Character<DEPTH> {
    // CONSTRUCTOR
    pub fn new(codepoint: char, foreground: [u8;DEPTH], background:  [u8;DEPTH]) -> Self {
        Character {
            codepoint,
            foreground,
            background,
        }
    }
}


// SCREEN
//Struct
#[derive(Clone, Copy)]
pub struct Renderer<const HEIGHT: usize, const WIDTH: usize, const DEPTH: usize> {
    pointer: *mut u8,
}
//General Implementation
impl<const HEIGHT: usize, const WIDTH: usize, const DEPTH: usize> 
Renderer<HEIGHT, WIDTH, DEPTH> {
    // CONSTRUCTOR
    pub fn new(pointer: *mut u8) -> Self {
        Renderer{
            pointer,
        }
    }

    // BASIC FUNCTION
    //Render character to physical screen
    pub fn render_character(&mut self, character: Character<DEPTH>, y: usize, x: usize) {
        //Check valid character position
        if y*16 >= HEIGHT || x*16 >= WIDTH {
            return;
        }
        //Find bitmap
        let bitmap = retrieve_font_bitmap(&character.codepoint);
        //Loop through bitmap
        for byte_row in 0..16 {
            for byte_column in 0..2 {
                for bit in 0..8 {
                    //Get position of of pixel in width*height space
                    let pixel_index = (y*16 + byte_row)*WIDTH + (x*16 + byte_column*8 + bit);
                    //Get color from bitmap bit and Character
                    let color: [u8; DEPTH] = 
                        if bitmap[byte_row*2 + byte_column] & (1 << bit) != 0{character.foreground}
                        else {character.background};
                    //Write color to screen
                    unsafe{
                        for i in 0..DEPTH {
                            write_volatile(
                                self.pointer.add(pixel_index*DEPTH + i),
                                color[i]
                            );
                        }
                    }
                }
            }
        }
    }
}


// CHARACTER FRAME
//Struct
pub struct CharacterFrame<const SCREEN_HEIGHT: usize, const SCREEN_WIDTH: usize, const SCREEN_DEPTH: usize, const HEIGHT: usize, const WIDTH: usize> {
    screen:          Renderer<SCREEN_HEIGHT, SCREEN_WIDTH, SCREEN_DEPTH>,
    character_frame: [[Character<SCREEN_DEPTH>; WIDTH]; HEIGHT]
}
//General Implementation
impl<const SCREEN_HEIGHT: usize, const SCREEN_WIDTH: usize, const SCREEN_DEPTH: usize, const HEIGHT: usize, const WIDTH: usize> 
CharacterFrame<SCREEN_HEIGHT, SCREEN_WIDTH, SCREEN_DEPTH, HEIGHT, WIDTH> {
    //CONSTRUCTOR
    pub fn new(screen: Renderer<SCREEN_HEIGHT, SCREEN_WIDTH, SCREEN_DEPTH>, whitespace: Character<SCREEN_DEPTH>) -> Self {
        CharacterFrame{
            screen,
            character_frame: [[whitespace; WIDTH]; HEIGHT]
        }
    }

    // BASIC FUNCTION
    //Place Character in Frame
    pub fn place(&mut self, character: Character<SCREEN_DEPTH>, y: usize, x: usize) {
        //Check valid character position
        if y >= HEIGHT || x >= WIDTH {
            return;
        }
        //Set character
        self.character_frame[y][x] = character;
    }

    // HIGHER FUNCTIONS
    //Render Entire Frame
    pub fn render(&mut self) {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                self.screen.render_character(self.character_frame[y][x], y, x);
            }
        }
    }

    //Draw character to characterframe screen and render it to physical screen
    pub fn character_place_render(&mut self, character: Character<SCREEN_DEPTH>, y: usize, x: usize) {
        self.place(character, y, x);
        self.screen.render_character(character, y, x);
    }

    //Horizontal Line
    pub fn horizontal_line(&mut self, y: usize, x1: usize, x2:usize, whitespace: Character<SCREEN_DEPTH>) {
        {
            let check: char = self.character_frame[y][x1].codepoint;
            let write = match check {
                '═' => '═', '║' => '╠', '╔' => '╔', '╗' => '╦',
                '╚' => '╚', '╝' => '╩', '╞' => '╞', '╠' => '╠',
                '╡' => '═', '╣' => '╬', '╥' => '╔', '╦' => '╦',
                '╨' => '╚', '╩' => '╩', '╬' => '╬',  _  => '╞',
            };
            self.character_frame[y][x1] = Character::new(write, whitespace.foreground, whitespace.background);
        }
        for x in x1+1..x2 {
            let check: char = self.character_frame[y][x].codepoint;
            let write = match check {
                '═' => '═', '║' => '╬', '╔' => '╦', '╗' => '╦',
                '╚' => '╩', '╝' => '╩', '╞' => '═', '╠' => '╬',
                '╡' => '═', '╣' => '╬', '╥' => '╦', '╦' => '╦',
                '╨' => '╩', '╩' => '╩', '╬' => '╬',  _  => '═',
            };
            self.character_frame[y][x] = Character::new(write, whitespace.foreground, whitespace.background);
        }
        {
            let check: char = self.character_frame[y][x2].codepoint;
            let write = match check {
                '═' => '═', '║' => '╣', '╔' => '╦', '╗' => '╗',
                '╚' => '╩', '╝' => '╝', '╞' => '═', '╠' => '╬',
                '╡' => '╡', '╣' => '╣', '╥' => '╗', '╦' => '╦',
                '╨' => '╝', '╩' => '╩', '╬' => '╬',  _  => '╡',
            };
            self.character_frame[y][x2] = Character::new(write, whitespace.foreground, whitespace.background);
        }
    }

    //Vertical Line
    pub fn vertical_line(&mut self, x: usize, y1: usize, y2:usize, whitespace: Character<SCREEN_DEPTH>) {
        {
            let check: char = self.character_frame[y1][x].codepoint;
            let write = match check {
                '═' => '╦', '║' => '║', '╔' => '╔', '╗' => '╗',
                '╚' => '╠', '╝' => '╣', '╞' => '╔', '╠' => '╠',
                '╡' => '╗', '╣' => '╣', '╥' => '╥', '╦' => '╦', 
                '╨' => '║', '╩' => '╬', '╬' => '╬',  _  => '╥',
            };
            self.character_frame[y1][x] = Character::new(write, whitespace.foreground, whitespace.background);
        }
        for y in y1+1..y2 {
            let check: char = self.character_frame[y][x].codepoint;
            let write = match check {
                '═' => '╬', '║' => '║', '╔' => '╠', '╗' => '╣',
                '╚' => '╠', '╝' => '╣', '╞' => '╠', '╠' => '╠',
                '╡' => '╣', '╣' => '╣', '╥' => '║', '╦' => '╬',
                '╨' => '║', '╩' => '╬', '╬' => '╬',  _  => '║',
            };
            self.character_frame[y][x] = Character::new(write, whitespace.foreground, whitespace.background);
        }
        {
            let check: char = self.character_frame[y2][x].codepoint;
            let write = match check {
                '═' => '╩', '║' => '║', '╔' => '╠', '╗' => '╣',
                '╚' => '╚', '╝' => '╝', '╞' => '╚', '╠' => '╠',
                '╡' => '╝', '╣' => '╣', '╥' => '║', '╦' => '╬',
                '╨' => '╨', '╩' => '╩', '╬' => '╬',  _  => '╨',
            };
            self.character_frame[y2][x] = Character::new(write, whitespace.foreground, whitespace.background);
        }
    }

    //Place string in arbitrary location on screen
    pub fn horizontal_string(&mut self, string: &str, y: usize, x: usize, whitespace: Character<SCREEN_DEPTH>) {
        if y >= HEIGHT {return;}
        let mut p = x;
        //Place characters on screen
        for c in string.chars() {
            //Check validity
            if p >= WIDTH {return;}
            //Draw
            self.character_frame[y][p] = Character::new(c, whitespace.foreground, whitespace.background);
            //Move Position
            p += 1;
        }
    }
}


// PRINT WINDOW
//Struct
pub struct PrintWindow<const SCREEN_HEIGHT: usize, const SCREEN_WIDTH: usize, const SCREEN_DEPTH: usize, const LINES: usize, const HEIGHT: usize, const WIDTH: usize, const Y: usize, const X: usize> {
    screen:           Renderer<SCREEN_HEIGHT, SCREEN_WIDTH, SCREEN_DEPTH>,
    print_buffer:     [[Character<SCREEN_DEPTH>; WIDTH]; LINES],
    print_y:          usize,
    print_x:          usize,
    write_whitespace: Character<SCREEN_DEPTH>,
    line_whitespace:  Character<SCREEN_DEPTH>,
}
//General Implementation
impl<const SCREEN_HEIGHT: usize, const SCREEN_WIDTH: usize, const SCREEN_DEPTH: usize, const LINES: usize, const HEIGHT: usize, const WIDTH: usize, const Y: usize, const X: usize>
PrintWindow<SCREEN_HEIGHT, SCREEN_WIDTH, SCREEN_DEPTH, LINES, HEIGHT, WIDTH, Y, X> {
    //CONSTRUCTOR
    pub fn new(screen: Renderer<SCREEN_HEIGHT, SCREEN_WIDTH, SCREEN_DEPTH>, write_whitespace: Character<SCREEN_DEPTH>, line_whitespace: Character<SCREEN_DEPTH>) -> Self{
        PrintWindow{
            screen,
            print_buffer: [[write_whitespace; WIDTH]; LINES],
            print_y: LINES - HEIGHT,
            print_x: 0,
            write_whitespace,
            line_whitespace,
        }
    }

    // BASIC FUNCTION
    //Input string to print buffer
    pub fn push(&mut self, string: &str, whitespace: Character<SCREEN_DEPTH>) -> bool {
        let mut render:bool = false;
        //begin writing to the buffer
        for codepoint in string.chars() {
            let control = (codepoint as u32) < 0x20;
            let control_newline = codepoint=='\n' || codepoint=='\r';
            let control_backwards = codepoint=='\x08';
            let end_forward = self.print_x >= WIDTH;
            let end_backward = self.print_x <= 0;
            //move to next line (line feed, carriage return, end of line moving forward)
            if (control_newline || end_forward) &! control_backwards {
                //reset xbuffer
                self.print_x=0;
                //move buffer up
                for y in 0..LINES-1 {
                    self.print_buffer[y] = self.print_buffer[y+1];
                }
                //fill last line with spaces
                for x in 0..WIDTH {
                    self.print_buffer[LINES-1][x] = self.line_whitespace;
                }
                render = true;
            }
            //move to previous line (beginning of line moving backward)
            if control_backwards && end_backward {
                //reset xbuffer
                self.print_x = WIDTH;
                //move buffer down
                for y in (1..LINES).rev() {
                    self.print_buffer[y] = self.print_buffer[y-1];
                }
                //fill first line with spaces
                for x in 0..WIDTH {
                    self.print_buffer[0][x] = whitespace;
                }
                render = true;
            }
            //unprint character (backspace)
            if control_backwards {
                //move xbuffer back one
                self.print_x = self.print_x - 1;
                //remove character
                self.print_buffer[LINES - 1][self.print_x] = whitespace;
            }
            //print character (not: line Feed, carriage return, backspace)
            if !control {
                //place character
                self.print_buffer[LINES - 1][self.print_x] = Character::new(codepoint, whitespace.foreground, whitespace.background);
                //move xbuffer right
                self.print_x = self.print_x + 1;
            }
        }
        return render;
    }

    // HIGHER FUNCTIONS
    //Render entire printbuffer
    pub fn render(&mut self) {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                self.screen.render_character(self.print_buffer[self.print_y+y][x], Y+y, X+x);
            }
        }
    }

    //Input string to print buffer, draw it to characterframe, and render it to physical screen
    pub fn push_render(&mut self, character: &str, whitespace: Character<SCREEN_DEPTH>) {
        if self.push(character, whitespace) {self.render()}
    }
    
    //Line Up
    pub fn line_up(&mut self) {
        if self.print_y > 0 {
            self.print_y -= 1;
            self.render();
        }
    }

    //Line Down
    pub fn line_down(&mut self) {
        if self.print_y+HEIGHT < LINES {
            self.print_y +=1;
            self.render();
        }
    }

    //Page Up
    pub fn page_up(&mut self) {
        if self.print_y > HEIGHT {
            self.print_y -= HEIGHT;
            self.render();
        }
        else if self.print_y > 0 {
            self.print_y = 0;
            self.render();
        }
    }

    //Page Down
    pub fn page_down(&mut self) {
        if self.print_y+HEIGHT < LINES-HEIGHT {
            self.print_y += HEIGHT;
            self.render();
        }
        else if self.print_y+HEIGHT < LINES {
            self.print_y = LINES - HEIGHT;
            self.render();
        }
    }

    //End Up
    pub fn end_up(&mut self) {
        self.print_y = 0;
        self.render();
    }

    //End Down
    pub fn end_down(&mut self) {
        self.print_y = LINES - HEIGHT;
        self.render();
    }
}
//Write Impementation
impl <const SCREEN_HEIGHT: usize, const SCREEN_WIDTH: usize, const SCREEN_DEPTH: usize, const LINES: usize, const HEIGHT: usize, const WIDTH: usize, const Y: usize, const X: usize>
Write for PrintWindow<SCREEN_HEIGHT, SCREEN_WIDTH, SCREEN_DEPTH, LINES, HEIGHT, WIDTH, Y, X> {
    fn write_str(&mut self, string: &str) -> Result {
        self.push(string, self.write_whitespace);
        return Ok(());
    }

    fn write_char(&mut self, codepoint: char) -> Result {
        self.push(codepoint.encode_utf8(&mut [0; 4]), self.write_whitespace);
        return Ok(());
    }

    fn write_fmt(mut self: &mut Self, arguments: Arguments<'_>) -> Result {
        let r = core::fmt::write(&mut self, arguments);
        self.render();
        return r;
    }
}


// INPUT WINDOW
//Struct
pub struct InputWindow<const SCREEN_HEIGHT: usize, const SCREEN_WIDTH: usize, const SCREEN_DEPTH: usize, const LENGTH: usize, const WIDTH: usize, const Y: usize, const X: usize> {
    screen:           Renderer<SCREEN_HEIGHT, SCREEN_WIDTH, SCREEN_DEPTH>,
    input_buffer:     [Character<SCREEN_DEPTH>; LENGTH],
    input_p:          usize,
}
//General Implementation
impl<const SCREEN_HEIGHT: usize, const SCREEN_WIDTH: usize, const SCREEN_DEPTH: usize, const LENGTH: usize, const WIDTH: usize, const Y: usize, const X: usize>
InputWindow<SCREEN_HEIGHT, SCREEN_WIDTH, SCREEN_DEPTH, LENGTH, WIDTH, Y, X> {
    // CONSTRUCTOR
    pub fn new(screen: Renderer<SCREEN_HEIGHT, SCREEN_WIDTH, SCREEN_DEPTH>, whitespace: Character<SCREEN_DEPTH>) -> Self {
        InputWindow {
            screen,
            input_buffer: [whitespace; LENGTH],
            input_p: 0,
        }
    }

    // BASIC FUNCTION
    //Input character to input stack
    pub fn push(&mut self, character: Character<SCREEN_DEPTH>, whitespace: Character<SCREEN_DEPTH>) -> usize {
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

    // HIGHER FUNCTIONS
    //Input character to input stack and render it
    pub fn push_render(&mut self, character: Character<SCREEN_DEPTH>, whitespace: Character<SCREEN_DEPTH>) {
        let p: usize = self.push(character, whitespace);
        if p % WIDTH == 0 || p % WIDTH == WIDTH - 1 {
            self.render(whitespace);
        }
        else {
            self.screen.render_character(self.input_buffer[p%WIDTH], Y, X+p);
        }
    }

    //Render entire inputstack
    pub fn render(&mut self, whitespace: Character<SCREEN_DEPTH>) {
        let overhang: usize = self.input_buffer.len() % WIDTH;
        let full: usize = self.input_buffer.len() - overhang;
        let line_width: usize = if self.input_p < full {WIDTH} else {overhang};
        for x in 0..line_width {
            self.screen.render_character(self.input_buffer[self.input_p/WIDTH+x], Y, X + x);
        }
        for x in line_width..WIDTH {
            self.screen.render_character(whitespace, Y, X+x)
        }
    }

    //Remove all information from input stack and replace with Character given
    pub fn flush(&mut self, whitespace: Character<SCREEN_DEPTH>) {
        for i in 0..LENGTH {
            self.input_buffer[i] = whitespace;
        }
        self.input_p = 0;
        self.render(whitespace);
    }

    //Return contents of the input stack as char array
    pub fn to_chararray<'f>(&mut self, buffer: &'f mut [char; LENGTH]) -> &'f [char] {
        for i in 0..LENGTH {
            buffer[i] = self.input_buffer[i].codepoint;
        }
        return &buffer[0..self.input_p];
    }

    //Return the length of the input stack
    pub fn len(&self) -> usize {
        return self.input_buffer.len();
    }
}


// SPECIFIC COLOR FORMATS
//Pixel Format: Blue, Green, Red, Reserved
pub const BGRX_DEPTH:     usize            = 4;                        //BIT DEPTH
pub const COLOR_BLK_BGRX: [u8; BGRX_DEPTH] = [0x00, 0x00, 0x00, 0x00]; //BLACK
pub const COLOR_RED_BGRX: [u8; BGRX_DEPTH] = [0x00, 0x00, 0xFF, 0x00]; //RED
pub const COLOR_GRN_BGRX: [u8; BGRX_DEPTH] = [0x00, 0xFF, 0x00, 0x00]; //GREEN
pub const COLOR_BLU_BGRX: [u8; BGRX_DEPTH] = [0xFF, 0x00, 0x00, 0x00]; //BLUE
pub const COLOR_CYN_BGRX: [u8; BGRX_DEPTH] = [0xFF, 0xFF, 0x00, 0x00]; //CYAN
pub const COLOR_MGT_BGRX: [u8; BGRX_DEPTH] = [0xFF, 0x00, 0xFF, 0x00]; //MAGENTA
pub const COLOR_YLW_BGRX: [u8; BGRX_DEPTH] = [0x00, 0xFF, 0xFF, 0x00]; //YELLOW
pub const COLOR_WHT_BGRX: [u8; BGRX_DEPTH] = [0xFF, 0xFF, 0xFF, 0x00]; //COLOR PURE WHITE


// SPECIFIC RESOLUTIONS
//Width: 1920 Pixels, Height: 1080 Pixels, Pixel Format: Blue, Green, Red, Reserved
pub const SCREEN_W_1920_1080_BRGX: usize = 1920; //PIXEL WIDTH OF SCREEN
pub const SCREEN_H_1920_1080_BRGX: usize = 1080; //PIXEL HEIGHT OF SCREEN
pub const CHARFR_W_1920_1080_BRGX: usize = 120;  //TEXT MODE WIDTH OF CHARACTER FRAME
pub const CHARFR_H_1920_1080_BRGX: usize = 67;   //TEXT MODE HEIGHT OF CHARACTER FRAME
pub const PRINTW_X_1920_1080_BRGX: usize = 1;    //TEXT MODE HORIZONTAL POSITION OF PRINT RESULT WINDOW
pub const PRINTW_Y_1920_1080_BRGX: usize = 2;    //TEXT MODE VERTICAL POSITION OF PRINT RESULT WINDOW
pub const PRINTW_W_1920_1080_BRGX: usize = 118;  //TEXT MODE WIDTH OF PRINT RESULT WINDOW
pub const PRINTW_H_1920_1080_BRGX: usize = 62;   //TEXT MODE HEIGHT OF PRINT RESULT WINDOW ON SCREEN
pub const PRINTW_M_1920_1080_BRGX: usize = 200;  //TEXT MODE HEIGHT OF PRINT RESULT WINDOW IN MEMORY
pub const INPUTW_X_1920_1080_BRGX: usize = 1;    //TEXT MODE HORIZONTAL POSITION OF INPUT WINDOW
pub const INPUTW_Y_1920_1080_BRGX: usize = 65;   //TEXT MODE VERTICAL POSITION OF INPUT WINDOW
pub const INPUTW_W_1920_1080_BRGX: usize = 118;  //TEXT MODE WIDTH OF INPUT WINDOW
pub const INPUTW_L_1920_1080_BRGX: usize = 118;  //TEXT MODE HEIGHT OF INPUT WINDOW IN MEMORY
