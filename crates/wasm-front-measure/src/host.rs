//! Host imports the WebAssembly module needs from `env`.

/// Reports the progress of the table build (thousandths) to the host; the
/// host returns nonzero to cancel the build.
#[cfg(target_arch = "wasm32")]
pub fn progress(permille: u16) -> bool {
    #[link(wasm_import_module = "env")]
    extern "C" {
        fn cubacle_host_progress(permille: u32) -> u32;
    }
    // SAFETY: a plain import call with a number; the host returns a number.
    unsafe { cubacle_host_progress(u32::from(permille)) == 0 }
}

/// Outside WebAssembly nothing watches the build.
#[cfg(not(target_arch = "wasm32"))]
pub fn progress(_permille: u16) -> bool {
    true
}

#[cfg(all(target_arch = "wasm32", getrandom_backend = "custom"))]
mod random {
    #[link(wasm_import_module = "env")]
    extern "C" {
        fn cubacle_host_random(pointer: *mut u8, len: usize);
    }

    /// The `getrandom` custom backend: the host fills the buffer.
    ///
    /// # Safety
    /// Called by `getrandom` with a writable buffer.
    #[no_mangle]
    unsafe extern "Rust" fn __getrandom_v03_custom(
        dest: *mut u8,
        len: usize,
    ) -> Result<(), getrandom::Error> {
        // SAFETY: `dest` is writable for `len` bytes, as `getrandom` guarantees.
        unsafe { cubacle_host_random(dest, len) };
        Ok(())
    }
}
