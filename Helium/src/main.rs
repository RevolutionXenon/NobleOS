// HELIUM
// Helium is the Noble Kernel:
// Code execution
// Interrupt handling
// CPU time sharing
// (PLANNED) System call handling
// (PLANNED) Thread management
// (PLANNED) Program loading
// (PLANNED) Inter-process communication handling


// HEADER
//Flags
#![no_std]
#![no_main]
#![allow(unused_must_use)]
#![allow(clippy::if_same_then_else)]
#![allow(clippy::fn_to_numeric_cast)]
#![feature(abi_x86_interrupt)]
#![feature(asm_const)]
#![feature(naked_functions)]
#![feature(panic_info_message)]
#![feature(start)]
#![feature(used_with_arg)]

//Modules
mod alloc;
mod gdt;
mod kstruct;
mod limine_boot;
mod pmm;

//Imports
use crate::alloc::*;
use crate::pmm::*;
use crate::kstruct::*;
use gluon::GLUON_VERSION;
use gluon::noble::address_space::*;
use gluon::noble::data_type::*;
use gluon::noble::file_system::MemoryVolume;
//use gluon::noble::file_system::*;
use gluon::noble::input_events::*;
use gluon::noble::system_calls::*;
//use gluon::pc::fat::*;
use gluon::pc::ports::*;
use gluon::pc::pci::*;
use gluon::pc::pic;
use gluon::pc::pit;
use gluon::pc::ps2;
use gluon::sysv::executable::*;
use gluon::x86_64::instructions::*;
use gluon::x86_64::lapic;
use gluon::x86_64::paging::*;
use gluon::x86_64::port::*;
use gluon::x86_64::registers::*;
use gluon::x86_64::segmentation::*;
use photon::*;
use photon::formats::f1::*;
use core::arch::asm;
use core::cell::RefCell;
use core::convert::TryFrom;
use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr::write_volatile;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering;

//Constants
const HELIUM_VERSION: &str = "vDEV-2022"; //CURRENT VERSION OF KERNEL
static WHITESPACE:  CharacterTwoTone::<ColorBGRX> = CharacterTwoTone::<ColorBGRX> {codepoint: ' ', foreground: COLOR_BGRX_WHITE, background: COLOR_BGRX_BLACK};
static _BLACKSPACE: CharacterTwoTone::<ColorBGRX> = CharacterTwoTone::<ColorBGRX> {codepoint: ' ', foreground: COLOR_BGRX_BLACK, background: COLOR_BGRX_WHITE};
static _BLUESPACE:  CharacterTwoTone::<ColorBGRX> = CharacterTwoTone::<ColorBGRX> {codepoint: ' ', foreground: COLOR_BGRX_BLUE,  background: COLOR_BGRX_BLACK};
static REDSPACE:    CharacterTwoTone::<ColorBGRX> = CharacterTwoTone::<ColorBGRX> {codepoint: ' ', foreground: COLOR_BGRX_RED,   background: COLOR_BGRX_BLACK};


// MACROS
//Interrupt that halts (no error code)
macro_rules!interrupt_halt_noe {
    ($text:expr) => {{
        unsafe extern "x86-interrupt" fn handler(stack_frame: InterruptStackFrame) {
            asm!("MOV SS, {:x}", in(reg) u16::from(gdt::SUPERVISOR_DATA));
            asm!("MOV DS, {:x}", in(reg) u16::from(gdt::SUPERVISOR_DATA));
            let rip = stack_frame.code_pointer().0;
            let rsp = stack_frame.stack_pointer().0;
            let rflags = stack_frame.rflags_image();
            let cs = stack_frame.code_selector();
            let ss = stack_frame.stack_selector();
            if let Some(printer_pointer) = GLOBAL_WRITE_POINTER {
                let printer = &mut *printer_pointer;
                writeln!(printer, "\n{}\nRIP:    {:016X}\nRSP:    {:016X}\nCS:     Index: {:02X} RPL: {:01X}\nSS:     Index: {:02X} RPL: {:01X}\nRFLAGS: {:016X}\nCR2:    {:016X}\n",
                $text, rip, rsp,
                cs.descriptor_table_index, cs.requested_privilege_level as u8,
                ss.descriptor_table_index, ss.requested_privilege_level as u8,
                rflags, read_cr2());
            }
            loop {hlt();};
        }
        handler as unsafe extern "x86-interrupt" fn(InterruptStackFrame) as usize as u64
    }}
}

//Interrupt that halts (with error code)
macro_rules!interrupt_halt_err {
    ($text:expr) => {{
        unsafe extern "x86-interrupt" fn handler(stack_frame: InterruptStackFrame, error_code: u64) {
            asm!("MOV SS, {:x}", in(reg) u16::from(gdt::SUPERVISOR_DATA));
            asm!("MOV DS, {:x}", in(reg) u16::from(gdt::SUPERVISOR_DATA));
            let rip = stack_frame.code_pointer().0;
            let rsp = stack_frame.stack_pointer().0;
            let rflags = stack_frame.rflags_image();
            let cs = stack_frame.code_selector();
            let ss = stack_frame.stack_selector();
            if let Some(printer_pointer) = GLOBAL_WRITE_POINTER {
                let printer = &mut *printer_pointer;
                writeln!(printer, "\n{}\nRIP:    {:016X}\nRSP:    {:016X}\nCS:     Index: {:02X} RPL: {:01X}\nSS:     Index: {:02X} RPL: {:01X}\nRFLAGS: {:016X}\nERROR:  {:016X}\nCR2:    {:016X}\n",
                $text, rip, rsp,
                cs.descriptor_table_index, cs.requested_privilege_level as u8,
                ss.descriptor_table_index, ss.requested_privilege_level as u8,
                rflags, error_code, read_cr2());
            }
            loop {hlt();};
        }
        handler as unsafe extern "x86-interrupt" fn(InterruptStackFrame, u64) as usize as u64
    }}
}


