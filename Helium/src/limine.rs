// HELIUM: LIMINE

// HEADER
//Flags

//Imports
use gluon::x86_64::paging::MIB;
use limine::*;


// REQUESTS
#[no_mangle] pub static REQ_INFO:        LimineBootInfoRequest    = LimineBootInfoRequest::new(0);
#[no_mangle] pub static REQ_FRAMEBUFFER: LimineFramebufferRequest = LimineFramebufferRequest::new(0);
#[no_mangle] pub static REQ_STACK:       LimineStackSizeRequest   = LimineStackSizeRequest::new(0).stack_size((8*MIB) as u64);
#[no_mangle] pub static REQ_MEMMAP:      LimineMmapRequest        = LimineMmapRequest::new(0);


// LIMINE REQS SECTION
#[no_mangle] #[link_section=".limine_reqs"] static REQ_INFO_PTR:        &LimineBootInfoRequest    = &REQ_INFO;
#[no_mangle] #[link_section=".limine_reqs"] static REQ_FRAMEBUFFER_PTR: &LimineFramebufferRequest = &REQ_FRAMEBUFFER;
#[no_mangle] #[link_section=".limine_reqs"] static REQ_STACK_PTR:       &LimineStackSizeRequest   = &REQ_STACK;
#[no_mangle] #[link_section=".limine_reqs"] static REQ_MEMMAP_PTR:      &LimineMmapRequest        = &REQ_MEMMAP;
#[no_mangle] #[link_section=".limine_reqs"] static REQ_END:             usize                     = 0;
