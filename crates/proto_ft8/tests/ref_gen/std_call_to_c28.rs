// https://codingfleet.com/code-converter/fortran/rust/ - needs to be checked

use std::env;
use std::process;

const NTOKENS: u64 = 2063592;
const MAX22: u64 = 4194304;

const A1: &str = " 0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const A2: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const A3: &str = "0123456789";
const A4: &str = " ABCDEFGHIJKLMNOPQRSTUVWXYZ";

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage:   std_call_to_c28 <call_std>");
        eprintln!("Example: std_call_to_c28 K1ABC");
        process::exit(1);
    }

    let raw_arg = &args[1];

    // Simulate Fortran character*6 assignment: take up to first 6 characters,
    // then left-justify and pad on the right with spaces to length 6.
    let truncated: String = raw_arg.chars().take(6).collect();
    let call_std_assigned = format!("{:<6}", truncated);

    // Simulate Fortran's adjustr: move trailing spaces to the front.
    let call_std = adjustr(&call_std_assigned);

    // Extract individual characters (call_std is guaranteed to be 6 chars).
    let chars: Vec<char> = call_std.chars().collect();
    if chars.len() != 6 {
        eprintln!("Internal error: call_std length is not 6");
        process::exit(1);
    }
    let c1 = chars[0];
    let c2 = chars[1];
    let c3 = chars[2];
    let c4 = chars[3];
    let c5 = chars[4];
    let c6 = chars[5];

    // Find indices (0-based), equivalent to Fortran's index() - 1.
    // Print an error and exit if a character is not in the allowed set.
    let i1 = A1.find(c1).unwrap_or_else(|| {
        eprintln!("Error: character '{}' not allowed in position 1", c1);
        process::exit(1);
    }) as u64;

    let i2 = A2.find(c2).unwrap_or_else(|| {
        eprintln!("Error: character '{}' not allowed in position 2", c2);
        process::exit(1);
    }) as u64;

    let i3 = A3.find(c3).unwrap_or_else(|| {
        eprintln!("Error: character '{}' not allowed in position 3", c3);
        process::exit(1);
    }) as u64;

    let i4 = A4.find(c4).unwrap_or_else(|| {
        eprintln!("Error: character '{}' not allowed in position 4", c4);
        process::exit(1);
    }) as u64;

    let i5 = A4.find(c5).unwrap_or_else(|| {
        eprintln!("Error: character '{}' not allowed in position 5", c5);
        process::exit(1);
    }) as u64;

    let i6 = A4.find(c6).unwrap_or_else(|| {
        eprintln!("Error: character '{}' not allowed in position 6", c6);
        process::exit(1);
    }) as u64;

    let n28 = NTOKENS
        + MAX22
        + 36 * 10 * 27 * 27 * 27 * i1
        + 10 * 27 * 27 * 27 * i2
        + 27 * 27 * 27 * i3
        + 27 * 27 * i4
        + 27 * i5
        + i6;

    println!(
        "Callsign: {}  c28 as decimal integer:{:10}",
        call_std, n28
    );
}

/// Fortran's adjustr: returns the string with all trailing spaces moved to the front.
/// The length of the string remains unchanged.
fn adjustr(s: &str) -> String {
    let len = s.len();
    let trailing = s.len() - s.trim_end_matches(' ').len();
    if trailing > 0 {
        let content = &s[..len - trailing];
        " ".repeat(trailing) + content
    } else {
        s.to_string()
    }
}
