// HEADER
//Flags
#![no_std]

//Modules
pub mod font_handler;

//Imports
use core::{
    fmt::{
        Write, 
        Result, 
        Arguments
    }, 
    ptr::{
        write_volatile
    }
};
use crate::{
    font_handler::{
        retrieve_font_bitmap
    }
};

//Constants
pub const PIXL_SCRN_X_DIM: usize = 1920;                               //PIXEL WIDTH OF SCREEN
pub const PIXL_SCRN_Y_DIM: usize = 1080;                               //PIXEL HEIGHT OF SCREEN
pub const PIXL_SCRN_B_DEP: usize = 4;                                  //PIXEL BIT DEPTH
pub const COLR_WHITE:[u8; PIXL_SCRN_B_DEP] = [0xFF, 0xFF, 0xFF, 0x00]; //COLOR WHITE
pub const COLR_BLACK:[u8; PIXL_SCRN_B_DEP] = [0x00, 0x00, 0x00, 0x00]; //COLOR BLACK
pub const COLR_PRRED:[u8; PIXL_SCRN_B_DEP] = [0x00, 0x00, 0xFF, 0x00]; //COLOR PURE RED
pub const CHAR_SCRN_X_DIM: usize = 120;                                //TEXT MODE WIDTH OF ENTIRE SCREEN
pub const CHAR_SCRN_Y_DIM: usize = 67;                                 //TEXT MODE HEIGHT OF ENTIRE SCREEN
pub const CHAR_PRNT_X_POS: usize = 1;                                  //TEXT MODE HORIZONTAL POSITION OF PRINT RESULT WINDOW
pub const CHAR_PRNT_Y_POS: usize = 2;                                  //TEXT MODE VERTICAL POSITION OF PRINT RESULT WINDOW
pub const CHAR_PRNT_X_DIM: usize = 118;                                //TEXT MODE WIDTH OF PRINT RESULT WINDOW
pub const CHAR_PRNT_Y_DIM_DSP: usize = 62;                             //TEXT MODE HEIGHT OF PRINT RESULT WINDOW ON SCREEN
pub const CHAR_PRNT_Y_DIM_MEM: usize = 400;                            //TEXT MODE HEIGHT OF PRINT RESULT WINDOW IN MEMORY
pub const CHAR_INPT_X_POS: usize = 1;                                  //TEXT MODE HORIZONTAL POSITION OF INPUT WINDOW
pub const CHAR_INPT_Y_POS: usize = 65;                                 //TEXT MODE VERTICAL POSITION OF INPUT WINDOW
pub const CHAR_INPT_X_DIM: usize = 118;                                //TEXT MODE WIDTH OF INPUT WINDOW
pub const CHAR_INPT_Y_DIM: usize = 1;                                  //TEXT MODE HEIGHT OF INPUT WINDOW
pub const CHAR_INPT_Y_DIM_MEM: usize = 1;                              //TEXT MODE HEIGHT OF INPUT WINDOW IN MEMORY
pub const PHOTON_VERSION: &str = "v2021-08-05";                        //CURRENT VERSION OF BOOTLOADER


// STRUCTS
//Character
#[derive(Copy, Clone)]
pub struct Character {
    codepoint: char,
    foreground: [u8; PIXL_SCRN_B_DEP],
    background: [u8; PIXL_SCRN_B_DEP]
}
impl Character{
    // CONSTRUCTOR
    pub fn new(codepoint: char, foreground: [u8; PIXL_SCRN_B_DEP], background: [u8;PIXL_SCRN_B_DEP]) -> Character{
        return Character{
            codepoint:  codepoint,
            foreground: foreground,
            background: background
        }
    }
}

