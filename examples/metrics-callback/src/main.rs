use fregate::{
    axum::{routing::get, Router},
    bootstrap, tokio, AppConfig, Application,
};
use metrics::counter;
use std::alloc::{GlobalAlloc, Layout};
use std::sync::atomic::{AtomicU64, Ordering};

static ALLOC: AtomicU64 = AtomicU64::new(0);
static DEALLOC: AtomicU64 = AtomicU64::new(0);

async fn handler() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    let config: AppConfig = bootstrap([]).unwrap();

    Application::new(&config)
        .router(Router::new().route("/", get(handler)))
        .metrics_callback(|| {
            counter!("allocations", ALLOC.load(Ordering::Relaxed));
            counter!("deallocations", DEALLOC.load(Ordering::Relaxed));
        })
        .serve()
        .await
        .unwrap();
}

pub struct SystemWrapper;

unsafe impl GlobalAlloc for SystemWrapper {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = std::alloc::System.alloc(layout);
        if !ret.is_null() {
            ALLOC.fetch_add(1, Ordering::Relaxed);
        }
        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        std::alloc::System.dealloc(ptr, layout);
        DEALLOC.fetch_add(1, Ordering::Relaxed);
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ret = std::alloc::System.alloc_zeroed(layout);
        if layout.align() <= MIN_ALIGN && layout.align() <= layout.size() && !ret.is_null() {
            ALLOC.fetch_add(1, Ordering::Relaxed);
        }
        ret
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let ret = std::alloc::System.realloc(ptr, layout, new_size);
        if layout.align() <= MIN_ALIGN && layout.align() <= layout.size() && !ret.is_null() {
            ALLOC.fetch_add(1, Ordering::Relaxed);
        }
        ret
    }
}

// FROM: std::sys::common::alloc::MIN_ALIGN
// The minimum alignment guaranteed by the architecture. This value is used to
// add fast paths for low alignment values.
#[cfg(any(
    target_arch = "x86",
    target_arch = "arm",
    target_arch = "mips",
    target_arch = "powerpc",
    target_arch = "powerpc64",
    target_arch = "sparc",
    target_arch = "asmjs",
    target_arch = "wasm32",
    target_arch = "hexagon",
    all(target_arch = "riscv32", not(target_os = "espidf")),
    all(target_arch = "xtensa", not(target_os = "espidf")),
))]
pub const MIN_ALIGN: usize = 8;
#[cfg(any(
    target_arch = "x86_64",
    target_arch = "aarch64",
    target_arch = "mips64",
    target_arch = "s390x",
    target_arch = "sparc64",
    target_arch = "riscv64",
    target_arch = "wasm64",
))]
pub const MIN_ALIGN: usize = 16;
// The allocator on the esp-idf platform guarantees 4 byte alignment.
#[cfg(any(
    all(target_arch = "riscv32", target_os = "espidf"),
    all(target_arch = "xtensa", target_os = "espidf"),
))]
pub const MIN_ALIGN: usize = 4;