// MAIN
//Main Entry Point After Hydrogen Boot
#[no_mangle]
pub extern "sysv64" fn _start() -> ! {
    // DISABLE INTERRUPTS
    cli();

    // LIMINE SETUP
    let framebuffer = limine_boot::LIMINE_FRAMEBUFFER.get_response().unwrap().framebuffers().next().unwrap();
    let framebuffer_address = framebuffer.addr();

    // GRAPHICS SETUP
    let pixel_renderer: PixelRendererHWD<ColorBGRX>;
    let character_renderer: CharacterTwoToneRenderer16x16<ColorBGRX>;
    let mut printer: PrintWindow::<PRINT_LINES, PRINT_HEIGHT, PRINT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>;
    let mut inputter: InputWindow::<INPUT_LENGTH, INPUT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>;
    {
        //Pixel Renderer
        pixel_renderer = PixelRendererHWD {pointer: framebuffer_address as *mut ColorBGRX, height: SCREEN_HEIGHT, width: SCREEN_WIDTH};
        for y in 0..SCREEN_HEIGHT {for x in 0..SCREEN_WIDTH {unsafe {pixel_renderer.render_pixel(COLOR_BGRX_BLACK, y, x);}}}
        //Character Renderer
        character_renderer = CharacterTwoToneRenderer16x16::<ColorBGRX> {renderer: &pixel_renderer, height: FRAME_HEIGHT, width: FRAME_WIDTH, y: 0, x: 0};
        character_renderer.render_character(CharacterTwoTone::<ColorBGRX>{ codepoint: '?', foreground: COLOR_BGRX_BLACK, background: COLOR_BGRX_RED }, 0, 0);
        //Frame
        let mut frame: FrameWindow::<FRAME_HEIGHT, FRAME_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>> = FrameWindow::<FRAME_HEIGHT, FRAME_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>::new(&character_renderer, WHITESPACE, 0, 0);
        frame.horizontal_line(PRINT_Y-1, 0, FRAME_WIDTH-1, REDSPACE);
        frame.horizontal_line(INPUT_Y-1, 0, FRAME_WIDTH-1, REDSPACE);
        frame.horizontal_line(INPUT_Y+1, 0, FRAME_WIDTH-1, REDSPACE);
        frame.vertical_line(0, PRINT_Y-1, INPUT_Y+1, REDSPACE);
        frame.vertical_line(FRAME_WIDTH-1, PRINT_Y-1, INPUT_Y+1, REDSPACE);
        frame.horizontal_string("NOBLE OS", 0, 0, REDSPACE);
        frame.horizontal_string("HELIUM KERNEL", 0, FRAME_WIDTH - 14 - HELIUM_VERSION.len(), REDSPACE);
        frame.horizontal_string(HELIUM_VERSION, 0, FRAME_WIDTH - HELIUM_VERSION.len(), REDSPACE);
        frame.render();
        //Globals
        inputter = InputWindow::<INPUT_LENGTH, INPUT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>::new(&character_renderer, WHITESPACE, INPUT_Y, INPUT_X);
        printer = PrintWindow::<PRINT_LINES, PRINT_HEIGHT, PRINT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>::new(&character_renderer, WHITESPACE, WHITESPACE, PRINT_Y, PRINT_X);
        unsafe {GLOBAL_WRITE_POINTER = Some(&mut printer as &mut dyn Write as *mut dyn Write)};
        unsafe {GLOBAL_INPUT_POINTER = Some(&mut inputter as *mut InputWindow<INPUT_LENGTH, INPUT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>)};
        unsafe {GLOBAL_PRINT_POINTER = Some(&mut printer as *mut PrintWindow<PRINT_LINES, PRINT_HEIGHT, PRINT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>)}
        //Print Welcome
        writeln!(printer, "Welcome to Noble OS");
        let boot_info = limine_boot::LIMINE_INFO.get_response().expect("No Bootloader Info");
        writeln!(printer, "{} Bootloader       v{}", boot_info.name(), boot_info.version());
        writeln!(printer, "Helium Kernel           {}", HELIUM_VERSION);
        writeln!(printer, "Photon Graphics Library {}", PHOTON_VERSION);
        writeln!(printer, "Gluon Memory Library    {}", GLUON_VERSION);
    }

    // NEW MEMORY SYSTEM TESTING
    writeln!(printer, "\n=== PAGE MAP ===\n");
    let hhdm_address: usize;
    let pml4: PageMap;
    let translator: OffsetIdentity;
    let mut allocator: MemoryStack;
    let mut memmap_xu: MapMemory;
    let mut memmap_eu: MapMemory;
    let mut memunmap: UnmapMemory;
    unsafe {
        //Limine HHDM
        hhdm_address = limine_boot::LIMINE_HHDM.get_response().unwrap().offset() as usize;
        writeln!(printer, "HHDM Address: 0x{:016X}", hhdm_address);
        //Physical to linear address translator
        translator = OffsetIdentity {
            offset: PHYSICAL_MEMORY_PTR,
            limit: PAGE_SIZE_512G,
        };
        //Limine Setup
        let limine_memmap_response = limine_boot::LIMINE_MEMMAP.get_response().unwrap();
        let limine_memmap_slice = limine_memmap_response.entries();
        let limine_areas_usable = limine_memmap_slice.iter()
            .filter(|x| x.entry_type == limine::memory_map::EntryType::USABLE);
        let mut limine_pages_usable = limine_memmap_slice.iter()
            .filter(|x| x.entry_type == limine::memory_map::EntryType::USABLE)
            .flat_map(|x| (0..x.length as usize / PAGE_SIZE_4KIB).map(move |p| x.base as usize + PAGE_SIZE_4KIB * p))
            .map(PhysicalAddress);
        writeln!(printer, "Successfully read Limine memory information.");
        //Limine Allocator
        let ref_cell: RefCell<&mut dyn Iterator<Item = PhysicalAddress>> = RefCell::new(&mut limine_pages_usable);
        let mut limine_pages_allocator = IteratorAllocator {
            iter_ref: ref_cell,
        };
        let mut limine_map_memory = MapMemory {
            allocator: &mut limine_pages_allocator,
            translator: &translator,
            write: true,
            user: true,
            execute_disable: true,
        };
        //Page map
        let pml4_physical = read_cr3_address();
        pml4 = PageMap::new(translator.translate(pml4_physical).unwrap(), PageMapLevel::L4).unwrap();
        writeln!(printer, "Successfully retrieved CR3: 0x{:016X}", pml4_physical.0);
        //Setup operations
        let mut markinuse = MarkInUse {translator: &translator};
        let mut deprivilege = DePrivilege {translator: &translator};
        //Mark in use
        virtual_memory_editor(pml4, &mut markinuse, LinearAddress(PHYSICAL_MEMORY_PTR), LinearAddress(PHYSICAL_MEMORY_PTR + page_size(PHYSICAL_MEMORY_LVL)));
        virtual_memory_editor(pml4, &mut markinuse, LinearAddress(KERNEL_CODE_PTR), LinearAddress(KERNEL_CODE_PTR - 1 + page_size(KERNEL_CODE_LVL)));
        //Deprivilege
        virtual_memory_editor(pml4, &mut deprivilege, LinearAddress(PHYSICAL_MEMORY_PTR), LinearAddress(PHYSICAL_MEMORY_PTR + page_size(PHYSICAL_MEMORY_LVL)));
        virtual_memory_editor(pml4, &mut deprivilege, LinearAddress(KERNEL_CODE_PTR), LinearAddress(KERNEL_CODE_PTR - 1 + page_size(KERNEL_CODE_LVL)));
        writeln!(printer, "Successfully sanitized page maps.");
        //Create memory stack
        let total_pages = {let mut sum: usize = 0; for i in limine_areas_usable {sum += i.length as usize / PAGE_SIZE_4KIB;} sum};
        writeln!(printer, "FREE MEMORY 1: {}", total_pages);
        virtual_memory_editor(pml4, &mut limine_map_memory, LinearAddress(ALLOCATOR_STACK_PTR), LinearAddress(ALLOCATOR_STACK_PTR + total_pages * 8));
        let stack_ptr = ALLOCATOR_STACK_PTR as *mut PhysicalAddress;
        let mut stack_count = 0;
        for address in limine_pages_usable {
            *stack_ptr.add(stack_count) = address;
            stack_count += 1;
        }
        writeln!(printer, "FREE MEMORY 2: {}", stack_count);
        //Create stack allocator
        allocator = MemoryStack {
            index: RefCell::new(stack_count),
            stack: stack_ptr.add(1),
            translator: &*(&translator as *const OffsetIdentity),
        };
        //Map and unmap operations
        memmap_xu = MapMemory {
            allocator: &*(&allocator as *const MemoryStack),
            translator: &*(&translator as *const OffsetIdentity),
            write: true,
            user: true,
            execute_disable: true,
        };
        memmap_eu = MapMemory {
            allocator: &*(&allocator as *const MemoryStack),
            translator: &*(&translator as *const OffsetIdentity),
            write: true,
            user: true,
            execute_disable: false,
        };
        memunmap = UnmapMemory {
            allocator: &*(&allocator as *const MemoryStack),
            translator: &*(&translator as *const OffsetIdentity),
        };
    }

    // HEAP ALLOCATION
    writeln!(printer, "\n=== HEAP ALLOCATION ===\n");
    unsafe {
        let heap_port_map_address = allocator.take_one().unwrap();
        let heap_port = MemPort{address: heap_port_map_address, level: KERNEL_HEAP_LVL, data_type: DataType::Binary};
        let map_port = MapPort {
            allocator: &allocator,
            translator: &translator,
            write: true,
            user: true,
            execute_disable: true,
        };
        map_port.map(pml4, heap_port, LinearAddress(KERNEL_HEAP_PTR)).unwrap();
        let mut heap = Heap1G::new();
        let heap_port_map = PageMap::new(translator.translate(heap_port_map_address).unwrap(), PageMapLevel::L2).unwrap();
        heap.init(
            heap_port_map,
            KERNEL_HEAP_PTR,
            &allocator as *const MemoryStack as *const dyn PhysicalAddressAllocator,
            &mut memmap_xu as *mut MapMemory as *mut dyn PageOperation,
            &mut memunmap as *mut UnmapMemory as *mut dyn PageOperation
        ).unwrap();
        //Testing
        for i in 0..26 {
            writeln!(printer, "Index: {:2}, Size: {:16X}", i, Heap1G::index_to_size(i));
        }
        writeln!(printer, "{:?}", Heap1G::split(25, AllocPtr{ state: AllocState::Free, next_address: PAGE_SIZE_1GIB }));
    }

    // PCI TESTING
    writeln!(printer, "\n=== PERIPHERAL COMPONENT INTERCONNECT BUS ===\n");
    let mut pci_uhci_option = None;
    unsafe {
        //Iterate over pci busses
        for pci_bus in 0..256 {
            //Iterate over pci devices
            for pci_device in 0..32 {
                //Iterate over device functions
                for pci_function in 0..8 {
                    //Look at the current endpoint and skip if it doesn't exist
                    let pci_endpoint = match PciEndpoint::new(pci_bus, pci_device, pci_function) {Ok(pci) => pci, Err(_) => break};
                    //print diagnostics
                    write!(printer, "PCI DEVICE:");
                    write!(printer, "  Bus: {:02X}, Device: {:02X}, Function: {:01X}", pci_bus, pci_device, pci_function);
                    writeln!(printer, "  |  Vendor ID: {:04X}, Device ID: {:04X}, Status: {:04X}", pci_endpoint.vendor_id(), pci_endpoint.device_id(), pci_endpoint.status());
                    //writeln!(printer, "  Revision ID:   {:02X}, Prog IF:       {:02X}, Subclass:      {:02X}, Class Code:    {:02X}", pci.revision_id(), pci.prog_if(), pci.subclass(), pci.class_code());
                    //writeln!(printer, "  Cache LSZ:     {:02X}, Latency Tmr:   {:02X}, Header Type:   {:02X}, BIST:          {:02X}", pci.chache_lz(), pci.latency(), pci.header_type(), pci.bist());
                    //If a UHCI endpoint is found, keep track of it
                    if let Ok(o) = PciUhci::new(pci_endpoint) {pci_uhci_option = Some(o)};
                }
            }
        }
    }

    // USB TESTING
    unsafe {if let Some(pci_uhci) = pci_uhci_option {
        writeln!(printer, "\n=== UHCI USB ===\n");
        //PCI diagnostic
        for i in 0..0x0F {
            writeln!(printer, "PCI Register {:02X}: {:08X}", i, pci_uhci.pci.register(i).unwrap());
        }
        //USB diagnostic
        writeln!(printer, "USB Command:             {:04X}", pci_uhci.command.read());
        writeln!(printer, "USB Status:              {:04X}", pci_uhci.status.read());
        writeln!(printer, "USB Interrupt Enable:    {:04X}", pci_uhci.interrupt.read());
        writeln!(printer, "Frame Number:            {:04X}", pci_uhci.frame_num.read());
        writeln!(printer, "Frame List Base Address: {:04X}", pci_uhci.frame_base.read());
        writeln!(printer, "Frame Modify Start:      {:04X}", pci_uhci.frame_mod.read());
        writeln!(printer, "Status/Control Port 1:   {:04X}", pci_uhci.sc_1.read());
        writeln!(printer, "Status/Control Port 2:   {:04X}", pci_uhci.sc_2.read());
    }}

    // GDT SETUP
    let gdt: GlobalDescriptorTable;
    writeln!(printer, "\n=== GLOBAL DESCRIPTOR TABLE ===\n");
    unsafe {
        //Allocate space for GDT
        allocator.take_one().unwrap();
        let a = allocator.take_one().unwrap();
        writeln!(printer, "GDT Physical Address: 0x{:016X}", a.0);
        gdt = GlobalDescriptorTable{address: translator.translate(a).unwrap(), limit: 512};
        writeln!(printer, "GDT Linear Address:   0x{:016X}", gdt.address.0);
        //Substitute symbol in TSS entry for actual TSS
        gdt::TASK_STATE_SEGMENT_ENTRY.base = &TASK_STATE_SEGMENT as *const TaskStateSegment as u64;
        //Write TSS entry into GDT
        gdt.write_system_entry(gdt::TASK_STATE_SEGMENT_ENTRY, gdt::TASK_STATE_SEGMENT_POSITION).unwrap();
        //Write GDT code and data entries
        gdt.write_entry(gdt::SUPERVISOR_CODE_ENTRY, gdt::SUPERVISOR_CODE_POSITION).unwrap();
        gdt.write_entry(gdt::SUPERVISOR_DATA_ENTRY, gdt::SUPERVISOR_DATA_POSITION).unwrap();
        gdt.write_entry(gdt::USER_CODE_ENTRY, gdt::USER_CODE_POSITION).unwrap();
        gdt.write_entry(gdt::USER_DATA_ENTRY, gdt::USER_DATA_POSITION).unwrap();
        //Load GDTR
        gdt.write_gdtr(gdt::SUPERVISOR_CODE, gdt::SUPERVISOR_DATA, gdt::SUPERVISOR_DATA);
        //Load Task Register
        load_task_register(gdt::TASK_STATE_SEGMENT);
    }

    // IDT SETUP
    writeln!(printer, "\n=== INTERRUPT DESCRIPTOR TABLE ===\n");
    let idt: InterruptDescriptorTable;
    unsafe {
        //Allocate space for IDT
        idt = InterruptDescriptorTable {address: translator.translate(allocator.take_one().unwrap()).unwrap(), limit: 255};
        //INT 00h - INT 19h
        //CPU exceptions
        let mut int_exception = InterruptDescriptor {
            offset: 0,
            segment_selector: gdt::SUPERVISOR_CODE,
            segment_present: true,
            privilege_level: PrivilegeLevel::Supervisor,
            interrupt_stack_table: 1,
            descriptor_type: DescriptorType::InterruptGate,
        };
        int_exception.offset = interrupt_halt_noe!("INTERRUPT VECTOR 0x00 (#DE): Divide Error");                 idt.write_entry(&int_exception, 0x00);
        int_exception.offset = interrupt_halt_noe!("INTERRUPT VECTOR 0x01 (#DB): Debug");                        idt.write_entry(&int_exception, 0x01);
        int_exception.offset = interrupt_halt_noe!("INTERRUPT VECTOR 0x02: Non-Maskable Interrupt");             idt.write_entry(&int_exception, 0x02);
        int_exception.offset = interrupt_halt_noe!("INTERRUPT VECTOR 0x03 (#BP): Breakpoint");                   idt.write_entry(&int_exception, 0x03);
        int_exception.offset = interrupt_halt_noe!("INTERRUPT VECTOR 0x04 (#OF): Overflow");                     idt.write_entry(&int_exception, 0x04);
        int_exception.offset = interrupt_halt_noe!("INTERRUPT VECTOR 0x05 (#BR): Bound Range Exceeded");         idt.write_entry(&int_exception, 0x05);
        int_exception.offset = interrupt_halt_noe!("INTERRUPT VECTOR 0x06 (#UD): Invalid Opcode");               idt.write_entry(&int_exception, 0x06);
        int_exception.offset = interrupt_halt_noe!("INTERRUPT VECTOR 0x07 (#NM): Device Not Available");         idt.write_entry(&int_exception, 0x07);
        int_exception.offset = interrupt_halt_noe!("INTERRUPT VECTOR 0x08 (#DF): Double Fault");                 idt.write_entry(&int_exception, 0x08);
        int_exception.offset = interrupt_halt_err!("INTERRUPT VECTOR 0x0A (#TS): Invalid Task State Segment");   idt.write_entry(&int_exception, 0x0A);
        int_exception.offset = interrupt_halt_err!("INTERRUPT VECTOR 0x0B (#NP): Segment Not Present");          idt.write_entry(&int_exception, 0x0B);
        int_exception.offset = interrupt_halt_err!("INTERRUPT VECTOR 0x0C (#SS): Stack Fault");                  idt.write_entry(&int_exception, 0x0C);
        int_exception.offset = interrupt_halt_err!("INTERRUPT VECTOR 0x0D (#GP): General Protection Fault");     idt.write_entry(&int_exception, 0x0D);
        int_exception.offset = interrupt_halt_err!("INTERRUPT VECTOR 0x0E (#PF): Page Fault");                   idt.write_entry(&int_exception, 0x0E);
        int_exception.offset = interrupt_halt_noe!("INTERRUPT VECTOR 0x10 (#MF): x87 FPU Floating Point Error"); idt.write_entry(&int_exception, 0x10);
        int_exception.offset = interrupt_halt_err!("INTERRUPT VECTOR 0x11 (#AC): Alignment Check");              idt.write_entry(&int_exception, 0x11);
        int_exception.offset = interrupt_halt_noe!("INTERRUPT VECTOR 0x12 (#MC): Machine Check");                idt.write_entry(&int_exception, 0x12);
        int_exception.offset = interrupt_halt_noe!("INTERRUPT VECTOR 0x13 (#XM): SIMD Floating Point Error");    idt.write_entry(&int_exception, 0x13);
        int_exception.offset = interrupt_halt_noe!("INTERRUPT VECTOR 0x14 (#VE): Virtualization Fault");         idt.write_entry(&int_exception, 0x14);
        int_exception.offset = interrupt_halt_err!("INTERRUPT VECTOR 0x15 (#CP): Control Protection Fault");     idt.write_entry(&int_exception, 0x15);
        //INT 20h - INT FFh
        //Immediate returns to all non-exception interrupts
        let int_user: InterruptDescriptor = InterruptDescriptor {
            offset: interrupt_immediate_return as unsafe extern "x86-interrupt" fn() as u64,
            segment_selector: gdt::SUPERVISOR_CODE,
            segment_present: true,
            privilege_level: PrivilegeLevel::Supervisor,
            interrupt_stack_table: 0,
            descriptor_type: DescriptorType::InterruptGate,
        };
        for position in 0x20..0xFF {idt.write_entry(&int_user, position);}
        //INT 20h
        //IRQ 0: Programmable Interval Timer
        let int_pit: InterruptDescriptor = InterruptDescriptor {
            offset: interrupt_irq_00 as unsafe extern "x86-interrupt" fn() as u64,
            segment_selector: gdt::SUPERVISOR_CODE,
            segment_present: true,
            privilege_level: PrivilegeLevel::Supervisor,
            interrupt_stack_table: 0,
            descriptor_type: DescriptorType::InterruptGate,
        };
        idt.write_entry(&int_pit, 0x20);
        //INT 21h
        //IRQ 1: Keyboard Handler
        let int_keyboard: InterruptDescriptor = InterruptDescriptor {
            offset: interrupt_irq_01 as unsafe extern "x86-interrupt" fn() as usize as u64,
            segment_selector: gdt::SUPERVISOR_CODE,
            segment_present: true,
            privilege_level: PrivilegeLevel::Supervisor,
            interrupt_stack_table: 2,
            descriptor_type: DescriptorType::InterruptGate,
        };
        idt.write_entry(&int_keyboard, 0x21);
        //INT 30h
        //LAPIC Timer
        let int_timer: InterruptDescriptor = InterruptDescriptor {
            offset: interrupt_timer as unsafe extern "x86-interrupt" fn() as usize as u64,
            segment_selector: gdt::SUPERVISOR_CODE,
            segment_present: true,
            privilege_level: PrivilegeLevel::Supervisor,
            interrupt_stack_table: 0,
            descriptor_type: DescriptorType::InterruptGate,
        };
        idt.write_entry(&int_timer, 0x30);
        //INT 31h
        //User accessible yield interrupt
        let int_yield: InterruptDescriptor = InterruptDescriptor {
            offset: interrupt_yield as unsafe extern "x86-interrupt" fn() as usize as u64,
            segment_selector: gdt::SUPERVISOR_CODE,
            segment_present: true,
            privilege_level: PrivilegeLevel::User,
            interrupt_stack_table: 0,
            descriptor_type: DescriptorType::InterruptGate,
        };
        idt.write_entry(&int_yield, 0x31);
        //INT 32h
        //System Call Interrupt
        let int_syscall: InterruptDescriptor = InterruptDescriptor {
            offset: interrupt_syscall as unsafe extern "x86-interrupt" fn() as usize as u64,
            segment_selector: gdt::SUPERVISOR_CODE,
            segment_present: true,
            privilege_level: PrivilegeLevel::User,
            interrupt_stack_table: 0,
            descriptor_type: DescriptorType::InterruptGate,
        };
        idt.write_entry(&int_syscall, 0x32);
        //INT FFh
        //LAPIC Spurious Interrupt
        let int_spurious: InterruptDescriptor = InterruptDescriptor {
            offset: interrupt_spurious as extern "x86-interrupt" fn() as usize as u64,
            segment_selector: gdt::SUPERVISOR_CODE,
            segment_present: true,
            privilege_level: PrivilegeLevel::Supervisor,
            interrupt_stack_table: 2,
            descriptor_type: DescriptorType::InterruptGate,
        };
        idt.write_entry(&int_spurious, 0xFF);
        //Write IDTR
        idt.write_idtr();
        //Diagnostic
        writeln!(printer, "IDT Linear Address:         0x{:016X}", idt.address.0);
    }

    // IST SETUP
    writeln!(printer, "\n=== INTERRUPT STACK TABLE ===\n");
    //let pm_kstack: PageMap2;
    unsafe {
        //Create kernel stack
        let start: LinearAddress = LinearAddress(KERNEL_STACKS_PTR + PAGE_SIZE_4KIB * 1);
        let end: LinearAddress   = LinearAddress(KERNEL_STACKS_PTR + PAGE_SIZE_4KIB * 4);
        virtual_memory_editor(pml4, &mut memmap_xu, start, end);
        let kernel_stack = (KERNEL_STACKS_PTR + PAGE_SIZE_4KIB * 4) as u64;
        //Update TSS
        TASK_STATE_SEGMENT.rsp0 = kernel_stack;
        TASK_STATE_SEGMENT.ist1 = kernel_stack;
        TASK_STATE_SEGMENT.ist2 = kernel_stack;
    }

    // PIC SETUP
    writeln!(printer, "\n=== PROGRAMMABLE INTERRUPT CONTROLLER ===\n");
    unsafe {
        //Remap PIC
        pic::remap(0x20, 0x28).unwrap();
        //Enable IRQ 1
        pic::enable_irq(0x1).unwrap();
        //Test ability to read PIC inputs
        writeln!(printer, "PIC Data: {:08b} {:08b}", PIC1_DATA.read(), PIC2_DATA.read());
    }

    // PIT BUS SPEED MEASUREMENT
    writeln!(printer, "\n=== PROGRAMMABLE INTERVAL TIMER ===\n");
    let cpu_hz: u64;
    unsafe {
        //Enable IRQ 0
        pic::enable_irq(0x0);
        sti();
        //Set PIT channel 0 to one shot mode
        pit::send_command(pit::Channel::C1, pit::AccessMode::Full, pit::OperatingMode::RateGenerator, pit::BinaryMode::Binary);
        //Set PIT to Interrupt in 1/41 of a second
        pit::set_reload_full(pit::Channel::C1, (pit::PIT_FREQUENCY / 41) as u16);
        //Get interval 1
        hlt();
        let interval_1: u64 = rdtsc();
        //Get interval 2
        hlt();
        let interval_2: u64 = rdtsc();
        //Get interval 3
        hlt();
        let interval_3: u64 = rdtsc();
        //Get interval 4
        hlt();
        let interval_4: u64 = rdtsc();
        //Get interval 5
        hlt();
        let interval_5: u64 = rdtsc();
        //Disable IRQ 0
        pic::disable_irq(0x0);
        cli();
        //Calculate Hz
        let hz_2 = (interval_2 - interval_1) * 41;
        let hz_3 = (interval_3 - interval_2) * 41;
        let hz_4 = (interval_4 - interval_3) * 41;
        let hz_5 = (interval_5 - interval_4) * 41;
        cpu_hz = (hz_2 + hz_3 + hz_4 + hz_5) / 4;
        //Diagnostic
        writeln!(printer, "Interval 1:  {:16X}", interval_1);
        writeln!(printer, "Interval 2:  {:16X} ({:5}MHz)", interval_2, hz_2 / 1_000_000);
        writeln!(printer, "Interval 3:  {:16X} ({:5}MHz)", interval_3, hz_3 / 1_000_000);
        writeln!(printer, "Interval 4:  {:16X} ({:5}MHz)", interval_4, hz_4 / 1_000_000);
        writeln!(printer, "Interval 5:  {:16X} ({:5}MHz)", interval_5, hz_5 / 1_000_000);
        writeln!(printer, "Average MHz: {:16}", cpu_hz / 1_000_000);
    }

    // PS/2 BUS
    writeln!(printer, "\n=== PERSONAL SYSTEM/2 BUS ===\n");
    unsafe {
        //Disable PS/2 ports
        ps2::disable_port1();
        ps2::disable_port2();
        //Set PS/2 flags
        let a1 = ps2::read_memory(0x0000).unwrap();
        ps2::write_memory(0x0000, a1 & 0b1011_1100).unwrap();
        //Test PS/2 controller
        let ps2_port1_present: bool;
        let ps2_port2_present: bool;
        if ps2::test_controller() {
            writeln!(printer, "PS/2 Controller test succeeded.");
            //Test PS/2 controller ports
            ps2_port1_present = ps2::test_port_1();
            ps2_port2_present = ps2::test_port_2();
            if ps2_port1_present || ps2_port2_present {
                writeln!(printer, "PS/2 Port tests succeeded:");
                //Enable Port 1
                if ps2_port1_present {
                    writeln!(printer, "  PS/2 Port 1 Present.");
                    ps2::flush_output();
                    ps2::keyboard_disable_scan().unwrap();
                    ps2::keyboard_set_scancode_set(1).unwrap();
                    let scancode_set = ps2::keyboard_get_scancode_set().unwrap();
                    writeln!(printer, "  PS/2 Keyboard Scancode Set: {}", scancode_set);
                    ps2::keyboard_enable_scan().unwrap();
                    ps2::enable_port1();
                    ps2::enable_int_port1();
                }
                //Enable Port 2
                if ps2_port2_present {
                    writeln!(printer, "  PS/2 Port 2 Present.");
                    //ps2::enable_port2();
                }
                else {
                    writeln!(printer, "  PS/2 Port 2 Not Present.");
                }
            }
            else {writeln!(printer, "PS/2 Port tests failed.");}
        }
        else {writeln!(printer, "PS/2 Controller test failed.");}
    }

    // APIC SETUP
    writeln!(printer, "\n=== ADVANCED PROGRAMMABLE INTERRUPT CONTROLLER ===\n");
    unsafe {
        //Diagnostic
        writeln!(printer, "APIC Present: {}", lapic::apic_check());
        writeln!(printer, "APIC Base: 0x{:16X}", lapic::get_base());
        //Shift Base
        lapic::LAPIC_ADDRESS = (lapic::get_base() as usize + hhdm_address as usize) as *mut u8;
        //Spurious interrupt
        lapic::spurious(0xFF);
        //Diagnostic
        writeln!(printer, "APIC ID:   0x{:1X}", lapic::read_register(0x20).unwrap() >> 24);
        writeln!(printer, "APIC 0xF0: 0x{:08X}", lapic::read_register(0xF0).unwrap());
        //Set Timer Mode
        lapic::timer(0x30, false, lapic::TimerMode::Periodic);
        lapic::divide_config(lapic::Divide::Divide_1);
        //Enable LAPIC
        lapic::enable();
    }

    // RAMDISK TESTING
    /*writeln!(printer, "\n=== RAMDISK TEST ===\n");
    unsafe {
        /*let bytes: [u8;0x200] = read_volatile(oct4_to_pointer(RAMDISK_OCT).unwrap() as *const [u8;0x200]);
        let boot_sector_r = FATBootSector::try_from(bytes);
        writeln!(printer, "{:?}", boot_sector_r);
        writeln!(printer, "{:016X}", oct4_to_usize(RAMDISK_OCT).unwrap());
        match boot_sector_r {
            Ok(boot_sector) => {
                writeln!(printer, "Total Sectors:     {:16X}", boot_sector.total_sectors());
                writeln!(printer, "Cluster Size:      {:16X}", boot_sector.cluster_size());
                writeln!(printer, "First FAT Sector:  {:16X}", boot_sector.first_fat_sector());
                writeln!(printer, "First Root Sector: {:16X}", boot_sector.first_root_sector());
                writeln!(printer, "First Data Sector: {:16X}", boot_sector.first_data_sector());
            },
            Err(_) => {writeln!(printer, "Boot sector invalid.");},
        }*/
        let volume = MemoryVolume{offset: oct4_to_usize(RAMDISK_OCT).unwrap(), size: 8 * MIB};
        let file_system = FATFileSystem::from_existing_volume(&volume).unwrap();
        let root_directory_id = file_system.root().unwrap();
        writeln!(printer, "Root Directory ID:          {:?}", root_directory_id);
        let root_directory_open = file_system.open(root_directory_id).unwrap();
        writeln!(printer, "Open Root Directory ID:     {:?}", root_directory_open);
        let root_directory_first = file_system.dir_first(root_directory_open).unwrap().unwrap();
        writeln!(printer, "Root Directory First Index: {}", root_directory_first);
        let root_directory_find = file_system.dir_name(root_directory_open, "test.raw");
        writeln!(printer, "Root Directory Name Index:  {:?}", root_directory_find);
        let root_directory_none = file_system.dir_name(root_directory_open, "test2.raw");
        writeln!(printer, "Root Directory None Test:   {:?}", root_directory_none);
        let test_file_id = file_system.get_id(root_directory_open, root_directory_first).unwrap();
        writeln!(printer, "Test File ID:               {:?}", test_file_id);
        let test_file_open = file_system.open(test_file_id).unwrap();
        writeln!(printer, "Open Test File ID:          {:?}", test_file_open).unwrap();
        let mut buffer = [0u8;12];
        writeln!(printer, "File Name:                  {}", file_system.get_name(root_directory_open, root_directory_first, &mut buffer).unwrap());
        let test_file = FileShortcut{fs: &file_system, id: test_file_open};
        for i in 1..12 {
            writeln!(printer, "{:?}", {
                let mut buffer = [0u8;16];
                test_file.read((i*KIB) as u64 - 8, &mut buffer).unwrap();
                buffer
            });
        }
    }*/

    // MODULE LOADING
    writeln!(printer, "\n=== LIMINE MODULES ===\n");
    let mut module_instruction_ptr: u64 = 0;
    unsafe {
        //Executable loading
        let mut current_module_address = LinearAddress(MODULE_CODE_PTR);
        //Load Modules
        let modules_response = limine_boot::LIMINE_MODULES.get_response().unwrap();
        let modules = modules_response.modules();
        writeln!(printer, "MODULE COUNT: {}", modules.len());
        //Iterate over modules
        for module in modules {
            //Load module path
            let module_file_path: &str = core::str::from_utf8_unchecked(module.path());
            writeln!(printer, "MODULE: {}", module_file_path);
            //Load file details
            let module_file_location: usize = module.addr() as usize;
            let module_file_size: usize = module.size() as usize;
            writeln!(printer, "MODULE FILE LOCATION: 0x{:016X}", module_file_location);
            writeln!(printer, "MODULE FILE SIZE:     0x{:016X}", module_file_size);
            //Executable module
            if module_file_path.ends_with("x86-64.elf") {
                writeln!(printer, "MODULE EXECUTABLE:    TRUE");
                //Setup file
                let module_file: MemoryVolume = MemoryVolume {offset: module_file_location, size: module_file_size};
                if let Ok(mut module) = ELFFile::new(&module_file) {
                    //Check ELF header validity
                    let valid_binary_interface: bool = module.header.binary_interface == ApplicationBinaryInterface::None;
                    let valid_binary_interface_version: bool = module.header.binary_interface_version == 0x00;
                    let valid_architecture: bool = module.header.architecture == InstructionSetArchitecture::EmX86_64;
                    let valid_object_type: bool = module.header.object_type == ObjectType::Shared;
                    let valid: bool = valid_binary_interface && valid_binary_interface_version && valid_architecture && valid_object_type;
                    writeln!(printer, "MODULE VALID:         {}", valid);
                    if valid {
                        //Allocate memory for module
                        let module_size: usize = module.program_memory_size() as usize;
                        writeln!(printer, "MODULE SIZE:          0x{:016X}", module_size);
                        virtual_memory_editor(pml4, &mut memmap_eu, current_module_address, current_module_address.add(module_size)).unwrap();
                        //Load and relocate module
                        let module_ptr: *mut u8 = current_module_address.0 as *mut u8;
                        writeln!(printer, "LOADING MODULE AT:    0x{:016X}", module_ptr as usize);
                        module.load(module_ptr).unwrap();
                        module.relocate(module_ptr, module_ptr).unwrap();
                        //Save module instruction pointer
                        module_instruction_ptr = (current_module_address.0 as u64) + module.header.entry_point;
                        writeln!(printer, "MODULE ENTRY POINT:   0x{:016X}", module_instruction_ptr);
                        //Adjust next module location
                        current_module_address = current_module_address.add(page_size(align_lvl(module_size)));
                    }
                }
                else {writeln!(printer, "MODULE CORRUPTED");}
            }
            else {writeln!(printer, "MODULE EXECUTABLE:    FALSE");}
            writeln!(printer);
        }
    }

    // CREATE THREADS
    writeln!(printer, "\n=== THREAD STACK TEST ===\n");
    unsafe {
        //Stack pointers
        let s0p = oct_to_usize_4(0, 0, 0, 0, 0).unwrap();
        let s1p = oct_to_usize_4(0, 0, 1, 0, 0).unwrap();
        let s2p = oct_to_usize_4(0, 0, 2, 0, 0).unwrap();
        let s3p = oct_to_usize_4(0, 0, 3, 0, 0).unwrap();
        let s4p = oct_to_usize_4(0, 0, 4, 0, 0).unwrap();
        //Allocate stack space
        virtual_memory_editor(pml4, &mut memmap_xu, LinearAddress(s0p + PAGE_SIZE_4KIB), LinearAddress(s1p));
        virtual_memory_editor(pml4, &mut memmap_xu, LinearAddress(s1p + PAGE_SIZE_4KIB), LinearAddress(s2p));
        virtual_memory_editor(pml4, &mut memmap_xu, LinearAddress(s2p + PAGE_SIZE_4KIB), LinearAddress(s3p));
        virtual_memory_editor(pml4, &mut memmap_xu, LinearAddress(s3p + PAGE_SIZE_4KIB), LinearAddress(s4p));
        //Instruction pointers
        let i1p = read_loop as fn() as usize as u64;
        let i2p = ps2_keyboard as unsafe fn() as usize as u64;
        let i3p = module_instruction_ptr;
        //Create tasks
        create_thread(1, pml4, &translator, &mut memmap_xu, i1p, gdt::SUPERVISOR_CODE, 0x00000202, s1p, gdt::SUPERVISOR_DATA);
        create_thread(2, pml4, &translator, &mut memmap_xu, i2p, gdt::USER_CODE, 0x00000202, s2p, gdt::USER_DATA);
        create_thread(3, pml4, &translator, &mut memmap_xu, i3p, gdt::USER_CODE, 0x00000202, s3p, gdt::USER_DATA);
        //Diagnostic
        writeln!(printer, "Thread 1 (PIPE READ AND PRINT):");
        writeln!(printer, "  Stack Pointer Before Init: 0x{:16X}", s1p);
        writeln!(printer, "  Stack Pointer After Init:  0x{:16X}", TASK_STACKS[1]);
        writeln!(printer, "  Instruction Pointer:       0x{:16X}", i1p);
        writeln!(printer, "Thread 2 (PS2 KEYBOARD):");
        writeln!(printer, "  Stack Pointer Before Init: 0x{:16X}", s2p);
        writeln!(printer, "  Stack Pointer After Init:  0x{:16X}", TASK_STACKS[2]);
        writeln!(printer, "  Instruction Pointer:       0x{:16X}", i2p);
        writeln!(printer, "Thread 3 (TEST MODULE):");
        writeln!(printer, "  Stack Pointer Before Init: 0x{:16X}", s3p);
        writeln!(printer, "  Stack Pointer After Init:  0x{:16X}", TASK_STACKS[3]);
        writeln!(printer, "  Instruction Pointer:       0x{:16X}", i3p);
    }

    // FINISH LOADING
    writeln!(printer, "\n=== STARTUP COMPLETE ===\n");
    unsafe {
        //Start LAPIC timer
        lapic::initial_count((cpu_hz / 1000) as u32);
        //Enable Interrupts
        sti();
        //Halt init thread
        loop{hlt();}
    }
}


