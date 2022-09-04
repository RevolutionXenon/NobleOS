// PHOTON
// Photon is the Noble text graphics library:
// Draw to a frame buffer who's size decided at compile-time
// Character struct which holds a codepoint and color
// Screen struct which holds a frame buffer location and backlogs of text


// HEADER
//Flags
#![no_std]
#![allow(incomplete_features)]
#![allow(clippy::missing_safety_doc)]
#![feature(generic_const_exprs)]

//Modules
pub mod font_16_16;
pub mod formats;

//Imports
use core::fmt::Error;
use core::fmt::Arguments;
use core::fmt::Write;
use core::ptr::write_volatile;
use core::str;
use crate::font_16_16::retrieve_font_bitmap;

//Constants
pub const PHOTON_VERSION: & str = "vDEV-2021-09-04"; //CURRENT VERSION OF GRAPHICS LIBRARY


// TRAITS
//Color Format
pub trait ColorFormat:Clone+Copy {
}

//Character Format
pub trait CharacterFormat<Color: ColorFormat>:Clone+Copy {
    fn get_codepoint(&self) -> char;
    fn set_codepoint(&mut self, codepoint: char);
}

//Pixel Renderer
pub trait PixelRenderer<Color: ColorFormat> {
    unsafe fn render_pixel(&self, color: Color, y: usize, x: usize);
    unsafe fn render_line(&self, line: &[Color], y: usize, x: usize);
    unsafe fn render_image(&self, image: &[&[Color]], y: usize, x: usize);
    unsafe fn render_screen(&self, color: Color);
}

//Character Renderer
pub trait CharacterRenderer<Color: ColorFormat, Character: CharacterFormat<Color>> {
    fn render_character(&self, character: Character, y: usize, x: usize);
}


// SPECIFIC IMPLEMENTATIONS
//Color: BGRX
#[repr(C)]
#[derive(Default)]
#[derive(Copy, Clone)]
pub struct ColorBGRX {
    blue:     u8,
    green:    u8,
    red:      u8,
    reserved: u8,
}
impl ColorBGRX                 {
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self {
            blue,
            green,
            red,
            reserved: 0,
        }
    }
}
impl ColorFormat for ColorBGRX {
}

//Character: Two Tone
#[derive(Copy, Clone)]
pub struct CharacterTwoTone<Color: ColorFormat> {
    pub codepoint:  char,
    pub foreground: Color,
    pub background: Color,
}
impl<Color: ColorFormat> CharacterFormat<Color> for CharacterTwoTone<Color> {
    fn get_codepoint(&self) -> char {self.codepoint}
    fn set_codepoint(&mut self, codepoint: char) {self.codepoint = codepoint;}
}

//Pixel Render: Height then Width then Depth
pub struct PixelRendererHWD<Color: ColorFormat> {
    pub pointer: *mut Color,
    pub height:       usize,
    pub width:        usize,
}
impl<Color: ColorFormat> PixelRenderer<Color> for PixelRendererHWD<Color> {
    unsafe fn render_pixel(&self, color: Color, y: usize, x: usize) {
        if !(y<self.height && x<self.width) {return};
        write_volatile(self.pointer.add(y*self.width + x), color);
    }
    unsafe fn render_line(&self, line: &[Color], y: usize, x: usize) {
        for (i, pixel) in line.iter().enumerate() {
            self.render_pixel(*pixel, y, x+i)
        }
    }
    unsafe fn render_image(&self, image: &[&[Color]], y: usize, x: usize) {
        for (i, line) in image.iter().enumerate() {
            self.render_line(line, y+i, x)
        }
    }
    unsafe fn render_screen(&self, color: Color) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.render_pixel(color, y, x);
            }
        }
    }
}

