use core::panic::PanicInfo;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    // TODO: Provide more sophisticatd panic handling! For example, print an error message or even
    // better propagate it to ESP-IDF panic/abort handling.
    loop {}
}
