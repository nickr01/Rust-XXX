use crate::*;

// TODO: This is wasteful, should figure out something more elegant
const _A0: &str = " 0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ+-./?";
const _A1: &str = " 0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const _A2: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const _A3: &str = "0123456789";
const _A4: &str = " ABCDEFGHIJKLMNOPQRSTUVWXYZ";

// Pack a special token, a 22-bit hash code, or a valid base call
// into a 28-bit integer.
pub fn _pack28(callsign: &str) -> i32 {
    const NTOKENS: u32 = 2063592;
    const MAX22: u32 = 4194304;

    // Check for special tokens first
    if callsign.starts_with("DE") {
        return 0;
    }

    if callsign.starts_with("QRZ") {
        return 1;
    }

    if callsign.starts_with("CQ") {
        return 2;
    }

    if callsign.starts_with("CQ_") {
        //int nnum = 0, nlet = 0;
        // TODO:
    }

    // TODO: Check for <...> callsign
    /*
    char c6[6] = { ' ', ' ', ' ', ' ', ' ', ' ' };

    int length = 0; //strlen(callsign);  //We will need it later
    while (callsign[length] != ' ' && callsign[length] != 0)
    {
        length++;
    }

    // Copy callsign to 6 character buffer
    if (starts_with(callsign, "3DA0") && length <= 7)
    {
        // Work-around for Swaziland prefix: 3DA0XYZ -> 3D0XYZ
        memcpy(c6, "3D0", 3);
        memcpy(c6 + 3, callsign + 4, length -4);
    }
    else if (starts_with(callsign, "3X") && is_letter(callsign[2]) && length <= 7)
    {
        // Work-around for Guinea prefixes: 3XA0XYZ -> QA0XYZ
        memcpy(c6, "Q", 1);
        memcpy(c6 + 1, callsign + 2, length -2);
    }
    else
    {
        if (is_digit(callsign[2]) && length <= 6)
        {
            // AB0XYZ
            memcpy(c6, callsign, length);
        }
        else if (is_digit(callsign[1]) && length <= 5)
        {
            // A0XYZ -> " A0XYZ"
            memcpy(c6 + 1, callsign, length);
        }
    }
    */
    // Check for standard callsign
    let call: Vec<char> = callsign.chars().collect();
    if let (Some(i0), Some(i1), Some(i2), Some(i3), Some(i4), Some(i5)) = (
        _A1.find(call[0]),
        _A2.find(call[1]),
        _A3.find(call[2]),
        _A4.find(call[3]),
        _A4.find(call[4]),
        _A4.find(call[5]),
    ) {
        let mut n28: i32 = i0 as i32;
        n28 = n28 * 36 + i1 as i32;
        n28 = n28 * 10 + i2 as i32;
        n28 = n28 * 27 + i3 as i32;
        n28 = n28 * 27 + i4 as i32;
        n28 = n28 * 27 + i5 as i32;

        return (NTOKENS + MAX22) as i32 + n28;
    }

    -1
}

pub fn _packgrid(grid4: &str) -> u16 {
    const MAXGRID4: u16 = 32400;

    // Take care of special cases
    if grid4 == "RRR" {
        return MAXGRID4 + 2;
    }

    if grid4 == "RR73" {
        return MAXGRID4 + 3;
    }

    if grid4 == "73" {
        return MAXGRID4 + 4;
    }

    let gstr: Vec<char> = grid4.chars().collect();

    if text::_in_range(gstr[0], 'A', 'R')
        && text::_in_range(gstr[1], 'A', 'R')
        && text::_in_range(gstr[2], '0', '9')
        && text::_in_range(gstr[3], '0', '9')
    {
        let mut igrid4: u16 = gstr[0] as u16 - 'A' as u16;
        igrid4 = igrid4 * 18 + (gstr[1] as u16 - 'A' as u16);
        igrid4 = igrid4 * 10 + (gstr[2] as u16 - '0' as u16);
        igrid4 = igrid4 * 10 + (gstr[3] as u16 - '0' as u16);

        return igrid4;
    }

    // Parse report: +dd /-dd /R+dd /R-dd
    // TODO: check the range of dd
    if gstr[0] == 'R' {
        let dd = text::_dd_to_int(&grid4.chars().take(1).collect::<String>());
        let irpt = (35 + dd) as u16;
        (MAXGRID4 + irpt) | 0x8000 // ir = 1
    } else {
        let dd = text::_dd_to_int(grid4);
        let irpt = (35 + dd) as u16;
        MAXGRID4 + irpt // ir = 0
    }
    //return MAXGRID4 + 1;
}