//Screen
pub struct Screen<'a>{
    //Screen variables
    pub screen_physical: *   mut u8,
    pub screen_charframe:&'a mut [Character;CHAR_SCRN_X_DIM*CHAR_SCRN_Y_DIM],
    //Input Window Variables
    pub input_stack:     &'a mut [Character; CHAR_INPT_X_DIM * CHAR_INPT_Y_DIM_MEM],
    pub input_p:         &'a mut usize,
    //Print Result Window Variables
    pub print_buffer:    &'a mut [Character; CHAR_PRNT_X_DIM * CHAR_PRNT_Y_DIM_MEM],
    pub print_y:         &'a mut usize,
    pub print_x:         &'a mut usize,
}
impl<'a> Screen<'a>{
    // BASIC FUNCTIONS
    //Render character to physical screen
    pub fn character_render(&mut self, c: Character, y: usize, x: usize){
        //Check valid character position
        if x >= CHAR_SCRN_X_DIM || y >= CHAR_SCRN_Y_DIM {
            return;
        }
        //Find bitmap
        let bitmap = retrieve_font_bitmap(&c.codepoint);
        //Loop through bitmap
        for byte_row in 0..16{
            for byte_column in 0..2{
                for bit in 0..8{
                    //Get position of of pixel in width*height space
                    let pixel_index = (y*16 + byte_row)*PIXL_SCRN_X_DIM + (x*16 + byte_column*8 + bit);
                    //Get color from bitmap bit and Character
                    let color:[u8;4] = 
                        if bitmap[byte_row*2 + byte_column] & (1 << bit) != 0{c.foreground}
                        else {c.background};
                    //Write color to screen
                    unsafe{
                        //let mut t = self.screen_physical[pixel_index*PIXL_SCRN_B_DEP..(pixel_index+1)*PIXL_SCRN_B_DEP].as_mut_ptr();
                        for i in 0..4{
                            write_volatile(
                                self.screen_physical.add(pixel_index*PIXL_SCRN_B_DEP + i),
                                color[i]
                            );
                        }
                    }
                }
            }
        }
    }

    //Draw character to characterframe screen
    pub fn character_draw(&mut self, c: Character, y: usize, x: usize){
        //Check valid character position
        if y >= CHAR_SCRN_Y_DIM || x >= CHAR_SCRN_X_DIM {
            return;
        }
        //Set character
        self.screen_charframe[y*CHAR_SCRN_X_DIM + x] = c;
    }

    //Input character to input stack
    pub fn character_input(&mut self, c: Character) -> usize{
        let mut render:usize = 0;
        //control character booleans
        let control:bool = (c.codepoint as u32) < 0x20;                    //Determines if character is a control character
        let control_newline:bool = c.codepoint=='\n' || c.codepoint=='\r'; //Determines if character is a newline character
        let printable:bool = !control || control_newline;                  //Determines if a character is printable
        let control_backwards:bool = c.codepoint=='\x08';                  //Determines if character types backwards (i.e. backspace)
        let end_forward:bool = *self.input_p >= self.input_stack.len();    //Determines if the end of the stack is reached and position can no longer move forward
        let end_backward:bool = *self.input_p <= 0;                        //Determines if the beginning of the stack is reached and position can no longer move backwards
        //add printable character to stack
        if printable && !end_forward {
            self.input_stack[*self.input_p] = c;
            render = *self.input_p;
            *self.input_p = *self.input_p + 1;
        }
        //backspace handling
        else if control_backwards && !end_backward {
            *self.input_p = *self.input_p - 1;
            render = *self.input_p;
            self.input_stack[*self.input_p] = Character::new(' ', c.foreground, c.background);
        }
        return render;
    }

    //Input string to print buffer
    pub fn string_print(&mut self, s: &str, foreground: [u8;4], background: [u8;4]) -> bool{
        let mut render:bool = false;
        //begin writing to the buffer
        for codepoint in s.chars(){
            let control = (codepoint as u32) < 0x20;
            let control_newline = codepoint=='\n' || codepoint=='\r';
            let control_backwards = codepoint=='\x08';
            let end_forward = *self.print_x >= CHAR_PRNT_X_DIM;
            let end_backward = *self.print_x <= 0;
            //move to next line (line feed, carriage return, end of line moving forward)
            if (control_newline || end_forward) &! control_backwards {
                //reset xbuffer
                *self.print_x=0;
                //move buffer up
                for y in 0..CHAR_PRNT_Y_DIM_MEM-1{
                    for x in 0..CHAR_PRNT_X_DIM{
                        self.print_buffer[y*CHAR_PRNT_X_DIM+x] = self.print_buffer[(y+1)*CHAR_PRNT_X_DIM+x];
                    }
                }
                //fill last line with spaces
                for x in 0..CHAR_PRNT_X_DIM{
                    self.print_buffer[(CHAR_PRNT_Y_DIM_MEM-1)*CHAR_PRNT_X_DIM+x] = Character::new(' ', foreground, background);
                }
                render = true;
            }
            //move to previous line (beginning of line moving backward)
            if control_backwards && end_backward {
                //reset xbuffer
                *self.print_x = CHAR_PRNT_X_DIM;
                //move buffer down
                for y in (1..CHAR_PRNT_Y_DIM_MEM).rev(){
                    for x in 0..CHAR_PRNT_X_DIM{
                        self.print_buffer[y*CHAR_PRNT_X_DIM+x] = self.print_buffer[(y-1)*CHAR_PRNT_X_DIM+x];
                    }
                }
                //fill first line with spaces
                for x in 0..CHAR_PRNT_X_DIM{
                    self.print_buffer[x] = Character::new(' ', foreground, background);
                }
                render = true;
            }
            //unprint character (backspace)
            if control_backwards {
                //move xbuffer back one
                *self.print_x = *self.print_x - 1;
                //remove character
                self.print_buffer[(CHAR_PRNT_Y_DIM_MEM - 1) * CHAR_PRNT_X_DIM + *self.print_x] = Character::new(' ', foreground, background);
            }
            //print character (not: line Feed, carriage return, backspace)
            if !control{
                //place character
                self.print_buffer[(CHAR_PRNT_Y_DIM_MEM - 1) * CHAR_PRNT_X_DIM + *self.print_x] = Character::new(codepoint, foreground, background);
                //move xbuffer right
                *self.print_x = *self.print_x + 1;
            }
        }
        return render;
    }