// TASKING
//Global variables
static GLOBAL_TIME: AtomicU64 = AtomicU64::new(0);
static mut GLOBAL_WRITE_POINTER: Option<*mut dyn Write> = None;
static mut GLOBAL_PRINT_POINTER: Option<*mut PrintWindow::<PRINT_LINES, PRINT_HEIGHT, PRINT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>> = None;
static mut GLOBAL_INPUT_POINTER: Option<*mut InputWindow::<INPUT_LENGTH, INPUT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>> = None;
static mut TASK_INDEX: usize = 0;
static mut TASK_STACKS: [u64; 4] = [0;4];

//Thread Creation Function
unsafe fn create_thread(thread_index: usize, map: PageMap, translator: &dyn AddressTranslator, mmap: &mut MapMemory, instruction_pointer: u64, code_selector: SegmentSelector, eflags_image: u32, stack_pointer: usize, stack_selector: SegmentSelector) {
    //Create stack
    assert!(map.map_level == PageMapLevel::L4);
    let start = LinearAddress(KERNEL_STACKS_PTR + PAGE_SIZE_4KIB * (thread_index * 4 + 1));
    let end   = LinearAddress(KERNEL_STACKS_PTR + PAGE_SIZE_4KIB * (thread_index * 4 + 4));
    virtual_memory_editor(map, mmap, start, end);
    let rsp = (KERNEL_STACKS_PTR as *mut u64).byte_add(PAGE_SIZE_4KIB * (thread_index * 4 + 4));
    //Write stack frame
    write_volatile(rsp.sub(1), u16::from(stack_selector) as u64);
    write_volatile(rsp.sub(2), stack_pointer as u64);
    write_volatile(rsp.sub(3), eflags_image as u64);
    write_volatile(rsp.sub(4), u16::from(code_selector) as u64);
    write_volatile(rsp.sub(5), instruction_pointer);
    //Zero register save states
    for i in 5..53 {
        write_volatile((stack_pointer as *mut u64).sub(i), 0);
    }
    //Save stack pointer
    TASK_STACKS[thread_index] = rsp.sub(20) as u64;
}

