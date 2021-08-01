use crate::*;

pub fn draw_hline_to_textframe(textframe: &mut [char;CHAR_SCRN_X_DIM*CHAR_SCRN_Y_DIM], y: usize, x1: usize, x2:usize){
    {
        let pos: usize = y*CHAR_SCRN_X_DIM + x1;
        let check: char = textframe[pos];
        if      check == '═' {textframe[pos] = '═';}
        else if check == '║' {textframe[pos] = '╠';}
        else if check == '╔' {textframe[pos] = '╔';}
        else if check == '╗' {textframe[pos] = '╦';}
        else if check == '╚' {textframe[pos] = '╚';}
        else if check == '╝' {textframe[pos] = '╩';}
        else if check == '╞' {textframe[pos] = '╞';}
        else if check == '╠' {textframe[pos] = '╠';}
        else if check == '╡' {textframe[pos] = '═';}
        else if check == '╣' {textframe[pos] = '╬';}
        else if check == '╥' {textframe[pos] = '╔';}
        else if check == '╦' {textframe[pos] = '╦';}
        else if check == '╨' {textframe[pos] = '╚';}
        else if check == '╩' {textframe[pos] = '╩';}
        else if check == '╬' {textframe[pos] = '╬';}
        else                 {textframe[pos] = '╞';}
    }
    for x in x1+1..x2{
        let pos: usize = y*CHAR_SCRN_X_DIM + x;
        let check: char = textframe[pos];
        if      check == '═' {textframe[pos] = '═';}
        else if check == '║' {textframe[pos] = '╬';}
        else if check == '╔' {textframe[pos] = '╦';}
        else if check == '╗' {textframe[pos] = '╦';}
        else if check == '╚' {textframe[pos] = '╩';}
        else if check == '╝' {textframe[pos] = '╩';}
        else if check == '╞' {textframe[pos] = '═';}
        else if check == '╠' {textframe[pos] = '╬';}
        else if check == '╡' {textframe[pos] = '═';}
        else if check == '╣' {textframe[pos] = '╬';}
        else if check == '╥' {textframe[pos] = '╦';}
        else if check == '╦' {textframe[pos] = '╦';}
        else if check == '╨' {textframe[pos] = '╩';}
        else if check == '╩' {textframe[pos] = '╩';}
        else if check == '╬' {textframe[pos] = '╬';}
        else                 {textframe[pos] = '═';}
    }
    {
        let pos: usize = y*CHAR_SCRN_X_DIM + x2;
        let check: char = textframe[pos];
        if      check == '═' {textframe[pos] = '═';}
        else if check == '║' {textframe[pos] = '╣';}
        else if check == '╔' {textframe[pos] = '╦';}
        else if check == '╗' {textframe[pos] = '╗';}
        else if check == '╚' {textframe[pos] = '╩';}
        else if check == '╝' {textframe[pos] = '╝';}
        else if check == '╞' {textframe[pos] = '═';}
        else if check == '╠' {textframe[pos] = '╬';}
        else if check == '╡' {textframe[pos] = '╡';}
        else if check == '╣' {textframe[pos] = '╣';}
        else if check == '╥' {textframe[pos] = '╗';}
        else if check == '╦' {textframe[pos] = '╦';}
        else if check == '╨' {textframe[pos] = '╝';}
        else if check == '╩' {textframe[pos] = '╩';}
        else if check == '╬' {textframe[pos] = '╬';}
        else                 {textframe[pos] = '╡';}
    }
}

pub fn draw_vline_to_textframe(textframe: &mut [char;CHAR_SCRN_X_DIM*CHAR_SCRN_Y_DIM], x: usize, y1: usize, y2:usize){
    {
        let pos: usize = y1*CHAR_SCRN_X_DIM + x;
        let check: char = textframe[pos];
        if      check == '═' {textframe[pos] = '╦';}
        else if check == '║' {textframe[pos] = '║';}
        else if check == '╔' {textframe[pos] = '╔';}
        else if check == '╗' {textframe[pos] = '╗';}
        else if check == '╚' {textframe[pos] = '╠';}
        else if check == '╝' {textframe[pos] = '╣';}
        else if check == '╞' {textframe[pos] = '╔';}
        else if check == '╠' {textframe[pos] = '╠';}
        else if check == '╡' {textframe[pos] = '╗';}
        else if check == '╣' {textframe[pos] = '╣';}
        else if check == '╥' {textframe[pos] = '╥';}
        else if check == '╦' {textframe[pos] = '╦';}
        else if check == '╨' {textframe[pos] = '║';}
        else if check == '╩' {textframe[pos] = '╬';}
        else if check == '╬' {textframe[pos] = '╬';}
        else                 {textframe[pos] = '╥';}
    }
    for y in y1+1..y2{
        let pos: usize = y*CHAR_SCRN_X_DIM + x;
        let check: char = textframe[pos];
        if      check == '═' {textframe[pos] = '╬';}
        else if check == '║' {textframe[pos] = '║';}
        else if check == '╔' {textframe[pos] = '╠';}
        else if check == '╗' {textframe[pos] = '╣';}
        else if check == '╚' {textframe[pos] = '╠';}
        else if check == '╝' {textframe[pos] = '╣';}
        else if check == '╞' {textframe[pos] = '╠';}
        else if check == '╠' {textframe[pos] = '╠';}
        else if check == '╡' {textframe[pos] = '╣';}
        else if check == '╣' {textframe[pos] = '╣';}
        else if check == '╥' {textframe[pos] = '║';}
        else if check == '╦' {textframe[pos] = '╬';}
        else if check == '╨' {textframe[pos] = '║';}
        else if check == '╩' {textframe[pos] = '╬';}
        else if check == '╬' {textframe[pos] = '╬';}
        else                 {textframe[pos] = '║';}
    }
    {
        let pos: usize = y2*CHAR_SCRN_X_DIM + x;
        let check: char = textframe[pos];
        if      check == '═' {textframe[pos] = '╩';}
        else if check == '║' {textframe[pos] = '║';}
        else if check == '╔' {textframe[pos] = '╠';}
        else if check == '╗' {textframe[pos] = '╣';}
        else if check == '╚' {textframe[pos] = '╚';}
        else if check == '╝' {textframe[pos] = '╝';}
        else if check == '╞' {textframe[pos] = '╚';}
        else if check == '╠' {textframe[pos] = '╠';}
        else if check == '╡' {textframe[pos] = '╝';}
        else if check == '╣' {textframe[pos] = '╣';}
        else if check == '╥' {textframe[pos] = '║';}
        else if check == '╦' {textframe[pos] = '╬';}
        else if check == '╨' {textframe[pos] = '╨';}
        else if check == '╩' {textframe[pos] = '╩';}
        else if check == '╬' {textframe[pos] = '╬';}
        else                 {textframe[pos] = '╨';}
    }
}