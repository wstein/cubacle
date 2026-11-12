//! Host imports the WebAssembly module needs from `env`.

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