//Scheduler
unsafe extern "sysv64" fn scheduler() -> u64 {
    //Read current time
    let time = GLOBAL_TIME.load(Ordering::Relaxed);
    //Process thread to switch to
    TASK_INDEX = 
    if time % 1000 == 0                                                                                   {3} else
    if INPUT_PIPE.state  == RingBufferState::WriteWait                                                    {2} else
    if STRING_PIPE.state == RingBufferState::WriteWait || STRING_PIPE.state == RingBufferState::ReadBlock {1} else
                                                                                                          {0};
    //Change task state segment to new task
    TASK_STATE_SEGMENT.rsp0 = (KERNEL_STACKS_PTR as u64) + ((TASK_INDEX + 1) * 16 * KIB) as u64;
    //Update current time
    GLOBAL_TIME.fetch_add(1, Ordering::Relaxed);
    //Finish
    TASK_STACKS[TASK_INDEX]
}


// THREADS
//Thread 1: Pipe Read and Print
static mut STRING_PIPE: RingBuffer<u8, 4096> = RingBuffer{data: [0xFF; 4096], read_head: 0, write_head: 0, state: RingBufferState::ReadWait};
fn read_loop() {unsafe {
    let printer = &mut *GLOBAL_WRITE_POINTER.unwrap();
    loop {
        write_volatile(&mut STRING_PIPE.state as *mut RingBufferState, RingBufferState::ReadBlock);
        writeln!(printer, "{}", core::str::from_utf8(STRING_PIPE.read(&mut [0xFF; 4096])).unwrap());
        writeln!(printer, "SYSTEM CALL 00: 0x{:016X}", system_call_00());
        system_call_01();
        let a = system_call_02();
        writeln!(printer, "SYSTEM CALL 02: 0x{:016X}", a);
        writeln!(printer, "GLOBAL TIME:    0x{:016X}", GLOBAL_TIME.load(Ordering::Relaxed));
        write_volatile(&mut STRING_PIPE.state as *mut RingBufferState, RingBufferState::ReadWait);
        asm!("INT 31h");
    }
}}

