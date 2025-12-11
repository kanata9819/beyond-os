#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::beyond_console::_print(core::format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n");
    };
    ($($arg:tt)*) => {
        $crate::beyond_console::_print(core::format_args!($($arg)*));
        $crate::print!("\n");
    };
}
