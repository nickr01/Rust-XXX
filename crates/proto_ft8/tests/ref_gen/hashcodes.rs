// https://codingfleet.com/code-converter/fortran/rust/ - needs to be checked

use std::env;
use std::process;

/// Number of tokens (parameter from original Fortran code)
const NTOKENS: i32 = 2_063_592;

/// Large prime constant (integer*8 in Fortran)
const NPRIME: u64 = 47_055_833_459;

/// Bit sizes for the three hash functions
const NBITS: [i32; 3] = [10, 12, 22];

/// Character set for base-38 conversion.
/// Must match the exact order: space, 0-9, A-Z, /
const CHARS: &str = " 0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ/";

fn main() {
    // Expect exactly one argument (the callsign)
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage:    hashcodes <callsign>");
        eprintln!("Examples: hashcodes PJ4/K1ABC");
        eprintln!("          hashcodes YW18FIFA");
        process::exit(1);
    }

    // Mimic Fortran's: getarg into character*11 + adjustl
    let raw_callsign = &args[1];
    let callsign = fortran_getarg_and_adjustl(raw_callsign, 11);

    // Validate that every character is in the allowed set
    for ch in callsign.chars() {
        if !CHARS.contains(ch) {
            eprintln!("Error: invalid character '{}' in callsign", ch);
            process::exit(1);
        }
    }

    let mut n8 = [0u64; 3];
    let mut ihash = [0i32; 3];

    // Calculate the three hash codes
    for k in 0..3 {
        // Build base-38 number from the 11-character callsign
        for ch in callsign.chars() {
            let j = CHARS.find(ch).unwrap() as u64; // 0..37
            n8[k] = n8[k] * 38 + j;
        }

        // Compute: ihash(k) = ishft(nprime * n8(k), nbits(k)-64)
        let shift = NBITS[k] - 64;
        let product = NPRIME.wrapping_mul(n8[k]); // modulo 2^64 multiplication
        let shifted = if shift < 0 {
            // Right shift (logical)
            product >> ((-shift) as u32)
        } else {
            product << (shift as u32)
        };
        ihash[k] = shifted as i32;
    }

    // Biased hash for storage in c28
    let ih22_biased = ihash[2] + NTOKENS;

    // Output formatting exactly as in the original Fortran program
    // Header line
    println!("Callsign         h10       h12       h22");
    // Dashes (41 dashes)
    println!("-----------------------------------------");
    // Data line: callsign (11 chars), then three hash values with fixed widths
    println!(
        "{:<11}{:>9}{:>10}{:>10}",
        callsign, ihash[0], ihash[1], ihash[2]
    );
    // Biased line
    println!("Biased for storage in c28:{:>14}", ih22_biased);
}

/// Simulates Fortran's `getarg` into a `character*11` variable followed by `adjustl`.
///
/// Steps:
/// 1. Truncate or pad the raw argument to exactly `width` characters.
/// 2. Left-justify the contents: remove leading spaces, shift remaining chars left,
///    and fill the rest with spaces to keep the length fixed.
fn fortran_getarg_and_adjustl(raw: &str, width: usize) -> String {
    // Step 1: fill a fixed-width buffer (mimics character*11 assignment)
    let mut fixed = String::with_capacity(width);
    for ch in raw.chars().take(width) {
        fixed.push(ch);
    }
    while fixed.len() < width {
        fixed.push(' ');
    }

    // Step 2: adjustl – left‑justify inside the fixed width
    if let Some(pos) = fixed.find(|c: char| c != ' ') {
        if pos > 0 {
            let non_space = fixed[pos..].to_string();
            fixed = non_space;
            while fixed.len() < width {
                fixed.push(' ');
            }
        }
    }
    // If all characters are spaces, leave the string unchanged.

    fixed
}