//Thread 2: Pipe Write
fn byte_loop() {unsafe {
    loop {
        write_volatile(&mut STRING_PIPE.state as *mut RingBufferState, RingBufferState::WriteBlock);
        STRING_PIPE.write("HELLO FROM USERSPACE! ".as_bytes());
        write_volatile(&mut STRING_PIPE.state as *mut RingBufferState, RingBufferState::WriteWait);
        asm!("INT 31h");
    }
}}

//Thread 3: PS/2 Keyboard
static mut LEFT_SHIFT:  bool = false;
static mut RIGHT_SHIFT: bool = false;
static mut CAPS_LOCK:   bool = false;
static mut NUM_LOCK:    bool = false;
static mut INPUT_PIPE: RingBuffer<InputEvent, 512> = RingBuffer{data: [InputEvent{device_id: 0xFF, event_type: InputEventType::Blank, event_id: 0, event_data: 0}; 512], read_head: 0, write_head: 0, state: RingBufferState::Free};
unsafe fn ps2_keyboard() {
    let printer = &mut *GLOBAL_WRITE_POINTER.unwrap();
    let inputter = &mut *GLOBAL_INPUT_POINTER.unwrap();
    let window = &mut *GLOBAL_PRINT_POINTER.unwrap();
    loop {
        write_volatile(&mut INPUT_PIPE.state as *mut RingBufferState, RingBufferState::ReadBlock);
        let mut buffer = [InputEvent{device_id: 0xFF, event_type: InputEventType::Blank, event_id: 0, event_data: 0}; 512];
        let input_events = INPUT_PIPE.read(&mut buffer);
        for input_event in input_events {
            if input_event.event_type == InputEventType::DigitalKey {
                match KeyID::try_from(input_event.event_id) {Ok(key_id) => {
                    match PressType::try_from(input_event.event_data) {Ok(press_type) => {
                        match us_qwerty(key_id, CAPS_LOCK ^ (LEFT_SHIFT || RIGHT_SHIFT), NUM_LOCK) {
                            KeyStr::Key(key_id) => { match (key_id, press_type) {
                                (KeyID::NumLock,       PressType::Press)   => {NUM_LOCK    = !NUM_LOCK;}
                                (KeyID::KeyCapsLock,   PressType::Press)   => {CAPS_LOCK   = !CAPS_LOCK;}
                                (KeyID::KeyLeftShift,  PressType::Press)   => {LEFT_SHIFT  = true;}
                                (KeyID::KeyLeftShift,  PressType::Unpress) => {LEFT_SHIFT  = false;}
                                (KeyID::KeyRightShift, PressType::Press)   => {RIGHT_SHIFT = true;}
                                (KeyID::KeyRightShift, PressType::Unpress) => {RIGHT_SHIFT = false;}
                                (KeyID::KeyHome,       PressType::Press)   => {window.end_up();}
                                (KeyID::KeyEnd,        PressType::Press)   => {window.end_down();}
                                (KeyID::KeyPageUp,     PressType::Press)   => {window.page_up();}
                                (KeyID::KeyPageDown,   PressType::Press)   => {window.page_down();}
                                (KeyID::KeyUpArrow,    PressType::Press)   => {window.line_up();}
                                (KeyID::KeyDownArrow,  PressType::Press)   => {window.line_down();}
                                (KeyID::KeyEscape,     PressType::Press)   => {asm!("INT3")}
                                _ => {}
                            }},
                            KeyStr::Str(s) => {match press_type {PressType::Press => {
                                for codepoint in s.chars() {
                                    if codepoint == '\n' {
                                        while STRING_PIPE.state == RingBufferState::ReadBlock {}
                                        write_volatile(&mut STRING_PIPE.state as *mut RingBufferState, RingBufferState::WriteBlock);
                                        let mut buffer = [0u8; INPUT_LENGTH*4];
                                        let string = match inputter.to_str(&mut buffer) {
                                            Ok(string) => string,
                                            Err(error) => error,
                                        };
                                        STRING_PIPE.write(string.as_bytes());
                                        write_volatile(&mut STRING_PIPE.state as *mut RingBufferState, RingBufferState::WriteWait);
                                        inputter.flush(WHITESPACE);
                                    }
                                    else {
                                        inputter.push_render(CharacterTwoTone{codepoint, foreground: COLOR_BGRX_WHITE, background: COLOR_BGRX_BLACK}, WHITESPACE);
                                    }
                                }
                            } PressType::Unpress => {}}}
                        }
                    } Err(_) => {writeln!(printer, "Input Event Error: Unknown Press Type");}}
                } Err(_) => {writeln!(printer, "Input Event Error: Unknown Key ID");}}
            }
        }
        write_volatile(&mut INPUT_PIPE.state as *mut RingBufferState, RingBufferState::Free);
        asm!("INT 31h");
    }
}