    // COMPOSITE FUNCTIONS
    //Render entire charframe
    pub fn characterframe_render(&mut self){
        for y in 0..CHAR_SCRN_Y_DIM{
            for x in 0..CHAR_SCRN_X_DIM{
                self.character_render(self.screen_charframe[y*CHAR_SCRN_X_DIM + x], y, x);
            }
        }
    }

    //Draw character to characterframe screen and render it to physical screen
    pub fn character_draw_render(&mut self, c: Character, y: usize, x: usize){
        self.character_draw(c, y, x);
        self.character_render(c, y, x)
    }

    //Render entire inputstack
    pub fn inputstack_draw_render(&mut self){
        for x in 0..CHAR_INPT_X_DIM{
            self.character_draw_render(self.input_stack[*self.input_p/CHAR_INPT_X_DIM+x], CHAR_INPT_Y_POS, CHAR_INPT_X_POS + x);
        }
    }

    //Render entire printbuffer
    pub fn printbuffer_draw_render(&mut self){
        for y in 0..CHAR_PRNT_Y_DIM_DSP{
            for x in 0..CHAR_PRNT_X_DIM{
                self.character_draw_render(self.print_buffer[(*self.print_y + y) * CHAR_PRNT_X_DIM + x], y + CHAR_PRNT_Y_POS, x + CHAR_PRNT_X_POS);
            }
        }
    }

    //Input character to input stack, draw it to characterframe, and render it to physical screen
    pub fn character_input_draw_render(&mut self, c: Character){
        let p:usize = self.character_input(c);
        if p % CHAR_INPT_X_DIM == 0 || p % CHAR_INPT_X_DIM == CHAR_INPT_X_DIM - 1 {
            self.inputstack_draw_render();
        }
        else{
            self.character_draw_render(self.input_stack[p%CHAR_INPT_X_DIM], CHAR_INPT_Y_POS, CHAR_INPT_X_POS + p);
        }
    }

    //Input string to print buffer, draw it to characterframe, and render it to physical screen
    pub fn string_print_draw_render(&mut self, s: &str, foreground: [u8;4], background: [u8;4]) {
        self.string_print(s, foreground, background);
        self.printbuffer_draw_render();
    }

    //Remove all information from input stack and replace with Character given
    pub fn input_flush(&mut self, c: Character) {
        for i in 0..CHAR_INPT_X_DIM * CHAR_INPT_Y_DIM_MEM{
            self.input_stack[i] = c;
        }
        *self.input_p = 0;
        self.inputstack_draw_render();
    }

    //Return input characterstack as char array
    pub fn input_as_chararray(&mut self) -> [char; CHAR_INPT_X_DIM*CHAR_INPT_Y_DIM_MEM]{
        let mut value = [' ';CHAR_INPT_X_DIM*CHAR_INPT_Y_DIM_MEM];
        for i in 0..CHAR_INPT_X_DIM * CHAR_INPT_Y_DIM_MEM{
            value[i] = self.input_stack[i].codepoint;
        }
        return value;
    }


