#![no_std]

pub mod font_handler;
pub mod line_draw;

use core::{convert::TryInto, fmt::{Write, Result, Arguments}, ptr::write_volatile};
use crate::font_handler::retrieve_font_bitmap;

// SCREEN LAYOUT CONSTANTS
//1920 x 1080 Screen
pub const PIXL_SCRN_X_DIM: usize = 1920;                               //PIXEL WIDTH OF SCREEN
pub const PIXL_SCRN_Y_DIM: usize = 1080;                               //PIXEL HEIGHT OF SCREEN
pub const PIXL_SCRN_B_DEP: usize = 4;                                  //PIXEL BIT DEPTH

pub const COLR_FORE:[u8; PIXL_SCRN_B_DEP] = [0xFF, 0xFF, 0xFF, 0x00];  //COLOR WHITE TEXT
pub const COLR_BACK:[u8; PIXL_SCRN_B_DEP] = [0x00, 0x00, 0x00, 0x00];  //COLOR BLACK BACKGROUND

pub const CHAR_SCRN_X_DIM: usize = 120;                                //TEXT MODE WIDTH OF ENTIRE SCREEN
pub const CHAR_SCRN_Y_DIM: usize = 67;                                 //TEXT MODE HEIGHT OF ENTIRE SCREEN

pub const CHAR_PRNT_X_POS: usize = 1;                                  //TEXT MODE HORIZONTAL POSITION OF PRINT RESULT WINDOW
pub const CHAR_PRNT_Y_POS: usize = 2;                                  //TEXT MODE VERTICAL POSITION OF PRINT RESULT WINDOW
pub const CHAR_PRNT_X_DIM: usize = 118;                                //TEXT MODE WIDTH OF PRINT RESULT WINDOW
pub const CHAR_PRNT_Y_DIM_DSP: usize = 62;                             //TEXT MODE HEIGHT OF PRINT RESULT WINDOW ON SCREEN
pub const CHAR_PRNT_Y_DIM_MEM: usize = 200;                            //TEXT MODE HEIGHT OF PRINT RESULT WINDOW IN MEMORY

pub const CHAR_INPT_X_POS: usize = 1;                                  //TEXT MODE HORIZONTAL POSITION OF INPUT WINDOW
pub const CHAR_INPT_Y_POS: usize = 65;                                 //TEXT MODE VERTICAL POSITION OF INPUT WINDOW
pub const CHAR_INPT_X_DIM: usize = 118;                                //TEXT MODE WIDTH OF INPUT WINDOW
pub const CHAR_INPT_Y_DIM: usize = 1;                                  //TEXT MODE HEIGHT OF INPUT WINDOW
pub const CHAR_INPT_Y_DIM_MEM: usize = 20;                             //TEXT MODE HEIGHT OF INPUT WINDOW IN MEMORY