// INTERRUPT FUNCTIONS
//INT 20h-FFh: Immediate Return Interrupt
unsafe extern "x86-interrupt" fn interrupt_immediate_return() {}

//INT 20h: PIT IRQ
unsafe extern "x86-interrupt" fn interrupt_irq_00() {pic::end_irq(0x0);}

//INT 21h: PS/2 Keyboard IRQ
static mut PS2_SCANCODES: [u8;9] = [0u8;9];
static mut PS2_INDEX:   usize = 0x00;
unsafe extern "x86-interrupt" fn interrupt_irq_01() {
    while ps2::poll_output_buffer_status() {
        let scancode = ps2::read_output();
        PS2_SCANCODES[PS2_INDEX] = scancode;
        PS2_INDEX += 1;
        match ps2::scancodes_1(&PS2_SCANCODES[0..PS2_INDEX], 0x00) {
            Ok(ps2_scan) => match ps2_scan {
                ps2::Ps2Scan::Finish(input_event) => {
                    INPUT_PIPE.write(&[input_event]);
                    write_volatile(&mut INPUT_PIPE.state as *mut RingBufferState, RingBufferState::WriteWait);
                    PS2_INDEX = 0;
                }
                ps2::Ps2Scan::Continue => {}
            }
            Err(error) => {
                let printer = &mut *GLOBAL_WRITE_POINTER.unwrap();
                writeln!(printer, "PS/2 KEYBOARD ERROR: {:?} | {}", &PS2_SCANCODES[0..PS2_INDEX], error);
                PS2_INDEX = 0;
            }
        }
    }
    pic::end_irq(0x01).unwrap();
    //asm!("INT 80h");
}

