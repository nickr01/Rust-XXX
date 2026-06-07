// https://codingfleet.com/code-converter/fortran/rust/ - needs to be checked
use std::env;

/// The 42-character set used for encoding, exactly as in the original Fortran code.
/// The first character is a space.
const CHARSET: &str = " 0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ+-./?";

/// Returns the 0‑based index of `c` in `CHARSET`, or 0 if not found.
/// This replicates `index(c, w(i:i)) - 1` in the Fortran code.
fn char_index(c: char) -> u8 {
    CHARSET.find(c).map_or(0, |i| i as u8)
}

/// Replicates the Fortran behaviour of `getarg` + `adjustr`:
/// 1. Truncate to the first 13 characters if the argument is longer.
/// 2. Left‑justify and pad with spaces on the right to exactly 13 characters.
/// 3. Right‑justify by moving all trailing spaces to the front.
fn prepare_message(arg: &str) -> String {
    // Step 1 & 2: ensure exactly 13 characters, left‑justified with trailing spaces.
    let s = if arg.len() > 13 {
        arg[..13].to_string()
    } else {
        format!("{: <13}", arg) // left‑justify, pad to width 13 with spaces
    };
    // Step 3: move trailing spaces to the left (adjustr).
    let trimmed = s.trim_end();
    let n_spaces = s.len() - trimmed.len();
    let mut result = " ".repeat(n_spaces);
    result.push_str(trimmed);
    result
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage:   free_text_to_f71 \"<message>\"");
        eprintln!("Example: free_text_to_f71 \"TNX BOB 73 GL\"");
        return;
    }
    let input = &args[1];

    // Prepare the 13‑character right‑justified message (w in Fortran).
    let w = prepare_message(input);

    // Multi‑precision arithmetic replaced by simple u128 operations.
    // The original code works base‑256 and uses only 9 bytes; the maximum
    // encoded value fits comfortably in 71 bits, so a 128‑bit integer is safe.
    let mut val: u128 = 0;
    for c in w.chars() {
        let j = char_index(c) as u128;
        val = val * 42 + j;
    }

    // Format the 71‑bit binary string, most significant bit first.
    // This replaces the Fortran `write(f71,1000) qa(2:10)` with format `b7.7,8b8.8`.
    let f71 = format!("{:071b}", val);

    // Prepare the string printed as `c13` (original input, left‑justified/padded to 13).
    let display_input = if input.len() > 13 {
        input[..13].to_string()
    } else {
        format!("{: <13}", input)
    };

    println!("Free text: {}", display_input);
    println!("f71: {}", f71);
}