// Pack Type 1 (Standard 77-bit message) and Type 2 (ditto, with a "/P" call)
pub fn _pack77_1(msg: &str) -> Option<Vec<u8>> {
    // Locate the first delimiter
    let token: Vec<&str> = msg.split(' ').collect();
    let n28a = _pack28(token[0]);
    let n28b = _pack28(token[1]);

    if n28a < 0 || n28b < 0 {
        return None;
    }

    let igrid4 =  if token.len() > 2 { 
        _packgrid(token[2]) 
    } else {
        // Two callsigns, no grid/report
        _packgrid(" ")
    };

    let i3 = 1u8; // No suffix or /R

    // TODO: check for suffixes

    // Shift in ipa and ipb bits into n28a and n28b
    let n28a = (n28a as u32) << 1; // ipa = 0
    let n28b = (n28b as u32) << 1; // ipb = 0

    // Pack into (28 + 1) + (28 + 1) + (1 + 15) + 3 bits
    let mut b77 = Vec::with_capacity(protocol::FT8._ldpc_k_bytes().0);
    b77.push((n28a >> 21) as u8);
    b77.push((n28a >> 13) as u8);
    b77.push((n28a >> 5) as u8);
    b77.push((n28a << 3) as u8 | (n28b >> 26) as u8);
    b77.push((n28b >> 18) as u8);
    b77.push((n28b >> 10) as u8);
    b77.push((n28b >> 2) as u8);
    b77.push((n28b << 6) as u8 | (igrid4 >> 10) as u8);
    b77.push((igrid4 >> 2) as u8);
    b77.push((igrid4 << 6) as u8 | (i3 << 3));
    
    Some(b77)
}

pub fn _packtext77(text: &str) -> Vec<u8> {
    let text = text.trim();

    let mut b77 = Vec::with_capacity(protocol::FT8._ldpc_k_bytes().0);
    // Clear the first 72 bits representing a long number
    b77.resize(9, 0);
    // for i in 0..9 {
    //     b77[i] = 0;
    // }

    // Now express the text as base-42 number stored
    // in the first 72 bits of b77
    for j in 0..13 {
        // Multiply the long integer in b77 by 42
        let mut x = 0u16;
        for i in (0..8).rev() {
            x += b77[i] as u16 * 42u16;
            b77[i] = (x & 0xFF) as u8;
            x >>= 8;
        }

        // Get the index of the current char
        if j < text.len() {
            if let Some(c) = text.chars().nth(j) {
                if let Some(q) = _A0.find(c) {
                    x = if q > 0 { q as u16 } else { 0 };
                } else {
                    x = 0;
                }
            } else {
                x = 0;
            }
        } else {
            x = 0;
        }
        // Here we double each added number in order to have the result multiplied
        // by two as well, so that it's a 71 bit number left-aligned in 72 bits (9 bytes)
        x <<= 1;

        // Now add the number to our long number
        for i in (0..8).rev() {
            if x == 0 {
                break;
            }

            x += b77[i] as u16;
            b77[i] = (x & 0xFF) as u8;
            x >>= 8;
        }
    }
    // Set n3=0 (bits 71..73) and i3=0 (bits 74..76)
    b77[8] &= 0xFE;
    b77[9] &= 0x00;

    b77
}

pub fn _pack77(msg: &str) -> Vec<u8>  {
    // Check Type 1 (Standard 77-bit message) or Type 2, with optional "/P"
    match _pack77_1(msg) {
        Some(b77) => b77,
        None => {
            // TODO:
            // Check 0.5 (telemetry)

            // Check Type 4 (One nonstandard call and one hashed call)

            // Default to free text
            // i3=0 n3=0
            _packtext77(msg)
        }
    }
}