// STRUCTS
//Character
#[derive(Copy, Clone)]
pub struct Character {
    codepoint: char,
    foreground: [u8; PIXL_SCRN_B_DEP],
    background: [u8; PIXL_SCRN_B_DEP]
}
impl Character{
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
    pub screen_physical: &'a mut [u8;PIXL_SCRN_Y_DIM*PIXL_SCRN_X_DIM*PIXL_SCRN_B_DEP],
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
                        write_volatile(
                            &mut self.screen_physical[pixel_index*PIXL_SCRN_B_DEP..(pixel_index+1)*PIXL_SCRN_B_DEP].try_into().unwrap(), 
                            color
                        );
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
    pub fn character_handle(&mut self, c: Character) -> usize{
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
    pub fn string_handle(&mut self, s: &str, foreground: [u8;4], background: [u8;4]) -> bool{
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
    pub fn character_handle_draw_render(&mut self, c: Character){
        let p:usize = self.character_handle(c);
        if p % CHAR_INPT_X_DIM == 0 || p % CHAR_INPT_X_DIM == CHAR_INPT_X_DIM - 1{
            self.inputstack_draw_render();
        }
        else{
            self.character_draw_render(self.input_stack[p%CHAR_INPT_X_DIM], CHAR_INPT_Y_POS, CHAR_INPT_X_POS + p);
        }
    }

    //Input string to print buffer, draw it to characterframe, and render it to physical screen
    pub fn string_handle_draw_render(&mut self, s: &str, foreground: [u8;4], background: [u8;4]){
        self.string_handle(s, foreground, background);
        self.printbuffer_draw_render();
    }
}
impl<'a> Write for Screen<'a>{
    fn write_str(&mut self, s: &str) -> Result {
        self.string_handle_draw_render(s, COLR_FORE, COLR_BACK);
        return Ok(());
    }

    fn write_char(&mut self, c: char) -> Result {
        self.write_str(c.encode_utf8(&mut [0; 4]))
    }

    fn write_fmt(mut self: &mut Self, args: Arguments<'_>) -> Result {
        core::fmt::write(&mut self, args)
    }
}


// GRAPHICS
//Draw a framebuffer to the screen
pub fn draw_pixelframe_to_hardwarebuffer(buffer: *mut u8, framebuffer: &[u8;PIXL_SCRN_X_DIM*PIXL_SCRN_Y_DIM*PIXL_SCRN_B_DEP]){
    unsafe{
        for i in 0..PIXL_SCRN_X_DIM*PIXL_SCRN_Y_DIM* PIXL_SCRN_B_DEP{
            write_volatile(buffer.add(i), framebuffer[i]);
        }
    }
}

//Draw an area of pixels containing a character from a framebuffer to the screen
pub fn draw_char_from_pixelframe_to_hardwarebuffer(buffer: *mut u8, framebuffer: &[u8;PIXL_SCRN_X_DIM*PIXL_SCRN_Y_DIM*PIXL_SCRN_B_DEP], x: usize, y: usize){
    //Check valid character position
    if x >= CHAR_SCRN_X_DIM || y >= CHAR_SCRN_Y_DIM {
        return;
    }
    unsafe{
        for i in y*16..y*16+16{
            for j in x*16..x*16+16{
                for k in 0..PIXL_SCRN_B_DEP{
                    let p = i*PIXL_SCRN_X_DIM*PIXL_SCRN_B_DEP + j*PIXL_SCRN_B_DEP + k;
                    write_volatile(buffer.add(p), framebuffer[p]);
                }
            }
        }
    }
}

//Fill the screen with color
pub fn draw_color_to_pixelframe(framebuffer: &mut [u8;PIXL_SCRN_X_DIM*PIXL_SCRN_Y_DIM*PIXL_SCRN_B_DEP], color: [u8;PIXL_SCRN_B_DEP]) {
    for i in 0..PIXL_SCRN_X_DIM*PIXL_SCRN_Y_DIM{
        framebuffer[i*PIXL_SCRN_B_DEP..(i+1)*PIXL_SCRN_B_DEP].copy_from_slice(&color)
    }
}

//Draw an individual text character with a foreground mask only
pub fn draw_charf_to_pixelframe(framebuffer: &mut [u8;PIXL_SCRN_X_DIM*PIXL_SCRN_Y_DIM*PIXL_SCRN_B_DEP], codepoint: char, x: usize, y: usize, foreground: [u8;PIXL_SCRN_B_DEP]){
    //Check valid character position
    if x >= CHAR_SCRN_X_DIM || y >= CHAR_SCRN_Y_DIM {
        return;
    }
    let bitmap = retrieve_font_bitmap(&codepoint);
    for byte_row in 0..16{
        for byte_column in 0..2{
            for bit in 0..8{
                if bitmap[byte_row*2 + byte_column] & (1 << bit) != 0{
                    let pixel_index = (y*16 + byte_row)*PIXL_SCRN_X_DIM + (x*16 + byte_column*8 + bit);
                    framebuffer[pixel_index*PIXL_SCRN_B_DEP..(pixel_index+1)*PIXL_SCRN_B_DEP].copy_from_slice(&foreground);
                }
            }
        }
    }
}

//Draw an individual text character with a foreground and background mask
pub fn draw_charfb_to_pixelframe(framebuffer: &mut [u8;PIXL_SCRN_X_DIM*PIXL_SCRN_Y_DIM*PIXL_SCRN_B_DEP], codepoint: char, x: usize, y: usize, background: [u8;PIXL_SCRN_B_DEP], foreground: [u8;PIXL_SCRN_B_DEP]){
    //Check valid character position
    if x >= CHAR_SCRN_X_DIM || y >= CHAR_SCRN_Y_DIM {
        return;
    }
    let bitmap = retrieve_font_bitmap(&codepoint);
    let stride = PIXL_SCRN_X_DIM;
    for byte_row in 0..16{
        for byte_column in 0..2{
            for bit in 0..8{
                let pixel_index = (y*16 + byte_row)*stride + (x*16 + byte_column*8 + bit);
                if bitmap[byte_row*2 + byte_column] & (1 << bit) != 0{
                    framebuffer[pixel_index*PIXL_SCRN_B_DEP..(pixel_index+1)*PIXL_SCRN_B_DEP].copy_from_slice(&foreground);
                }
                else{
                    framebuffer[pixel_index*PIXL_SCRN_B_DEP..(pixel_index+1)*PIXL_SCRN_B_DEP].copy_from_slice(&background);
                }
            }
        }
    }
}

//Place a character into a character buffer and update it in a corresponding frame buffer
pub fn draw_char_to_textframe_and_pixelframe(framebuffer: &mut [u8;PIXL_SCRN_X_DIM*PIXL_SCRN_Y_DIM*PIXL_SCRN_B_DEP], charbuffer: &mut [char;CHAR_SCRN_X_DIM*CHAR_SCRN_Y_DIM], codepoint: char, x: usize, y:usize, background: [u8;PIXL_SCRN_B_DEP], foreground: [u8;PIXL_SCRN_B_DEP]){
    charbuffer[y* CHAR_SCRN_X_DIM +x]=codepoint;
    draw_charfb_to_pixelframe(framebuffer, charbuffer[y* CHAR_SCRN_X_DIM +x], x, y, background, foreground);
}

//Draw a character buffer to a corresponding frame buffer
pub fn draw_textframe_to_pixelframe(framebuffer: &mut [u8;PIXL_SCRN_X_DIM*PIXL_SCRN_Y_DIM*PIXL_SCRN_B_DEP], buffer: &[char;CHAR_SCRN_X_DIM*CHAR_SCRN_Y_DIM], background: [u8;PIXL_SCRN_B_DEP], foreground: [u8;PIXL_SCRN_B_DEP]){
    draw_color_to_pixelframe(framebuffer, COLR_BACK);
    for y in 0..CHAR_SCRN_Y_DIM{
        for x in 0..CHAR_SCRN_X_DIM{
            draw_charfb_to_pixelframe(framebuffer, buffer[y* CHAR_SCRN_X_DIM + x], x, y, background, foreground);
        }
    }
}

//Print a string to a buffer which flows from the bottom up and deletes the first line
pub fn print_str_to_textbuffer(buffer: &mut [char], dimx: usize, dimy: usize, xbuffer: &mut usize, text: &str){
    //check that attempting to write outside the buffer will not occur
    if dimx*dimy > buffer.len(){
        return;
    }
    //begin writing to the buffer
    for c in text.chars(){
        let control = (c as u32) < 0x20;
        let control_newline = c=='\n' || c=='\r';
        let control_backwards = c=='\x08';
        let end_forward = *xbuffer >= dimx;
        let end_backward = *xbuffer <= 0;
        //move to next line (line feed, carriage return, end of line moving forward)
        if (control_newline || end_forward) &! control_backwards {
            //reset xbuffer
            *xbuffer=0;
            //move buffer up
            for y in 0..dimy-1{
                for x in 0..dimx{
                    buffer[y*dimx+x]= buffer[(y+1)*dimx+x];
                }
            }
            //fill last line with spaces
            for x in 0..dimx{
                buffer[(dimy-1)*dimx+x]=' ';
            }
        }
        //move to previous line (beginning of line moving backward)
        if control_backwards && end_backward {
            //reset xbuffer
            *xbuffer = dimx;
            //move buffer down
            for y in (1..dimy).rev(){
                for x in 0..dimx{
                    buffer[y*dimx+x]= buffer[(y-1)*dimx+x];
                }
            }
            //fill first line with spaces
            for x in 0..dimx{
                buffer[x]=' ';
            }
        }
        //unprint character (backspace)
        if control_backwards {
            //move xbuffer back one
            *xbuffer = *xbuffer - 1;
            //remove character
            buffer[(dimy - 1) * dimx + *xbuffer] = ' ';
        }
        //print character (not: line Feed, carriage Return, backspace)
        if !control{
            //place character
            buffer[(dimy - 1) * dimx + *xbuffer] = c;
            //move xbuffer right
            *xbuffer = *xbuffer + 1;
        }
    }
}

//Print a character to a stack which flows from the top down and halts (no heed paid to dimensions)
pub fn print_char_to_textstack(stack: &mut [char], pbuffer: &mut usize, c: char){
    let control:bool = (c as u32) < 0x20;             //Determines if character is a control character
    let control_newline:bool = c=='\n' || c=='\r';    //Determines if character is a newline character
    let printable:bool = !control || control_newline; //Determines if a character is printable
    let control_backwards:bool = c=='\x08';           //Determines if character types backwards (i.e. backspace)
    let end_forward:bool = *pbuffer >= stack.len();   //Determines if the end of the stack is reached and position can no longer move forward
    let end_backward:bool = *pbuffer <= 0;            //Determines if the beginning of the stack is reached and position can no longer move backwards

    //add printable character to stack
    if printable && !end_forward {
        stack[*pbuffer] = c;
        *pbuffer = *pbuffer + 1;
    }
    //backspace handling
    else if control_backwards && !end_backward {
        *pbuffer = *pbuffer - 1;
        stack[*pbuffer] = ' ';
    }
}

//Input character
pub fn input_character(hardwarebuffer: *mut u8, framebuffer: &mut[u8;PIXL_SCRN_X_DIM*PIXL_SCRN_Y_DIM*PIXL_SCRN_B_DEP], charbuffer: &mut[char;CHAR_SCRN_X_DIM*CHAR_SCRN_Y_DIM], stack: &mut [char], pbuffer: &mut usize, c: char){
    let control:bool = (c as u32) < 0x20;             //Determines if character is a control character
    let control_newline:bool = c=='\n' || c=='\r';    //Determines if character is a newline character
    let printable:bool = !control || control_newline; //Determines if a character is printable
    let control_backwards:bool = c=='\x08';           //Determines if character types backwards (i.e. backspace)
    let end_forward:bool = *pbuffer >= stack.len();   //Determines if the end of the stack is reached and position can no longer move forward
    let end_backward:bool = *pbuffer <= 0;            //Determines if the beginning of the stack is reached and position can no longer move backwards

    //add printable character to stack
    if printable && !end_forward {
        stack[*pbuffer] = c;
        draw_char_to_textframe_and_pixelframe(framebuffer, charbuffer, c, CHAR_INPT_X_POS+*pbuffer, CHAR_INPT_Y_POS, COLR_BACK, COLR_FORE);
        draw_char_from_pixelframe_to_hardwarebuffer(hardwarebuffer, framebuffer, CHAR_INPT_X_POS+*pbuffer, CHAR_INPT_Y_POS);
        *pbuffer = *pbuffer + 1;
    }
    //backspace handling
    else if control_backwards && !end_backward {
        *pbuffer = *pbuffer - 1;
        stack[*pbuffer] = ' ';
        draw_char_to_textframe_and_pixelframe(framebuffer, charbuffer, ' ', CHAR_INPT_X_POS+*pbuffer, CHAR_INPT_Y_POS, COLR_BACK, COLR_FORE);
        draw_char_from_pixelframe_to_hardwarebuffer(hardwarebuffer, framebuffer, CHAR_INPT_X_POS+*pbuffer, CHAR_INPT_Y_POS);
    }
}