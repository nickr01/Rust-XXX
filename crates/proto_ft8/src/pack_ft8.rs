// use crate::*;
use crate::protocol::*;
use crate::text;

// Pack a special token, a 22-bit hash code, or a valid base call
// into a 28-bit integer.
fn ft8_pack_std_call(callsign: &str) -> Option<u32> {
    dbg!(callsign);

    // Check for special tokens first
    if callsign.starts_with("DE") {
        return Some(C28_DE);
    }

    if callsign.starts_with("QRZ") {
        return Some(C28_QRZ);
    }

    if callsign.starts_with("CQ") {
        return Some(C28_CQ);
    }

    if callsign.starts_with("CQ_") {
        //int nnum = 0, nlet = 0;
        todo!();
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

    let mut n28 = 0u32;
    for (i, c) in call.iter().enumerate() {
        let (opt_n, l) = match i {
            0 => { (CALL_A1.find(*c), CALL_A1_LEN) },
            1 => { (CALL_A2.find(*c), CALL_A2_LEN) },
            2 => { (CALL_A3.find(*c), CALL_A3_LEN) },
            3 => { (CALL_A4.find(*c), CALL_A4_LEN) },
            4 => { (CALL_A4.find(*c), CALL_A4_LEN) },
            5 => { (CALL_A4.find(*c), CALL_A4_LEN) },
            _ => { 
                dbg!("Too many callsign digits");
                return None;
            }
        };
        match opt_n {
            Some(n) => {
                n28 += n28 * l + n as u32;
            },
            None => {
                dbg!("Invalid callsign character for character position", c, i);
                return None;
            }
        }
    }
    Some(C28_STD_CALLS + n28)

    // if let (Some(i0), Some(i1), Some(i2), Some(i3), Some(i4), Some(i5)) = (
    //     _A1.find(call[0]),
    //     _A2.find(call[1]),
    //     _A3.find(call[2]),
    //     _A4.find(call[3]),
    //     _A4.find(call[4]),
    //     _A4.find(call[5]),
    // ) {
    //     let mut n28: i32 = i0 as i32;
    //     n28 = n28 * _A2_LEN + i1 as i32;
    //     n28 = n28 * _A3_LEN + i2 as i32;
    //     n28 = n28 * _A4_LEN + i3 as i32;
    //     n28 = n28 * _A4_LEN + i4 as i32;
    //     n28 = n28 * _A4_LEN + i5 as i32;

    //     return (NTOKENS + MAX22) as i32 + n28;
    // }

    // -1
}

fn ft8_pack_grid4(grid4: &str) -> Option<u16> {
    dbg!(grid4);

    // Take care of special cases
    if grid4 == "RRR" {
        return Some(MAXGRID4 + 2);
    }

    if grid4 == "RR73" {
        return Some(MAXGRID4 + 3);
    }

    if grid4 == "73" {
        return Some(MAXGRID4 + 4);
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

        return Some(igrid4);
    }

    // Parse report: +dd /-dd /R+dd /R-dd
    // TODO: check the range of dd
    if gstr[0] == 'R' {
        let dd = text::_dd_to_int(&grid4.chars().take(1).collect::<String>());
        let irpt = (35 + dd) as u16;
        return Some((MAXGRID4 + irpt) | 0x8000); // ir = 1
    } else {
        let dd = text::_dd_to_int(grid4);
        let irpt = (35 + dd) as u16;
        return Some(MAXGRID4 + irpt); // ir = 0
    }
    
    // !?&*!!! how do we get here?
    Some(MAXGRID4 + 1)
}

// Pack Type 1 (Standard 77-bit message) and Type 2 (ditto, with a "/P" call)
fn ft8_pack_type1(type1_msg: &str) -> Option<Vec<u8>> {
    dbg!(type1_msg);

    // Locate the first delimiter
    let token: Vec<&str> = type1_msg.split(' ').collect();
    let n28a = ft8_pack_std_call(token[0]);
    let n28b = ft8_pack_std_call(token[1]);

    if !n28a.is_some() || !n28b.is_some() {
        return None;
    }

    let n28a = n28a.unwrap();
    let n28b = n28b.unwrap();

    let igrid4 =  ft8_pack_grid4(if token.len() > 2 { token[2] } else { " " })
        .expect("expected a packed grid4");

    let i3 = 1u8; // No suffix or /R

    // TODO: check for suffixes

    // Shift in ipa and ipb bits into n28a and n28b
    let n28a = (n28a as u32) << 1; // ipa = 0
    let n28b = (n28b as u32) << 1; // ipb = 0

    // Pack into (28 + 1) + (28 + 1) + (1 + 15) + 3 bits
    let mut b77 = Vec::with_capacity(FT8._ldpc_k_bytes().0);
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

fn ft8_pack_type3(type3_msg: &str) -> Option<Vec<u8>> {
    dbg!(type3_msg);

    let text = type3_msg.trim();

    let mut b77 = Vec::with_capacity(FT8._ldpc_k_bytes().0);
    // Clear the first 72 bits representing a long number
    b77.resize(FT8._ldpc_k_bytes().0, 0);
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
                if let Some(q) = CALL_A0.find(c) {
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

    Some(b77)
}

pub fn ft8_pack_string(msg: &str) -> Option<Vec<u8>>  {
    dbg!(msg);

    // Check Type 1 (Standard 77-bit message) or Type 2, with optional "/P"
    match ft8_pack_type1(msg) {
        Some(b77) => Some(b77),
        None => {
            // TODO:
            // Check 0.5 (telemetry)

            // Check Type 4 (One nonstandard call and one hashed call)

            // Default to free text
            // i3=0 n3=0
            ft8_pack_type3(msg)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{unpack_ft8::ft8_unpack_to_string};

    fn test_roundtrip(msg: &str) {
        let cw = ft8_pack_string(&msg)
            .expect("Could not ft8_pack");
        let msg1 = ft8_unpack_to_string(&cw)
            .expect("Could not ft8_unpack");
        assert_eq!(msg, msg1);
    }

    #[test]
    fn test() {
        // test_roundtrip("TNX BOB 73 GL"); // 0.0
        // test_roundtrip("K1ABC RR73; W9XYZ <KH1/KH7Z> -08"); // 0.1
        // test_roundtrip("K1ABC W9XYZ 6A WI"); // 0.3 
        // test_roundtrip("W9XYZ K1ABC R 17B EMA"); // 0.4
        // test_roundtrip("123456789ABCDEF012"); // 0.5
        // test_roundtrip("K1ABC/R W9XYZ/R R EN37"); // 1.
        // test_roundtrip("G4ABC/P PA9XYZ JO22"); // 2.
        // test_roundtrip("K1ABC W9XYZ 579 WI"); // 3.
        // test_roundtrip("<W9XYZ> PJ4/K1ABC RRR"); // 4.
        // test_roundtrip("<G4ABC> <PA9XYZ> R 570007 JO22DB"); // 5. 
        test_roundtrip("CQ VK2ZTY QG61");
    }

}