    // DRAWING
    //Horizontal Line
    pub fn draw_hline(&mut self, y: usize, x1: usize, x2:usize, foreground: [u8;4], background: [u8;4]){
        {
            let pos: usize = y*CHAR_SCRN_X_DIM + x1;
            let draw: &mut Character = &mut self.screen_charframe[pos];
            let check: char = (*draw).codepoint;
            if      check == '═' {*draw = Character::new('═', foreground, background);}
            else if check == '║' {*draw = Character::new('╠', foreground, background);}
            else if check == '╔' {*draw = Character::new('╔', foreground, background);}
            else if check == '╗' {*draw = Character::new('╦', foreground, background);}
            else if check == '╚' {*draw = Character::new('╚', foreground, background);}
            else if check == '╝' {*draw = Character::new('╩', foreground, background);}
            else if check == '╞' {*draw = Character::new('╞', foreground, background);}
            else if check == '╠' {*draw = Character::new('╠', foreground, background);}
            else if check == '╡' {*draw = Character::new('═', foreground, background);}
            else if check == '╣' {*draw = Character::new('╬', foreground, background);}
            else if check == '╥' {*draw = Character::new('╔', foreground, background);}
            else if check == '╦' {*draw = Character::new('╦', foreground, background);}
            else if check == '╨' {*draw = Character::new('╚', foreground, background);}
            else if check == '╩' {*draw = Character::new('╩', foreground, background);}
            else if check == '╬' {*draw = Character::new('╬', foreground, background);}
            else                 {*draw = Character::new('╞', foreground, background);}
        }
        for x in x1+1..x2{
            let pos: usize = y*CHAR_SCRN_X_DIM + x;
            let draw: &mut Character = &mut self.screen_charframe[pos];
            let check: char = (*draw).codepoint;
            if      check == '═' {*draw = Character::new('═', foreground, background);}
            else if check == '║' {*draw = Character::new('╬', foreground, background);}
            else if check == '╔' {*draw = Character::new('╦', foreground, background);}
            else if check == '╗' {*draw = Character::new('╦', foreground, background);}
            else if check == '╚' {*draw = Character::new('╩', foreground, background);}
            else if check == '╝' {*draw = Character::new('╩', foreground, background);}
            else if check == '╞' {*draw = Character::new('═', foreground, background);}
            else if check == '╠' {*draw = Character::new('╬', foreground, background);}
            else if check == '╡' {*draw = Character::new('═', foreground, background);}
            else if check == '╣' {*draw = Character::new('╬', foreground, background);}
            else if check == '╥' {*draw = Character::new('╦', foreground, background);}
            else if check == '╦' {*draw = Character::new('╦', foreground, background);}
            else if check == '╨' {*draw = Character::new('╩', foreground, background);}
            else if check == '╩' {*draw = Character::new('╩', foreground, background);}
            else if check == '╬' {*draw = Character::new('╬', foreground, background);}
            else                 {*draw = Character::new('═', foreground, background);}
        }
        {
            let pos: usize = y*CHAR_SCRN_X_DIM + x2;
            let draw: &mut Character = &mut self.screen_charframe[pos];
            let check: char = (*draw).codepoint;
            if      check == '═' {*draw = Character::new('═',foreground, background);}
            else if check == '║' {*draw = Character::new('╣',foreground, background);}
            else if check == '╔' {*draw = Character::new('╦',foreground, background);}
            else if check == '╗' {*draw = Character::new('╗',foreground, background);}
            else if check == '╚' {*draw = Character::new('╩',foreground, background);}
            else if check == '╝' {*draw = Character::new('╝',foreground, background);}
            else if check == '╞' {*draw = Character::new('═',foreground, background);}
            else if check == '╠' {*draw = Character::new('╬',foreground, background);}
            else if check == '╡' {*draw = Character::new('╡',foreground, background);}
            else if check == '╣' {*draw = Character::new('╣',foreground, background);}
            else if check == '╥' {*draw = Character::new('╗',foreground, background);}
            else if check == '╦' {*draw = Character::new('╦',foreground, background);}
            else if check == '╨' {*draw = Character::new('╝',foreground, background);}
            else if check == '╩' {*draw = Character::new('╩',foreground, background);}
            else if check == '╬' {*draw = Character::new('╬',foreground, background);}
            else                 {*draw = Character::new('╡',foreground, background);}
        }
    }

