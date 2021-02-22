use crate::color::ColorCode;
use crate::color::Color;
use volatile::Volatile;
use lazy_static::lazy_static;
use spin::Mutex;
use core::fmt;

pub const BUFFER_WIDTH     : usize = 80;
pub const BUFFER_HEIGHT    : usize = 25;
pub const BUFFER_MUL       : usize = 16; // How many windows of vga_buffer we keep for scrolling.
pub const BUFFER_ALL_HEIGHT: usize = BUFFER_HEIGHT * BUFFER_MUL;

const BUFFER_START : usize = 0xb8000;

const ASCII_START  : u8    = 0x20;
const ASCII_END    : u8    = 0x7e;
const ASCII_UNKNOWN: u8    = 0xfe;

const COLOR_FOREGROUND: Color = Color::LightGreen;
const COLOR_BACKGROUND: Color = Color::Black;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BLANK_CHAR: ScreenChar = ScreenChar {
    ascii_character: b' ',
    color_code: ColorCode::new(COLOR_FOREGROUND, COLOR_BACKGROUND),
};

#[repr(transparent)]
struct VgaBuffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    row_position: usize,     // Row position within Buffer.
    row_start: usize,        // Start of the window within Buffer.
    color_code: ColorCode,
    vga_buffer: &'static mut VgaBuffer,
    buffer: &'static mut [[ScreenChar; BUFFER_WIDTH]; BUFFER_ALL_HEIGHT]
}

impl Writer {
    fn update(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer[self.row_start + row][col];
                self.vga_buffer.chars[row][col].write(character);
            }
        }
    }

    pub fn move_window_up(&mut self) {
        if self.row_start <= 0 {
            self.row_start = 0;
        } else {
            self.row_start -= 1;
        }

        self.update();
    }

    pub fn move_window_down(&mut self) {
        if self.row_start >= BUFFER_ALL_HEIGHT - BUFFER_HEIGHT {
            self.row_start = BUFFER_ALL_HEIGHT - BUFFER_HEIGHT;
        } else {
            self.row_start += 1;
        }

        self.update();
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte  => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }   

                let row = self.row_position;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer[row][col] = ScreenChar {
                    ascii_character: byte,
                    color_code,
                };
                self.column_position += 1;
            }
        }

        //self.update();
    }

    fn new_line(&mut self) {
        self.column_position = 0;

        if self.row_position - self.row_start >= BUFFER_HEIGHT - 1 {
            self.row_start += 1;
        }

        if self.row_position >= BUFFER_ALL_HEIGHT - 1 {
            self.row_start = BUFFER_ALL_HEIGHT - BUFFER_HEIGHT;

            for row in 1..BUFFER_ALL_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let character = self.buffer[row][col];
                    self.buffer[row - 1][col] = character;
                }
            }

            self.clear_row(BUFFER_ALL_HEIGHT - 1);

            return;
        }

        self.row_position += 1;

        self.update();
    }

    fn clear_row(&mut self, row: usize) {
        let blank_char = BLANK_CHAR;
        for col in 0..BUFFER_WIDTH {
            self.buffer[row][col] = blank_char;
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // Printable ASCII byte or newline
                ASCII_START..=ASCII_END | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(ASCII_UNKNOWN),
            }
        }

        self.update();
    }

    // Writes a format string at position (col, row).
    // abs: determines whether the writing is absolute to the buffer or relative
    // to the window frame.
    pub fn write_at(&mut self, abs: bool, row: usize, col: usize, args: fmt::Arguments) {
        use core::fmt::Write;

        let prev_row = self.row_position;
        let prev_col = self.column_position;

        self.row_position    = (if abs {0} else {self.row_start}) + row;
        self.column_position = col;

        self.write_fmt(args).unwrap();

        self.row_position    = prev_row;
        self.column_position = prev_col;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// The static WRITER, an interface to access the writing process.
// Initialized lazily; to provide mutability, we use a spinlock
// to ensure Sync property.

// Creating a static mut array, which will be used by the writer.
// We can omit the whole lazy_static thing here because the
// access to the writer is thread-safe.
static mut _BUFFER: &mut [[ScreenChar; BUFFER_WIDTH]; BUFFER_ALL_HEIGHT] =
                    &mut [[BLANK_CHAR; BUFFER_WIDTH]; BUFFER_ALL_HEIGHT];
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        row_position   : 0,
        row_start      : 0,
        color_code: ColorCode::new(COLOR_FOREGROUND, COLOR_BACKGROUND),
        vga_buffer: unsafe { &mut *(BUFFER_START as *mut VgaBuffer) },
        buffer    : unsafe { _BUFFER }
    });
}

// Next we define our orintln! macros.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print_at {
    ($r:expr, $c:expr, $($arg:tt)*) =>
        ($crate::vga_buffer::_print_at($r, $c, format_args!($($arg)*)));
}

// Print at absolute position (col, row) in Buffer.
#[macro_export]
macro_rules! print_at_abs {
    ($r:expr, $c:expr, $($arg:tt)*) =>
        ($crate::vga_buffer::_print_at_abs($r, $c, format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts; 
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[doc(hidden)]
pub fn _print_at(row: usize, col: usize, args: fmt::Arguments) {
    use x86_64::instructions::interrupts; 
    interrupts::without_interrupts(|| {
        WRITER.lock().write_at(false, row, col, args);
    });
}

#[doc(hidden)]
pub fn _print_at_abs(row: usize, col: usize, args: fmt::Arguments) {
    use x86_64::instructions::interrupts; 
    interrupts::without_interrupts(|| {
        WRITER.lock().write_at(true, row, col, args);
    });
}

pub fn move_up() {
    use x86_64::instructions::interrupts; 
    interrupts::without_interrupts(|| {
        WRITER.lock().move_window_up();
    });
}

pub fn move_down() {
    use x86_64::instructions::interrupts; 
    interrupts::without_interrupts(|| {
        WRITER.lock().move_window_down();
    });
}

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.vga_buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });
}
