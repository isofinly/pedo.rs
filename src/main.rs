#![no_std]
#![no_main]

use core::panic::PanicInfo;

extern "C" {
    fn pipe(p: *mut [i32; 2]) -> i32;
    fn fork() -> i32;
    fn read(fd: i32, buf: *mut u8, n: i32) -> i32;
    fn write(fd: i32, buf: *const u8, n: i32) -> i32;
    fn exit(code: i32) -> !;
    fn close(fd: i32) -> i32;
    fn sleep(n: i32) -> i32;
}

const NUM_BEARS: i32 = 6;
const LETTERS: &[u8] = b"GOOOAL";
const SLEEP_TIME: i32 = 10;

// Bear ASCII art as separate lines
const BEAR_LINES: [&[u8]; 9] = [
    b"   _     _   ",
    b"  (c).-.(c)  ",
    b"   / ._. \\   ",
    b" __\\( Y )/__ ",
    b"(_.-/'-'\\-._)",
    b"   ||   ||   ",
    b" _.' `-' '._ ",
    b"(.-./`-'\\.-.)",
    b" `-'     `-' ",
];

const BEAR_WIDTH: usize = 16; // Width of each bear including spacing

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { exit(1) }
}

unsafe fn write_str(s: &[u8]) {
    write(1, s.as_ptr(), s.len() as i32);
}

// Simplified number writing function that avoids array bounds checks
unsafe fn write_num(mut n: usize) {
    // Handle special case for 0
    if n == 0 {
        write(1, &b'0', 1);
        return;
    }

    // Convert number to string, one digit at a time
    let mut divisor = 1;
    let mut temp = n;

    // Find the largest power of 10 less than n
    while temp >= 10 {
        divisor *= 10;
        temp /= 10;
    }

    // Print each digit
    while divisor > 0 {
        let digit = (n / divisor) as u8 + b'0';
        write(1, &digit, 1);
        n %= divisor;
        divisor /= 10;
    }
}

unsafe fn print_bears(active_bear: usize) {
    static mut FIRST_DRAW: bool = true;

    if FIRST_DRAW {
        // Clear screen and setup
        write_str(b"\x1b[2J"); // Clear screen
        write_str(b"\x1b[H"); // Move to home
        write_str(b"Bear circle:\n");
        write_str(b"-----------------\n\n");
        FIRST_DRAW = false;
    }

    // Print each line of bears
    for (line_idx, &line) in BEAR_LINES.iter().enumerate() {
        for bear_idx in 0..NUM_BEARS as usize {
            // Position cursor for this part of the bear
            write_str(b"\x1b[");
            write_num(line_idx + 3);
            write_str(b";");
            write_num(bear_idx * BEAR_WIDTH + 1);
            write_str(b"H");

            // Print the bear line
            write_str(line);

            // Add marker if this is the active bear
            if bear_idx == active_bear && line_idx == 5 {
                write_str(b"[");
                write(1, &LETTERS[bear_idx], 1);
                write_str(b"]");
            } else if line_idx == 5 {
                write_str(b"   ");
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn main() -> ! {
    let mut pipes = [[0i32; 2]; NUM_BEARS as usize];

    unsafe {
        // Create pipes for the circle
        for i in 0..NUM_BEARS as usize {
            if pipe(&mut pipes[i] as *mut [i32; 2]) < 0 {
                write_str(b"Failed to create pipe\n");
                exit(1);
            }
        }

        // Create bear processes
        for i in 0..NUM_BEARS {
            let pid = fork();
            if pid < 0 {
                write_str(b"Fork failed\n");
                exit(1);
            }
            if pid == 0 {
                // Child process (bear)
                let bear_idx = i as usize;
                let next_idx = ((i + 1) % NUM_BEARS) as usize;

                // Close unused pipe ends
                for j in 0..NUM_BEARS as usize {
                    if j != next_idx {
                        close(pipes[j][1]);
                    }
                    if j != bear_idx {
                        close(pipes[j][0]);
                    }
                }

                let mut token = [0u8; 1];
                loop {
                    read(pipes[bear_idx][0], token.as_mut_ptr(), 1);
                    print_bears(bear_idx);
                    sleep(SLEEP_TIME);
                    write(pipes[next_idx][1], token.as_ptr(), 1);
                }
            }
        }

        // Parent process
        for i in 0..NUM_BEARS as usize {
            if i != 0 {
                close(pipes[i][1]);
            }
            close(pipes[i][0]);
        }

        let token = [1u8; 1];
        write(pipes[0][1], token.as_ptr(), 1);

        loop {
            sleep(1000);
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    main()
}