    //Vertical Line
    pub fn draw_vline(&mut self, x: usize, y1: usize, y2:usize, foreground: [u8;4], background: [u8;4]){
        {
            let pos: usize = y1*CHAR_SCRN_X_DIM + x;
            let draw: &mut Character = &mut self.screen_charframe[pos];
            let check: char = (*draw).codepoint;
            if      check == '═' {*draw = Character::new('╦', foreground, background);}
            else if check == '║' {*draw = Character::new('║', foreground, background);}
            else if check == '╔' {*draw = Character::new('╔', foreground, background);}
            else if check == '╗' {*draw = Character::new('╗', foreground, background);}
            else if check == '╚' {*draw = Character::new('╠', foreground, background);}
            else if check == '╝' {*draw = Character::new('╣', foreground, background);}
            else if check == '╞' {*draw = Character::new('╔', foreground, background);}
            else if check == '╠' {*draw = Character::new('╠', foreground, background);}
            else if check == '╡' {*draw = Character::new('╗', foreground, background);}
            else if check == '╣' {*draw = Character::new('╣', foreground, background);}
            else if check == '╥' {*draw = Character::new('╥', foreground, background);}
            else if check == '╦' {*draw = Character::new('╦', foreground, background);}
            else if check == '╨' {*draw = Character::new('║', foreground, background);}
            else if check == '╩' {*draw = Character::new('╬', foreground, background);}
            else if check == '╬' {*draw = Character::new('╬', foreground, background);}
            else                 {*draw = Character::new('╥', foreground, background);}
        }
        for y in y1+1..y2{
            let pos: usize = y*CHAR_SCRN_X_DIM + x;
            let draw: &mut Character = &mut self.screen_charframe[pos];
            let check: char = (*draw).codepoint;
            if      check == '═' {*draw = Character::new('╬', foreground, background);}
            else if check == '║' {*draw = Character::new('║', foreground, background);}
            else if check == '╔' {*draw = Character::new('╠', foreground, background);}
            else if check == '╗' {*draw = Character::new('╣', foreground, background);}
            else if check == '╚' {*draw = Character::new('╠', foreground, background);}
            else if check == '╝' {*draw = Character::new('╣', foreground, background);}
            else if check == '╞' {*draw = Character::new('╠', foreground, background);}
            else if check == '╠' {*draw = Character::new('╠', foreground, background);}
            else if check == '╡' {*draw = Character::new('╣', foreground, background);}
            else if check == '╣' {*draw = Character::new('╣', foreground, background);}
            else if check == '╥' {*draw = Character::new('║', foreground, background);}
            else if check == '╦' {*draw = Character::new('╬', foreground, background);}
            else if check == '╨' {*draw = Character::new('║', foreground, background);}
            else if check == '╩' {*draw = Character::new('╬', foreground, background);}
            else if check == '╬' {*draw = Character::new('╬', foreground, background);}
            else                 {*draw = Character::new('║', foreground, background);}
        }
        {
            let pos: usize = y2*CHAR_SCRN_X_DIM + x;
            let draw: &mut Character = &mut self.screen_charframe[pos];
            let check: char = (*draw).codepoint;
            if      check == '═' {*draw = Character::new('╩', foreground, background);}
            else if check == '║' {*draw = Character::new('║', foreground, background);}
            else if check == '╔' {*draw = Character::new('╠', foreground, background);}
            else if check == '╗' {*draw = Character::new('╣', foreground, background);}
            else if check == '╚' {*draw = Character::new('╚', foreground, background);}
            else if check == '╝' {*draw = Character::new('╝', foreground, background);}
            else if check == '╞' {*draw = Character::new('╚', foreground, background);}
            else if check == '╠' {*draw = Character::new('╠', foreground, background);}
            else if check == '╡' {*draw = Character::new('╝', foreground, background);}
            else if check == '╣' {*draw = Character::new('╣', foreground, background);}
            else if check == '╥' {*draw = Character::new('║', foreground, background);}
            else if check == '╦' {*draw = Character::new('╬', foreground, background);}
            else if check == '╨' {*draw = Character::new('╨', foreground, background);}
            else if check == '╩' {*draw = Character::new('╩', foreground, background);}
            else if check == '╬' {*draw = Character::new('╬', foreground, background);}
            else                 {*draw = Character::new('╨', foreground, background);}
        }
    }

    //Place string in arbitrary location on screen
    pub fn draw_string(&mut self, s: &str, y: usize, x: usize, foreground: [u8;4], background: [u8;4]){
        let mut p = y*CHAR_SCRN_X_DIM + x;
        //Check validity
        if p + s.len() >= self.screen_charframe.len() {return;}
        //Place characters on screen
        for c in s.chars(){
            self.screen_charframe[p] = Character::new(c, foreground, background);
            p += 1;
        }
    }
}
impl<'a> Write for Screen<'a>{
    fn write_str(&mut self, s: &str) -> Result {
        self.string_print(s, COLR_WHITE, COLR_BLACK);
        return Ok(());
    }

    fn write_char(&mut self, c: char) -> Result {
        self.string_print(c.encode_utf8(&mut [0; 4]), COLR_WHITE, COLR_BLACK);
        return Ok(());
    }

    fn write_fmt(mut self: &mut Self, args: Arguments<'_>) -> Result {
        let r = core::fmt::write(&mut self, args);
        self.printbuffer_draw_render();
        return r;
    }
}