//Character Renderer: Two Tone Characters, 16x16 Pixel Grid
pub struct CharacterTwoToneRenderer16x16<Color: ColorFormat+Default>                                                                                            {
    pub renderer: *const dyn PixelRenderer<Color>,
    pub height:           usize,
    pub width:            usize,
    pub y:                usize,
    pub x:                usize,
}
impl                                    <Color: ColorFormat+Default> CharacterRenderer<Color, CharacterTwoTone<Color>> for CharacterTwoToneRenderer16x16<Color> {
    fn render_character(&self, character: CharacterTwoTone<Color>, y: usize, x: usize) {
        //Find bitmap
        let bitmap = retrieve_font_bitmap(character.codepoint);
        //Loop through bitmap
        for i in 0..16 {
            let mut line = [Color::default();16];
            for j in 0..2 {
                for k in 0..8 {
                    let color = match bitmap[i*2 + j] & (1 << k) != 0 {
                        true => character.foreground,
                        false => character.background,
                    };
                    line[j*8+k] = color;
                    //unsafe {self.renderer.render_pixel(h, y*16+i, x*16+j*8+k);}
                }
            }
            unsafe {(*self.renderer).render_line(&line, (y<<4) + i + self.y, (x<<4) + self.x);}
        }
    }
}


// WINDOW STRUCTS
//Frame of Fixed Dimensions
pub struct FrameWindow<'s, const HEIGHT: usize, const WIDTH: usize, Color: ColorFormat, Character: CharacterFormat<Color>>                                                  {
    renderer: &'s dyn CharacterRenderer<Color, Character>,
    character_frame:  [[Character; WIDTH]; HEIGHT],
    y:                usize,
    x:                usize,
}
impl                  <'s, const HEIGHT: usize, const WIDTH: usize, Color: ColorFormat, Character: CharacterFormat<Color>> FrameWindow<'s, HEIGHT, WIDTH, Color, Character> {
    //CONSTRUCTOR
    pub fn new(renderer: &'s dyn CharacterRenderer<Color, Character>, whitespace: Character, y: usize, x: usize) -> Self {
        FrameWindow{
            renderer,
            character_frame: [[whitespace; WIDTH]; HEIGHT],
            y,
            x,
        }
    }

    // BASIC FUNCTION
    //Place Character in Frame
    pub fn place(&mut self, character: Character, y: usize, x: usize) {
        //Check valid character position
        if !(y < HEIGHT && x < WIDTH) {return;}
        //Set character
        self.character_frame[y][x] = character;
    }

