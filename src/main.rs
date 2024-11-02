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

const BEAR_LINE1: &[u8] = b"   _     _   \n\0";
const BEAR_LINE2: &[u8] = b"  (c).-.(c)  \n\0";
const BEAR_LINE3: &[u8] = b"   / ._. \\   \n\0";
const BEAR_LINE4: &[u8] = b" __\\( Y )/__ \n\0";
const BEAR_LINE5: &[u8] = b"(_.-/'-'\\-._)\n\0";
const BEAR_LINE6_PRE: &[u8] = b"   || \0";
const BEAR_LINE6_POST: &[u8] = b" ||   \n\0";
const BEAR_LINE7: &[u8] = b" _.' `-' '._ \n\0";
const BEAR_LINE8: &[u8] = b"(.-./`-'\\.-.) \n\0";
const BEAR_LINE9: &[u8] = b" `-'     `-' \n\0";

const NUM_BEARS: i32 = 6;
const LETTERS: &[u8] = b"GOOOAL";

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { exit(1) }
}

unsafe fn write_str(s: &[u8]) {
    write(1, s.as_ptr(), (s.len() - 1) as i32);
}

unsafe fn print_bear(num: i32, letter: u8) {
    // Print newlines for spacing
    for _ in 0..10 {
        write_str(b"\n\0");
    }

    // Print bear number
    write_str(b"Bear \0");
    let mut num_buf = [0u8; 16];
    let mut i = 0;
    let mut n = num;
    loop {
        num_buf[i] = (n % 10) as u8 + b'0';
        n /= 10;
        i += 1;
        if n == 0 {
            break;
        }
    }
    while i > 0 {
        i -= 1;
        write(1, &num_buf[i] as *const u8, 1);
    }
    write_str(b":\n\0");

    write_str(BEAR_LINE1);
    write_str(BEAR_LINE2);
    write_str(BEAR_LINE3);
    write_str(BEAR_LINE4);
    write_str(BEAR_LINE5);
    write_str(BEAR_LINE6_PRE);
    write(1, &letter as *const u8, 1);
    write_str(BEAR_LINE6_POST);
    write_str(BEAR_LINE7);
    write_str(BEAR_LINE8);
    write_str(BEAR_LINE9);
}

#[no_mangle]
pub extern "C" fn main() -> ! {
    let mut pipes = [[0i32; 2]; NUM_BEARS as usize];

    unsafe {
        // Create pipes for the circle
        for i in 0..NUM_BEARS as usize {
            if pipe(&mut pipes[i] as *mut [i32; 2]) < 0 {
                write_str(b"Failed to create pipe\n\0");
                exit(1);
            }
        }

        // Create bear processes
        for i in 0..NUM_BEARS {
            let pid = fork();
            if pid < 0 {
                write_str(b"Fork failed\n\0");
                exit(1);
            }
            if pid == 0 {
                // Child process (bear)
                let bear_idx = i as usize;
                let next_idx = ((i + 1) % NUM_BEARS) as usize;

                // Close all write ends except next bear's pipe
                for j in 0..NUM_BEARS as usize {
                    if j != next_idx {
                        close(pipes[j][1]);
                    }
                }

                // Close all read ends except current bear's pipe
                for j in 0..NUM_BEARS as usize {
                    if j != bear_idx {
                        close(pipes[j][0]);
                    }
                }

                let mut token = [0u8; 1];
                loop {
                    // Wait for token
                    read(pipes[bear_idx][0], token.as_mut_ptr(), 1);

                    // Print bear
                    if let Some(&letter) = LETTERS.get(bear_idx) {
                        print_bear(i + 1, letter);
                    }

                    // Wait a bit
                    sleep(1);

                    // Pass token to next bear
                    write(pipes[next_idx][1], token.as_ptr(), 1);
                }
            }
        }

        // Parent process
        // Close all pipes except the first one's write end
        for i in 0..NUM_BEARS as usize {
            if i != 0 {
                close(pipes[i][1]);
            }
            close(pipes[i][0]);
        }

        // Start the khorovod by sending the first token
        let token = [1u8; 1];
        write(pipes[0][1], token.as_ptr(), 1);

        // Keep the parent process alive
        loop {
            sleep(1000);
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    main()
}
