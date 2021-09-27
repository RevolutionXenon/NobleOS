

// HEADER
//Imports
use crate::*;

//constants
static PIC_EOI: u8 = 0x20; //End of interrupt command code.

pub unsafe fn remap(offset_1: u8, offset_2: u8) -> Result<(), &'static str> {
    if offset_1 % 8 != 0 || offset_2 % 8 != 0 {return Err("PIC: Remap offsets unaligned.")}
    //Save Masks
    let mask_1 = PORT_PIC1_DATA.read();
    let mask_2 = PORT_PIC2_DATA.read();
    //Start Initialization Sequence
    PORT_PIC1_COMMAND.write(0x11);  io_wait(); //ICW1: Start in cascade mode
    PORT_PIC2_COMMAND.write(0x11);  io_wait(); //ICW2: Start in cascade mode
    PORT_PIC1_DATA.write(offset_1); io_wait(); //ICW2: Write PIC1 offset
    PORT_PIC2_DATA.write(offset_2); io_wait(); //ICW2: Write PIC2 offset
    PORT_PIC1_DATA.write(0x04);     io_wait(); //ICW3: Write PIC1 PIC2 position (IRQ-2)
    PORT_PIC2_DATA.write(0x02);     io_wait(); //ICW3: Write PIC2 cascade identity
    PORT_PIC1_DATA.write(0x01);     io_wait(); //ICW4: Write PIC1 mode (8086 mode)
    PORT_PIC2_DATA.write(0x01);     io_wait(); //ICW4: Write PIC2 mode (8086 mode)
    //Rewrite Masks
    PORT_PIC1_DATA.write(mask_1);   io_wait();
    PORT_PIC2_DATA.write(mask_2);   io_wait();
    //Return
    Ok(())
}

pub unsafe fn disable_irq(irq: u8) -> Result<(), &'static str> {
    if irq < 8 {
        PORT_PIC1_DATA.write(PORT_PIC1_DATA.read() | (1 << irq));
    }
    else if irq < 16 {
        PORT_PIC2_DATA.write(PORT_PIC2_DATA.read() | (1 << (irq-8)));
    }
    else {return Err("PIC: IRQ out of bounds on set.")}
    Ok(())
}
pub unsafe fn enable_irq (irq: u8) -> Result<(), &'static str> {
    if      irq <  8 {PORT_PIC1_DATA.write(PORT_PIC1_DATA.read() & !(1 << irq));}
    else if irq < 16 {PORT_PIC2_DATA.write(PORT_PIC2_DATA.read() & !(1 << (irq-8)));}
    else {return Err("PIC: IRQ out of bounds on clear.")}
    Ok(())
}

pub unsafe fn end_irq(irq: u8) -> Result<(), &'static str> {
    if irq < 16 {
        if irq > 7 { PORT_PIC2_COMMAND.write(PIC_EOI);}
        PORT_PIC1_COMMAND.write(PIC_EOI);
        Ok(())
    }
    else {Err("PIC: IRQ out of bounds on end of interrupt.")}
}