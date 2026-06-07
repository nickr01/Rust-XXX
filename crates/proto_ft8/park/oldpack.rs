use crate::oldconst::*;
use crate::oldtext::*;

const _NTOKENS: u32 = 2063592;
const _MAX22: u32 = 4194304;
const _MAXGRID4: u16 = 32400;

// TODO: This is wasteful, should figure out something more elegant
const _A0: &str = " 0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ+-./?";
const _A1: &str = " 0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const _A2: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const _A3: &str = "0123456789";
const _A4: &str = " ABCDEFGHIJKLMNOPQRSTUVWXYZ";

// Pack a special token, a 22-bit hash code, or a valid base call
// into a 28-bit integer.
pub fn _pack28(callsign: &str) -> i32 {
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

    int length = 0; // strlen(callsign);  // We will need it later
    while (callsign[length] != ' ' && callsign[length] != 0)
    {
        length++;
    }

    // Copy callsign to 6 character buffer
    if (starts_with(callsign, "3DA0") && length <= 7)
    {
        // Work-around for Swaziland prefix: 3DA0XYZ -> 3D0XYZ
        memcpy(c6, "3D0", 3);
        memcpy(c6 + 3, callsign + 4, length - 4);
    }
    else if (starts_with(callsign, "3X") && is_letter(callsign[2]) && length <= 7)
    {
        // Work-around for Guinea prefixes: 3XA0XYZ -> QA0XYZ
        memcpy(c6, "Q", 1);
        memcpy(c6 + 1, callsign + 2, length - 2);
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
    let mut call: Vec<char> = callsign.chars().collect();
    call.resize(6, ' ');// added NR
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

        return (_NTOKENS + _MAX22) as i32 + n28;
    }

    -1
}

pub fn _packgrid(grid4: &str) -> u16 {
    // Take care of special cases
    if grid4 == "RRR" {
        return _MAXGRID4 + 2;
    }

    if grid4 == "RR73" {
        return _MAXGRID4 + 3;
    }

    if grid4 == "73" {
        return _MAXGRID4 + 4;
    }

    let gstr: Vec<char> = grid4.chars().collect();

    if in_range(gstr[0], 'A', 'R')
        && in_range(gstr[1], 'A', 'R')
        && in_range(gstr[2], '0', '9')
        && in_range(gstr[3], '0', '9')
    {
        let mut igrid4: u16 = gstr[0] as u16 - 'A' as u16;
        igrid4 = igrid4 * 18 + (gstr[1] as u16 - 'A' as u16);
        igrid4 = igrid4 * 10 + (gstr[2] as u16 - '0' as u16);
        igrid4 = igrid4 * 10 + (gstr[3] as u16 - '0' as u16);

        return igrid4;
    }

    // Parse report: +dd / -dd / R+dd / R-dd
    // TODO: check the range of dd
    if gstr[0] == 'R' {
        let dd = dd_to_int(&grid4.chars().take(1).collect::<String>());
        let irpt = (35 + dd) as u16;
        (_MAXGRID4 + irpt) | 0x8000 // ir = 1
    } else {
        let dd = dd_to_int(grid4);
        let irpt = (35 + dd) as u16;
        _MAXGRID4 + irpt // ir = 0
    }
    //return MAXGRID4 + 1;
}

// Pack Type 1 (Standard 77-bit message) and Type 2 (ditto, with a "/P" call)
pub fn _pack77_1(msg: &str, b77: &mut [u8; FTX_LDPC_K_BYTES]) -> i32 {
    // Locate the first delimiter
    let token: Vec<&str> = msg.split(' ').collect();
    let n28a = _pack28(token[0]);
    let n28b = _pack28(token[1]);

    if n28a < 0 || n28b < 0 {
        return -1;
    }

    let igrid4 = if token.len() > 2 {
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
    b77[0] = (n28a >> 21) as u8;
    b77[1] = (n28a >> 13) as u8;
    b77[2] = (n28a >> 5) as u8;
    b77[3] = (n28a << 3) as u8 | (n28b >> 26) as u8;
    b77[4] = (n28b >> 18) as u8;
    b77[5] = (n28b >> 10) as u8;
    b77[6] = (n28b >> 2) as u8;
    b77[7] = (n28b << 6) as u8 | (igrid4 >> 10) as u8;
    b77[8] = (igrid4 >> 2) as u8;
    b77[9] = (igrid4 << 6) as u8 | (i3 << 3) as u8;

    0
}

fn _packtext77(text: &str, b77: &mut [u8; FTX_LDPC_K_BYTES]) {
    let text = text.trim();

    // Clear the first 72 bits representing a long number
    for i in 0..9 {
        b77[i] = 0;
    }

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
}

pub fn _pack77(msg: &str, c77: &mut [u8; FTX_LDPC_K_BYTES]) -> i32 {
    // Check Type 1 (Standard 77-bit message) or Type 2, with optional "/P"

    if _pack77_1(msg, c77) == 0 {
        return 0;
    }
    // TODO:
    // Check 0.5 (telemetry)

    // Check Type 4 (One nonstandard call and one hashed call)

    // Default to free text
    // i3=0 n3=0
    _packtext77(msg, c77);
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::{unpack_ft8::ft8_unpack_to_string};

    fn test_roundtrip(msg: &str) {
        
        let mut c77 = [0u8; FTX_LDPC_K_BYTES];

        let _i32_pack = _pack77(&msg, &mut c77);

        let mut msg1 = String::new();
        let _i32_unpack = crate::oldunpack::unpack77(&c77, &mut msg1);

        assert_eq!(msg, msg1);
    }

    #[test]
    fn test_min1() {
        test_roundtrip("VK2TRF VK2ZTY QG61");
        test_roundtrip("CQ VK2ZTY QG61");
    }
    
    #[test]
    fn test_min2() {
        test_roundtrip("CQ K1ABC QG61");
    }

    #[test]
    fn test_0_0() {
        test_roundtrip("TNX BOB 73 GL"); // 0.0
    }

    #[test]
    fn test_0_1() {
        test_roundtrip("K1ABC RR73; W9XYZ <KH1/KH7Z> -08"); // 0.1
    }

    #[test]
    fn test_0_3() {
        test_roundtrip("K1ABC W9XYZ 6A WI"); // 0.3 
    }

    #[test]
    fn test_0_4() {
        test_roundtrip("W9XYZ K1ABC R 17B EMA"); // 0.4
    }

    #[test]
    fn test_0_5() {
        test_roundtrip("123456789ABCDEF012"); // 0.5
    }

    #[test]
    fn test_1() {
        test_roundtrip("K1ABC W9XYZ R EN37"); // 1.
    }

    #[test]
    fn test_1_r () {
        test_roundtrip("K1ABC/R W9XYZ/R R EN37"); // 1.
    }

    #[test]
    fn test_2() {
        test_roundtrip("G4ABC/P PA9XYZ JO22"); // 2.
    }

    #[test]
    fn test_3() {
        test_roundtrip("K1ABC W9XYZ 579 WI"); // 3.
    }

    #[test]
    fn test_4() {
        test_roundtrip("<W9XYZ> PJ4/K1ABC RRR"); // 4.
    }

    #[test]
    fn test_5() {
        test_roundtrip("<G4ABC> <PA9XYZ> R 570007 JO22DB"); // 5. 
    }
}