    // HIGHER FUNCTIONS
    //Render Entire Frame
    pub fn render(&mut self) {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                self.renderer.render_character(self.character_frame[y][x], y+self.y, x+self.x);
            }
        }
    }

    //Draw character to characterframe screen and render it to physical screen
    pub fn character_place_render(&mut self, character: Character, y: usize, x: usize) {
        self.place(character, y, x);
        self.renderer.render_character(character, y, x);
    }

    //Horizontal Line
    pub fn horizontal_line(&mut self, y: usize, x1: usize, x2:usize, whitespace: Character) {
        {
            let check = self.character_frame[y][x1];
            let write = match check.get_codepoint() {
                '═' => '═', '║' => '╠', '╔' => '╔', '╗' => '╦',
                '╚' => '╚', '╝' => '╩', '╞' => '╞', '╠' => '╠',
                '╡' => '═', '╣' => '╬', '╥' => '╔', '╦' => '╦',
                '╨' => '╚', '╩' => '╩', '╬' => '╬',  _  => '╞',
            };
            self.character_frame[y][x1] = {let mut replace = whitespace; replace.set_codepoint(write); replace};
        }
        for x in x1+1..x2 {
            let check = self.character_frame[y][x];
            let write = match check.get_codepoint() {
                '═' => '═', '║' => '╬', '╔' => '╦', '╗' => '╦',
                '╚' => '╩', '╝' => '╩', '╞' => '═', '╠' => '╬',
                '╡' => '═', '╣' => '╬', '╥' => '╦', '╦' => '╦',
                '╨' => '╩', '╩' => '╩', '╬' => '╬',  _  => '═',
            };
            self.character_frame[y][x] = {let mut replace = whitespace; replace.set_codepoint(write); replace};
        }
        {
            let check = self.character_frame[y][x2];
            let write = match check.get_codepoint() {
                '═' => '═', '║' => '╣', '╔' => '╦', '╗' => '╗',
                '╚' => '╩', '╝' => '╝', '╞' => '═', '╠' => '╬',
                '╡' => '╡', '╣' => '╣', '╥' => '╗', '╦' => '╦',
                '╨' => '╝', '╩' => '╩', '╬' => '╬',  _  => '╡',
            };
            self.character_frame[y][x2] = {let mut replace = whitespace; replace.set_codepoint(write); replace};
        }
    }

    //Vertical Line
    pub fn vertical_line(&mut self, x: usize, y1: usize, y2:usize, whitespace: Character) {
        {
            let check = self.character_frame[y1][x];
            let write = match check.get_codepoint() {
                '═' => '╦', '║' => '║', '╔' => '╔', '╗' => '╗',
                '╚' => '╠', '╝' => '╣', '╞' => '╔', '╠' => '╠',
                '╡' => '╗', '╣' => '╣', '╥' => '╥', '╦' => '╦', 
                '╨' => '║', '╩' => '╬', '╬' => '╬',  _  => '╥',
            };
            self.character_frame[y1][x] = {let mut replace = whitespace; replace.set_codepoint(write); replace};
        }
        for y in y1+1..y2 {
            let check = self.character_frame[y][x];
            let write = match check.get_codepoint() {
                '═' => '╬', '║' => '║', '╔' => '╠', '╗' => '╣',
                '╚' => '╠', '╝' => '╣', '╞' => '╠', '╠' => '╠',
                '╡' => '╣', '╣' => '╣', '╥' => '║', '╦' => '╬',
                '╨' => '║', '╩' => '╬', '╬' => '╬',  _  => '║',
            };
            self.character_frame[y][x] = {let mut replace = whitespace; replace.set_codepoint(write); replace};
        }
        {
            let check = self.character_frame[y2][x];
            let write = match check.get_codepoint() {
                '═' => '╩', '║' => '║', '╔' => '╠', '╗' => '╣',
                '╚' => '╚', '╝' => '╝', '╞' => '╚', '╠' => '╠',
                '╡' => '╝', '╣' => '╣', '╥' => '║', '╦' => '╬',
                '╨' => '╨', '╩' => '╩', '╬' => '╬',  _  => '╨',
            };
            self.character_frame[y2][x] = {let mut replace = whitespace; replace.set_codepoint(write); replace};
        }
    }

    //Place string in arbitrary location on screen
    pub fn horizontal_string(&mut self, string: &str, y: usize, x: usize, whitespace: Character) {
        if y >= HEIGHT {return;}
        let mut p = x;
        //Place characters on screen
        for c in string.chars() {
            //Check validity
            if p >= WIDTH {return;}
            //Draw
            self.character_frame[y][p] = {let mut replace = whitespace; replace.set_codepoint(c); replace};
            //Move Position
            p += 1;
        }
    }
}

//Print Window of Fixed Dimensions
pub struct PrintWindow<const LINES: usize, const HEIGHT: usize, const WIDTH: usize, Color: ColorFormat, Character: CharacterFormat<Color>>                                                                   {
    screen:   *const dyn CharacterRenderer<Color, Character>,
    print_buffer:         [[Character; WIDTH]; LINES],
    print_y:              usize,
    print_x:              usize,
    pub write_whitespace: Character,
    pub line_whitespace:  Character,
    pub y:                usize,
    pub x:                usize,
}
impl                  <const LINES: usize, const HEIGHT: usize, const WIDTH: usize, Color: ColorFormat, Character: CharacterFormat<Color>>           PrintWindow<LINES, HEIGHT, WIDTH, Color, Character> {
    //CONSTRUCTOR
    pub fn new(character_renderer: *const dyn CharacterRenderer<Color, Character>, write_whitespace: Character, line_whitespace: Character, y: usize, x: usize) -> Self{
        PrintWindow{
            screen: character_renderer,
            print_buffer: [[line_whitespace; WIDTH]; LINES],
            print_y: LINES - HEIGHT,
            print_x: 0,
            write_whitespace,
            line_whitespace,
            y,
            x,
        }
    }