//INT 30h: LAPIC Timer
#[naked] unsafe extern "x86-interrupt" fn interrupt_timer() {asm!(
    //Code
    "PUSH RAX", "PUSH RBP", "PUSH R15", "PUSH R14", //Save general registers
    "PUSH R13", "PUSH R12", "PUSH R11", "PUSH R10", //Save general registers
    "PUSH R9",  "PUSH R8",  "PUSH RDI", "PUSH RSI", //Save general registers
    "PUSH RDX", "PUSH RCX", "PUSH RBX",             //Save general registers
    "MOV RAX, [{stack_index}+RIP]",                 //Load stack index
    "SHL RAX, 3",                                   //Multiply by 8 (64-bit align)
    "LEA RCX, [{stack_array}+RIP]",                 //Load stack save loctation
    "MOV [RCX+RAX], RSP",                           //Save stack pointer
    "CALL {lapic_eoi}",                             //End interrupt
    "CALL {scheduler}",                             //Call scheduler
    "MOV RSP, RAX",                                 //Swap to thread stack
    "POP RBX", "POP RCX", "POP RDX",                //Load general registers
    "POP RSI", "POP RDI", "POP R8",  "POP R9",      //Load general registers
    "POP R10", "POP R11", "POP R12", "POP R13",     //Load general registers
    "POP R14", "POP R15", "POP RBP", "POP RAX",     //Load general registers
    "IRETQ",                                        //Enter code
    //Symbols
    stack_array  = sym TASK_STACKS,
    stack_index  = sym TASK_INDEX,
    scheduler    = sym scheduler,
    lapic_eoi    = sym lapic::end_int,
    //Options
    options(noreturn),
)}

