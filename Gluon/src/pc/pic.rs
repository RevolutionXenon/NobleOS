// GLUON: x86-64 PIC
// Functions related to the handling of the Programmable Interrupt Controller


// HEADER
//Imports
use crate::pc::ports::io_wait;
use crate::pc::ports::PIC1_COMMAND as COMMAND_PORT_1;
use crate::pc::ports::PIC2_COMMAND as COMMAND_PORT_2;
use crate::pc::ports::PIC1_DATA as DATA_PORT_1;
use crate::pc::ports::PIC2_DATA as DATA_PORT_2;
use crate::x86_64::port::*;


// PROGRAMMABLE INTERRUPT CONTROLLER
//Remap PIC to Different Interrupt Vectors
pub unsafe fn remap(pic_1_offset: u8, pic_2_offset: u8) -> Result<(), &'static str> {
    if pic_1_offset % 8 != 0 || pic_2_offset % 8 != 0 {return Err("PIC: Remap offsets unaligned.")}
    //Save Masks
    let mask_1 = DATA_PORT_1.read();
    let mask_2 = DATA_PORT_2.read();
    //Start Initialization Sequence
    COMMAND_PORT_1.write(0x11);      io_wait(); //ICW1: Start in cascade mode
    COMMAND_PORT_2.write(0x11);      io_wait(); //ICW1: Start in cascade mode
    DATA_PORT_1.write(pic_1_offset); io_wait(); //ICW2: Write PIC1 offset
    DATA_PORT_2.write(pic_2_offset); io_wait(); //ICW2: Write PIC2 offset
    DATA_PORT_1.write(0x04);         io_wait(); //ICW3: Write PIC1 PIC2 position (IRQ-2)
    DATA_PORT_2.write(0x02);         io_wait(); //ICW3: Write PIC2 cascade identity
    DATA_PORT_1.write(0x01);         io_wait(); //ICW4: Write PIC1 mode (8086 mode)
    DATA_PORT_2.write(0x01);         io_wait(); //ICW4: Write PIC2 mode (8086 mode)
    //Rewrite Masks
    DATA_PORT_1.write(mask_1);       io_wait();
    DATA_PORT_2.write(mask_2);       io_wait();
    //Return
    Ok(())
}

//Set IRQ Mask
pub unsafe fn set_mask(pic_1_mask: u8, pic_2_mask: u8) {
    DATA_PORT_1.write(pic_1_mask);
    DATA_PORT_2.write(pic_2_mask);
} 

//Enable or Disable an IRQ
pub unsafe fn disable_irq(irq: u8) -> Result<(), &'static str> {
    if irq < 8 {
        DATA_PORT_1.write(DATA_PORT_1.read() | (1 << irq));
    }
    else if irq < 16 {
        DATA_PORT_2.write(DATA_PORT_2.read() | (1 << (irq-8)));
    }
    else {return Err("PIC: IRQ out of bounds on set.")}
    Ok(())
}
pub unsafe fn enable_irq(irq: u8)  -> Result<(), &'static str> {
    if      irq <  8 {DATA_PORT_1.write(DATA_PORT_1.read() & !(1 << irq));}
    else if irq < 16 {DATA_PORT_2.write(DATA_PORT_2.read() & !(1 << (irq-8)));}
    else {return Err("PIC: IRQ out of bounds on clear.")}
    Ok(())
}

//Send End IRQ Signal
pub unsafe fn end_irq(irq: u8) -> Result<(), &'static str> {
    if irq < 16 {
        if irq > 7 {COMMAND_PORT_2.write(0x20);}
        COMMAND_PORT_1.write(0x20);
        Ok(())
    }
    else {Err("PIC: IRQ out of bounds on end of interrupt.")}
}
