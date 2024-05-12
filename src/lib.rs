pub mod domain;
pub mod domain_rc;
pub mod index;
pub mod syntax;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