    // BASIC FUNCTION
    //Input string to print buffer
    pub fn push(&mut self, string: &str, whitespace: Character) -> bool {
        let mut render:bool = false;
        //begin writing to the buffer
        for codepoint in string.chars() {
            let control = (codepoint as u32) < 0x20;
            let control_newline = codepoint=='\n' || codepoint=='\r';
            let control_backwards = codepoint=='\x08';
            let end_forward = self.print_x >= WIDTH;
            let end_backward = self.print_x == 0;
            //move to next line (line feed, carriage return, end of line moving forward)
            if (control_newline || end_forward) && !control_backwards {
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
                self.print_x -= 1;
                //remove character
                self.print_buffer[LINES - 1][self.print_x] = whitespace;
            }
            //print character (not: line Feed, carriage return, backspace)
            if !control {
                //place character
                self.print_buffer[LINES - 1][self.print_x] = {let mut clone = whitespace; clone.set_codepoint(codepoint); clone};
                //move xbuffer right
                self.print_x += 1;
            }
        }
        render
    }

    // HIGHER FUNCTIONS
    //Render entire printbuffer
    pub fn render(&mut self) {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                unsafe{& *self.screen}.render_character(self.print_buffer[self.print_y+y][x], y+self.y, x+self.x);
            }
        }
    }

    //Input string to print buffer, draw it to characterframe, and render it to physical screen
    pub fn push_render(&mut self, character: &str, whitespace: Character) {
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
impl                  <const LINES: usize, const HEIGHT: usize, const WIDTH: usize, Color: ColorFormat, Character: CharacterFormat<Color>> Write for PrintWindow<LINES, HEIGHT, WIDTH, Color, Character> {
    fn write_str(&mut self, string: &str) -> Result<(), Error> {
        self.push(string, self.write_whitespace);
        Ok(())
    }

    fn write_char(&mut self, codepoint: char) -> Result<(), Error> {
        self.push(codepoint.encode_utf8(&mut [0; 4]), self.write_whitespace);
        Ok(())
    }

    fn write_fmt(mut self: &mut Self, arguments: Arguments<'_>) -> Result<(), Error> {
        let r = core::fmt::write(&mut self, arguments);
        self.render();
        r
    }
}

//Input Window
pub struct InputWindow<const LENGTH: usize, const WIDTH: usize, Color: ColorFormat, Character: CharacterFormat<Color>>                                                  where [(); LENGTH*4]: {
    screen: *const dyn CharacterRenderer<Color, Character>,
    input_buffer:   [Character; LENGTH],
    input_p:        usize,
    y:              usize,
    x:              usize,
}
impl                  <const LENGTH: usize, const WIDTH: usize, Color: ColorFormat, Character: CharacterFormat<Color>> InputWindow<LENGTH, WIDTH, Color, Character> where [(); LENGTH*4]: {
    // CONSTRUCTOR
    pub fn new(character_renderer: *const dyn CharacterRenderer<Color, Character>, whitespace: Character, y: usize, x: usize) -> Self {
        InputWindow {
            screen: character_renderer,
            input_buffer: [whitespace; LENGTH],
            input_p: 0,
            y,
            x,
        }
    }

    // BASIC FUNCTION
    //Input character to input stack
    pub fn push(&mut self, character: Character, whitespace: Character) -> usize {
        let mut render:usize = 0;
        //control character booleans
        let control:bool = (character.get_codepoint() as u32) < 0x20;                                  //Determines if character is a control character
        let control_newline:bool = character.get_codepoint()=='\n' || character.get_codepoint()=='\r'; //Determines if character is a newline character
        let printable:bool = !control || control_newline;                                              //Determines if a character is printable
        let control_backwards:bool = character.get_codepoint()=='\x08';                                //Determines if character types backwards (i.e. backspace)
        let end_forward:bool = self.input_p >= self.input_buffer.len();                                //Determines if the end of the stack is reached and position can no longer move forward
        let end_backward:bool = self.input_p == 0;                                                     //Determines if the beginning of the stack is reached and position can no longer move backwards
        //add printable character to stack
        if printable && !end_forward {
            self.input_buffer[self.input_p] = character;
            render = self.input_p;
            self.input_p += 1;
        }
        //backspace handling
        else if control_backwards && !end_backward {
            self.input_p -= 1;
            render = self.input_p;
            self.input_buffer[self.input_p] = whitespace;
        }
        render
    }

    // HIGHER FUNCTIONS
    //Input character to input stack and render it
    pub fn push_render(&mut self, character: Character, whitespace: Character) {
        let p: usize = self.push(character, whitespace);
        if p % WIDTH == 0 || p % WIDTH == WIDTH - 1 {
            self.render(whitespace);
        }
        else {
            unsafe {& *self.screen}.render_character(self.input_buffer[p%WIDTH], self.y, p+self.x);
        }
    }

    //Render entire inputstack
    pub fn render(&mut self, whitespace: Character) {
        let overhang: usize = self.input_buffer.len() % WIDTH;
        let full: usize = self.input_buffer.len() - overhang;
        let line_width: usize = if self.input_p < full {WIDTH} else {overhang};
        for x in 0..line_width {
            unsafe {& *self.screen}.render_character(self.input_buffer[self.input_p/WIDTH+x], self.y, x+self.x);
        }
        for x in line_width..WIDTH {
            unsafe {& *self.screen}.render_character(whitespace, self.y, x+self.x)
        }
    }

    //Remove all information from input stack and replace with Character given
    pub fn flush(&mut self, whitespace: Character) {
        for i in 0..LENGTH {
            self.input_buffer[i] = whitespace;
        }
        self.input_p = 0;
        self.render(whitespace);
    }

    //Return contents of the input stack as char array
    pub fn to_chararray<'f>(&mut self, buffer: &'f mut [char; LENGTH]) -> &'f [char] {
        for (i, pos) in buffer.iter_mut().enumerate().take(LENGTH) {
            *pos = self.input_buffer[i].get_codepoint();
        }
        &buffer[0..self.input_p]
    }

    //Return contents of the input stack as a string slice
    pub fn to_str<'f>(&mut self, buffer: &'f mut [u8; LENGTH*4]) -> Result<&'f str, &'static str> {
        let mut p: usize = 0;
        for i in 0..self.input_p {
            let c = self.input_buffer[i].get_codepoint();
            let mut a = [0u8; 4];
            let l = c.len_utf8();
            char::encode_utf8(c, &mut a);
            for j in a.iter().take(l) {
                buffer[p] = *j;
                p += l;
            }
        }
        return str::from_utf8(&buffer[0..p]).map_err(|_| "Input: Error processing characters into string slice.");
    }

    //Return the length of the input stack
    pub fn length(&self) -> usize {
        self.input_buffer.len()
    }
}


// SPECIFIC COLOR FORMATS
//Pixel Format: Blue, Green, Red, Reserved
pub const COLOR_BGRX_BLACK:   ColorBGRX = ColorBGRX::new(0x00, 0x00, 0x00); //BLACK
pub const COLOR_BGRX_RED:     ColorBGRX = ColorBGRX::new(0xFF, 0x00, 0x00); //RED
pub const COLOR_BGRX_GREEN:   ColorBGRX = ColorBGRX::new(0x00, 0xFF, 0x00); //GREEN
pub const COLOR_BGRX_BLUE:    ColorBGRX = ColorBGRX::new(0x00, 0x00, 0xFF); //BLUE
pub const COLOR_BGRX_TEAL:    ColorBGRX = ColorBGRX::new(0x00, 0xFF, 0xFF); //TEAL
pub const COLOR_BGRX_MAGENTA: ColorBGRX = ColorBGRX::new(0xFF, 0x00, 0xFF); //MAGENTA
pub const COLOR_BGRX_YELLOW:  ColorBGRX = ColorBGRX::new(0xFF, 0xFF, 0x00); //YELLOW
pub const COLOR_BGRX_WHITE:   ColorBGRX = ColorBGRX::new(0xFF, 0xFF, 0xFF); //WHITE
