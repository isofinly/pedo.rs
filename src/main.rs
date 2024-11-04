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
const SLEEP_TIME: i32 = 5;

const DANCE_POSITIONS: [i32; 2] = [-2, 2]; // Up and down positions
const ANIMATION_FRAMES: usize = 2;

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

unsafe fn write_num(mut n: usize) {
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

unsafe fn print_bears(active_bear: usize, frame: usize) {
    static mut FIRST_DRAW: bool = true;
    static mut COLLECTED_LETTERS: usize = 0;

    write_str(b"\x1b[2J"); // Clear screen
    write_str(b"\x1b[H"); // Move to home
    write_str(b"Bear circle:\n");
    write_str(b"-----------------\n\n");

    if FIRST_DRAW {
        COLLECTED_LETTERS = 0;
        FIRST_DRAW = false;
    }

    // Update collected letters when a new bear becomes active
    if active_bear >= COLLECTED_LETTERS {
        COLLECTED_LETTERS = active_bear + 1;
    }

    // Add extra vertical space for dancing
    let base_line = 5; // Increased base line to give room for upward movement

    // Print each line of bears
    for (line_idx, &line) in BEAR_LINES.iter().enumerate() {
        for bear_idx in 0..NUM_BEARS as usize {
            // Calculate vertical offset for dancing
            let dance_offset = if bear_idx < COLLECTED_LETTERS {
                if bear_idx % 2 == frame % 2 {
                    DANCE_POSITIONS[0]
                } else {
                    DANCE_POSITIONS[1]
                }
            } else {
                0
            };

            // Position cursor for this part of the bear with dance offset
            write_str(b"\x1b[");
            write_num((line_idx as i32 + base_line + dance_offset) as usize);
            write_str(b";");
            write_num(bear_idx * BEAR_WIDTH + 1);
            write_str(b"H");

            // Print the bear line
            write_str(line);

            // Add marker and letter if this bear has been activated
            if line_idx == 5 {
                if bear_idx < COLLECTED_LETTERS {
                    write_str(b"[");
                    write(1, &LETTERS[bear_idx], 1);
                    write_str(b"]");
                } else if bear_idx == active_bear {
                    write_str(b"[");
                    write(1, &LETTERS[bear_idx], 1);
                    write_str(b"]");
                } else {
                    write_str(b"   ");
                }
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn main() -> ! {
    let mut pipes = [[0i32; 2]; NUM_BEARS as usize];

    unsafe {
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
                let mut frame = 0;
                loop {
                    read(pipes[bear_idx][0], token.as_mut_ptr(), 1);
                    print_bears(bear_idx, frame % ANIMATION_FRAMES);
                    frame += 1;
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
