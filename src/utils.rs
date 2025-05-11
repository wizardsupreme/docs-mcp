pub fn set_panic_hook() {
    // Better panic messages in console.
    console_error_panic_hook::set_once();
}