//INT 31h: User Accessible CPU Yield
#[naked] unsafe extern "x86-interrupt" fn interrupt_yield() {asm!(
    //Code
    "PUSH RAX", "PUSH RBP", "PUSH R15", "PUSH R14", //Save general registers
    "PUSH R13", "PUSH R12", "PUSH R11", "PUSH R10", //Save general registers
    "PUSH R9",  "PUSH R8",  "PUSH RDI", "PUSH RSI", //Save general registers
    "PUSH RDX", "PUSH RCX", "PUSH RBX",             //Save general registers
    "MOV RAX, [{stack_index}+RIP]",                 //Load stack index
    "SHL RAX, 3",                                   //Multiply by 8 (64-bit align)
    "LEA RCX, [{stack_array}+RIP]",                 //Load stack save loctation
    "MOV [RCX+RAX], RSP",                           //Save stack pointer
    "CALL {scheduler}",                             //Call scheduler
    "MOV RSP, RAX",                                 //Swap to thread stack
    "POP RBX", "POP RCX", "POP RDX",                //Load general registers
    "POP RSI", "POP RDI", "POP R8",  "POP R9",      //Load general registers
    "POP R10", "POP R11", "POP R12", "POP R13",     //Load general registers
    "POP R14", "POP R15", "POP RBP", "POP RAX",     //Load general registers
    "IRETQ",                                        //Enter code
    //Symbols
    stack_array  = sym TASK_STACKS,
    stack_index  = sym TASK_INDEX,
    scheduler    = sym scheduler,
    //Options
    options(noreturn),
)}

//INT 32h: System Call
#[naked] unsafe extern "x86-interrupt" fn interrupt_syscall() {asm!(
    //Code
    "PUSH RBX", "PUSH RBP", "PUSH R12", //Save registers
    "PUSH R13", "PUSH R14", "PUSH R15", //Save registers
    "CALL {handler}",                   //Call handler
    "POP R15", "POP R14", "POP R13",    //Load registers
    "POP R12", "POP RBP", "POP RBX",    //Load registers
    "IRETQ",                            //Return
    //Symbols
    handler = sym syscall_handler,
    //Options
    options(noreturn),
)}

//INT FFh: LAPIC Spurious Interrupt
extern "x86-interrupt" fn interrupt_spurious() {unsafe {lapic::end_int()}}


// PANIC HANDLER
#[panic_handler]
unsafe fn panic_handler(panic_info: &PanicInfo) -> ! {
    cli();                                                            //Turn off interrupts
    if let Some(printer_pointer) = GLOBAL_WRITE_POINTER {             //Check for presence of write routines
        let printer = &mut *printer_pointer;                          //Find write routines
        write!(printer, "\nKernel Halt: ");                           //Begin writing
        if let Some(panic_message) = panic_info.message() {           //Check if there's an associated message
            writeln!(printer, "{}", panic_message);                   //Print the panic message
        }
        if let Some(panic_location) = panic_info.location() {         //Check if there's an associated source location
            writeln!(printer, "File:   {}", panic_location.file());   //Print the source file
            writeln!(printer, "Line:   {}", panic_location.line());   //Print the source line
            writeln!(printer, "Column: {}", panic_location.column()); //Print the source column
        }
    }
    loop {hlt();};                                                   //Halt the processor
}


// PIPING
#[repr(C)]
struct RingBuffer<TYPE, const SIZE: usize> {
    state:       RingBufferState,
    read_head:   usize,
    write_head:  usize,
    data:        [TYPE; SIZE],
}
impl<TYPE, const SIZE: usize> RingBuffer<TYPE, SIZE> where TYPE: Clone+Copy {
    pub fn write(&mut self, data: &[TYPE]) {
        for item in data {
            self.data[self.write_head] = *item;
            self.write_head += 1;
            if self.write_head == SIZE {self.write_head = 0};
        }
    }
    pub fn read<'f>(&mut self, buffer: &'f mut [TYPE]) -> &'f [TYPE] {
        let mut j = 0;
        for item in &mut *buffer {
            if self.read_head == self.write_head {break}
            *item = self.data[self.read_head];
            j +=1;
            self.read_head += 1;
            if self.read_head == SIZE {self.read_head = 0}
        }
        &buffer[0..j]
    }
}

#[repr(u8)]
#[derive(PartialEq)]
pub enum RingBufferState {
    Free = 0x00,
    WriteBlock = 0x01,
    WriteWait = 0x02,
    ReadBlock = 0x03,
    ReadWait = 0x04,
}


// TASK STATE SEGMENT
pub static mut TASK_STATE_SEGMENT: TaskStateSegment = TaskStateSegment {
    _0:    0,
    rsp0:  0,
    rsp1:  0,
    rsp2:  0,
    _1:    0,
    ist1:  0,
    ist2:  0,
    ist3:  0,
    ist4:  0,
    ist5:  0,
    ist6:  0,
    ist7:  0,
    _2:    0,
    _3:    0,
    iomba: 0,
};


// SYSTEM CALLS
//Handle
#[inline(never)]
extern "sysv64" fn syscall_handler(call_number: u64, arg1: u64, arg2: u64, arg3: u64) -> (u64, u64) {
    let mut ret = (0, 0);
    match call_number {
        0 => {ret.0 = syscall_handler_00()},
        1 => {syscall_handler_01()}
        2 => {ret.0 = syscall_handler_02()}
        _ => panic!("Invalid System Call")
    }
    ret
}

#[inline(never)]
extern "sysv64" fn syscall_handler_00() -> u64 {
    0x1111_2222_3333_4444
}

#[inline(never)]
extern "sysv64" fn syscall_handler_01() {
    unsafe {
        let printer = &mut *GLOBAL_WRITE_POINTER.unwrap();
        writeln!(printer, "SYSTEM CALL 01");
    }
}

#[inline(never)]
extern "sysv64" fn syscall_handler_02() -> u64 {
    GLOBAL_TIME.load(Ordering::SeqCst)
}
