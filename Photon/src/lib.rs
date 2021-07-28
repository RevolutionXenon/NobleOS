#![no_std]

pub mod font_handler;

use core::ptr::write_volatile;
use crate::font_handler::retrieve_font_bitmap;

// SCREEN LAYOUT CONSTANTS
//1280 x  720 Screen
/*const PIXL_SCRN_X_DIM: usize = 1280;                                 //PIXEL WIDTH OF SCREEN
const PIXL_SCRN_Y_DIM: usize = 720;                                    //PIXEL HEIGHT OF SCREEN
const PIXL_SCRN_B_DEP: usize = 4;                                      //PIXEL BIT DEPTH
const FOREGROUND:[u8; PIXL_SCRN_B_DEP] = [255, 255, 255, 0];           //COLOR WHITE TEXT
const BACKGROUND:[u8; PIXL_SCRN_B_DEP] = [4, 4, 56, 0];                //COLOR MAROON BACKGROUND
const BLACK:[u8; PIXL_SCRN_B_DEP] = [0, 0, 0, 0];                      //COLOR BLACK
const CHAR_SCRN_X_DIM: usize = 80;                                     //TEXT MODE WIDTH OF ENTIRE SCREEN
const CHAR_SCRN_Y_DIM: usize = 45;                                     //TEXT MODE HEIGHT OF ENTIRE SCREEN
const CHAR_PRNT_X_POS: usize = 1;                                      //TEXT MODE HORIZONTAL POSITION OF PRINT RESULT WINDOW
const CHAR_PRNT_Y_POS: usize = 2;                                      //TEXT MODE VERTICAL POSITION OF PRINT RESULT WINDOW
const CHAR_PRNT_X_DIM: usize = 78;                                     //TEXT MODE WIDTH OF PRINT RESULT WINDOW
const CHAR_PRNT_Y_DIM_DSP: usize = 40;                                 //TEXT MODE HEIGHT OF PRINT RESULT WINDOW ON SCREEN
const CHAR_PRNT_Y_DIM_MEM: usize = 200;                                //TEXT MODE HEIGHT OF PRINT RESULT WINDOW IN MEMORY
const CHAR_INPT_X_DIM: usize = 78;                                     //TEXT MODE WIDTH OF INPUT WINDOW
const CHAR_INPT_Y_DIM_MEM: usize = 20;                                 //TEXT MODE HEIGHT OF INPUT WINDOW IN MEMORY*/
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
pub const CHAR_INPT_X_DIM: usize = 118;                                //TEXT MODE WIDTH OF INPUT WINDOW
pub const CHAR_INPT_Y_DIM_MEM: usize = 20;                             //TEXT MODE HEIGHT OF INPUT WINDOW IN MEMORY

// GRAPHICS
//Draw a framebuffer to the screen
pub fn draw_pixelframe_to_hardwarebuffer(buffer: *mut u8, framebuffer: &[u8;PIXL_SCRN_X_DIM*PIXL_SCRN_Y_DIM* PIXL_SCRN_B_DEP]){
    unsafe{
        for i in 0..framebuffer.len(){
            write_volatile(buffer.add(i), framebuffer[i]);
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
    let stride = PIXL_SCRN_X_DIM;
    for byte_row in 0..16{
        for byte_column in 0..2{
            for bit in 0..8{
                if bitmap[byte_row*2 + byte_column] & (1 << bit) != 0{
                    let pixel_index = (y*16 + byte_row)*stride + (x*16 + byte_column*8 + bit);
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
pub fn draw_char_to_textframe_and_pixelframe(framebuffer: &mut [u8;PIXL_SCRN_X_DIM*PIXL_SCRN_Y_DIM*PIXL_SCRN_B_DEP], charbuffer: &mut [char;3600], codepoint: char, x: usize, y:usize, background: [u8;PIXL_SCRN_B_DEP], foreground: [u8;PIXL_SCRN_B_DEP]){
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
