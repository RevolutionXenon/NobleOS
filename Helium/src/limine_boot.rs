// HELIUM: LIMINE

// HEADER
//Imports
use gluon::x86_64::paging::MIB;

#[no_mangle] #[used(linker)] pub static LIMINE_REVISION    : limine::BaseRevision                   = limine::BaseRevision::new();
#[no_mangle] #[used(linker)] pub static LIMINE_INFO        : limine::request::BootloaderInfoRequest = limine::request::BootloaderInfoRequest::new();
#[no_mangle] #[used(linker)] pub static LIMINE_FRAMEBUFFER : limine::request::FramebufferRequest    = limine::request::FramebufferRequest::new();
#[no_mangle] #[used(linker)] pub static LIMINE_STACK       : limine::request::StackSizeRequest      = limine::request::StackSizeRequest::new().with_size((1*MIB) as u64);
#[no_mangle] #[used(linker)] pub static LIMINE_MEMMAP      : limine::request::MemoryMapRequest      = limine::request::MemoryMapRequest::new();
#[no_mangle] #[used(linker)] pub static LIMINE_HHDM        : limine::request::HhdmRequest           = limine::request::HhdmRequest::new();
#[no_mangle] #[used(linker)] pub static LIMINE_MODULES     : limine::request::ModuleRequest         = limine::request::ModuleRequest::new();
