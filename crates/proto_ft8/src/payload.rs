use audioadapter_buffers::direct::SparseSequentialSliceOfSlices;

// Pack and Unpack with unit tests
use crate::protocol::*;
use crate::text;

// need for bitvec Traits
// use bitvec::prelude::*;

#[derive(PartialEq, Debug, Clone)]
pub struct CallId {
    pub id: String,
    special: bool,          
}

impl CallId {
    pub fn new() -> CallId {
        CallId {
            id: String::new(),
            special: false,          
        }
    }
}

#[derive(Debug)]
pub struct Ft8Msg {
    pub call_to: CallId,
    pub call_from: CallId,
    pub extra: String,
}

impl Ft8Msg {
    pub fn new() -> Ft8Msg {
        Ft8Msg{
            call_to: CallId::new(),
            call_from: CallId::new(),
            extra: String::new(),
        }
    }

    pub fn to_string(&self) -> Option<String> {
        let mut stg = String::new();

        if !self.call_to.id.is_empty() {
            stg.push_str(&self.call_to.id);
            stg.push(' ');
        }

        if !self.call_from.id.is_empty() {
            stg.push_str(&self.call_from.id);
            stg.push(' ');
        }

        if !self.extra.is_empty() {
            stg.push_str(&self.extra);
        }

        if stg.is_empty() {
            dbg!("FT8 unpacked to empty string");
            None
        } else {
            Some(stg)
        }
    }
}

//----------------------------------------------------------

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

const A71_BYTES: usize = 9; // (71+1)/8
type A71 = [u8; A71_BYTES];

const FREE_TEXT_LEN: usize = 13;
const FREE_TEXT_PAD: char = ' ';
const FREE_TEXT_CHARSET: &str = " 0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ+-./?";
const FREE_TEXT_CHARSET_LEN: usize = FREE_TEXT_CHARSET.len();  // 42
assert_eq!(FREE_TEXT_CHARSET, CALL_A0);

const TELEM_TEXT_LEN: usize = 18;
const TELEM_TEXT_PAD: char = '0';
const TELEM_TEXT_CHARSET: &str = "0123456789ABCDEF";
const TELEM_TEXT_CHARSET_LEN: usize = TELEM_TEXT_CHARSET.len(); // 16

const FT8_MESSAGE_BITS: usize = 71;

// - Leading and trailing whitespace removed
// - Trim to len
// - Right justify with pad_chars
fn left_pad(input_string: &String, len_var: usize, pad_char: char) -> String {
    let mut trimmed = input_string.trim().to_string();
    trimmed.truncate(len_var);
    {
        let left_pad_len: isize = len_var as isize - trimmed.len() as isize ;
        if left_pad_len > 0 {
            let pad_string: String = pad_char.to_string();
            let mut left_padded = pad_string.repeat(left_pad_len as usize);
            left_padded.push_str(&trimmed);
            trimmed = left_padded;
        }
    }
    trimmed
}

// - Accept a char string
// - Trim, truncate and right justify
// - Then build sum with num base of the char array length
// - Assume can build the output bitmap in u128 - might as well pass that around initially
fn ft8_pack_0_ (
    cn: String,
    bits: usize,
    charset: &str,
    pad_char: char,
) -> Option<u128> {
    let cn = left_pad(&cn,charset.len(), pad_char);
    let mut val: u128 = 0;
    for c in cn.chars() {
        match charset.find(c).map_or(None, |i| Some(i as u8)) {
            Some(j) => {
                val = val * charset.len() as u128 + j as u128;
            },
            None => {
                dbg!("invalid character in 0_0:free text");
                return None;
            }
        }
    }
    assert!(bits < 128);
    let val_max = 2u128.pow(bits as u32);
    if val < val_max {
        Some(val)
    } else {
        dbg!("packed data out of range");
        None
    }
}

fn ft8_pack_0_0 (
    c13: String,
) -> Option<u128> {
    ft8_pack_0_(c13, FT8_MESSAGE_BITS, FREE_TEXT_CHARSET, ' ')
}

fn ft8_pack_0_5 (
    c18: String,
) -> Option<u128> {
    ft8_pack_0_(c18, FT8_MESSAGE_BITS, TELEM_TEXT_CHARSET, '0')
}

fn ft8_unpack_0_0(
    a71: &[u8]
) -> Option<String> {
    dbg!(a71);

    let mut text = String::new();
    // todo!("test");

    let mut b71 = [0u8; 9];

    // Shift 71 bits right by 1 bit, so that it's right-aligned in the byte array
    let mut carry = 0;
    for i in 0..9 {
        b71[i] = carry | (a71[i] >> 1);
        carry = if (a71[i] & 1) != 0 { 0x80 } else { 0 };
    }

    let mut c14 = String::new();

    for _idx in 0..13 {
        // Divide the long integer in b71 by 42
        let mut rem = 0u16;
        for b in &mut b71 {
            rem = (rem << 8) | (*b as u16);
            *b = (rem / 42) as u8;
            rem %= 42;
        }
        c14.push(text::charn(rem as u8, 0));
    }

    text.push_str(c14.chars().rev().collect::<String>().trim());
    
    Some(text) // Success
}

fn ft8_unpack_type0_5(
    a71: &[u8], 
) -> Option<String> {
    dbg!(a71);

    let mut telemetry = String::new();
    let mut b71 = [0u8; 9];

    // Shift bits in a71 right by 1 bit
    let mut carry = 0u8;
    for i in 0..9 {
        b71[i] = (carry << 7) | (a71[i] >> 1);
        carry = a71[i] & 0x01;
    }

    // Convert b71 to hexadecimal string
    for b in &b71 {
        let nibble1 = *b >> 4;
        let nibble2 = *b & 0x0F;
        let c1 = if nibble1 > 9 {
            (nibble1 - 10 + b'A') as char
        } else {
            (nibble1 + b'0') as char
        };
        let c2 = if nibble2 > 9 {
            (nibble2 - 10 + b'A') as char
        } else {
            (nibble2 + b'0') as char
        };
        telemetry.push(c1);
        telemetry.push(c2);
    }

    Some(telemetry)
}

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

// n28 is a 28-bit integer, e.g. n28a or n28b, containing all the
// call sign bits from a packed message.
fn ft8_unpack_callsign(
    c28: u32, 
    ip: u8, 
    i3: u8
) -> Option<CallId> {
    dbg!(c28, ip, i3);
    let mut result = CallId::new();

    match c28 {
        C28_DE => {
            result.id.push_str("DE");
            result.special = true;
        },
        C28_QRZ => {
            result.id.push_str("QRZ");
            result.special = true;
        },
        C28_CQ => {
            result.id.push_str("CQ");
            result.special = true;
        },
        C28_CQ_DDD..C28_CQ_DDD_UNDEF => {
            // CQ_nnn with 3 digits
            result.id.push_str("CQ ");
            text::int_to_dd(&mut result.id, c28 as i32 - 3, false);
            result.special = true;
        },
        C28_CQ_DDD_UNDEF..C28_CQ_A => {
            dbg!("undefined cq_ddd");
        },
        C28_CQ_A..C28_CQ_A_UNDEF => {
            // CQ_aaaa with 4 alphanumeric symbols
            let mut n = c28 - (C28_CQ_A - 1); // - 1003;
            let mut aaaa = String::new();

            for _i in (0..1).rev() {
                aaaa.push(text::charn((n % 27) as u8, 4));
                n /= 27;
            }

            result.id.push_str("CQ ");
            result.id.push_str(aaaa.chars().rev().collect::<String>().trim());
            result.special = true;
        },
        C28_CQ_A_UNDEF..C28_CQ_AA => {
            dbg!("undefined cq_a");
        },
        C28_CQ_AA..C28_CQ_AA_UNDEF => {
            // CQ_aaaa with 4 alphanumeric symbols
            let mut n = c28 - (C28_CQ_AA - 1); // - 1003;
            let mut aaaa = String::new();

            for _i in (0..2).rev() {
                aaaa.push(text::charn((n % 27) as u8, 4));
                n /= 27;
            }

            result.id.push_str("CQ ");
            result.id.push_str(aaaa.chars().rev().collect::<String>().trim());
            result.special = true;

        },
        C28_CQ_AA_UNDEF..C28_CQ_AAA => {
            dbg!("undefined cq_aa");
        },

        C28_CQ_AAA..C28_CQ_AAA_UNDEF => {
            // CQ_aaaa with 4 alphanumeric symbols
            let mut n = c28 - (C28_CQ_AAA - 1); // - 1003;
            let mut aaaa = String::new();

            for _i in (0..3).rev() {
                aaaa.push(text::charn((n % 27) as u8, 4));
                n /= 27;
            }

            result.id.push_str("CQ ");
            result.id.push_str(aaaa.chars().rev().collect::<String>().trim());
            result.special = true;

        },

        C28_CQ_AAA_UNDEF..C28_CQ_AAAA => {
            dbg!("undefined cq_aaa");
        },

        C28_CQ_AAAA..C28_CQ_AAAA_UNDEF => {
            // CQ_aaaa with 4 alphanumeric symbols
            let mut n = c28 - (C28_CQ_AAAA - 1); // - 1003;
            let mut aaaa = String::new();

            for _i in (0..4).rev() {
                aaaa.push(text::charn((n % 27) as u8, 4));
                n /= 27;
            }

            result.id.push_str("CQ ");
            result.id.push_str(aaaa.chars().rev().collect::<String>().trim());
            result.special = true;

        },

        C28_CQ_AAAA_UNDEF..C28_HASH_CALL => {
            dbg!("undefined cq_a");
        },

        C28_HASH_CALL..C28_HASH_CALL_UNDEF => {
            dbg!("hashed call not yet supported");
            // This is a 22-bit hash of a result
            // TODO: implement
            // result.id.push_str("<...>");
            // todo!();
            // result[0] = '<';
            // int_to_dd(result + 1, n28, 7, false);
            // result[8] = '>';
            // result[9] = '\0';
            // return Some(result);
        },

        C28_STD_CALLS..=u32::MAX => {
            // Standard callsign
            let mut n = c28 - C28_STD_CALLS;

            let mut callsign = String::new();

            callsign.push(text::charn((n % 27) as u8, 4));
            n /= 27;
            callsign.push(text::charn((n % 27) as u8, 4));
            n /= 27;
            callsign.push(text::charn((n % 27) as u8, 4));
            n /= 27;
            callsign.push(text::charn((n % 10) as u8, 3));
            n /= 10;
            callsign.push(text::charn((n % 36) as u8, 2));
            n /= 36;
            callsign.push(text::charn((n % 37) as u8, 1));

            // Skip trailing and leading whitespace in case of a short callsign
            result.id.push_str(callsign.chars().rev().collect::<String>().trim());

            if !result.id.is_empty() {
                // Check if should append /R or /P suffix
                if ip != 0 {
                    if i3 == 1 {
                        result.id.push_str("/R");
                    } else if i3 == 2 {
                        result.id.push_str("/P");
                    }
                }
            }
        }
    }
    if result.id.is_empty() {
        None
    } else {
        Some(result)
    }
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


fn ft8_unpack_type1or2(
    a77: &[u8],
    i3: u8
) -> Option<Ft8Msg> {
    dbg!(a77, i3);

    let mut ft8_msg = Ft8Msg::new();
    
    // Extract packed fields
    let mut n28a = (a77[0] as u32) << 21;
    n28a |= (a77[1] as u32) << 13;
    n28a |= (a77[2] as u32) << 5;
    n28a |= (a77[3] as u32) >> 3;

    let mut n28b = ((a77[3] & 0x07) as u32) << 26;
    n28b |= (a77[4] as u32) << 18;
    n28b |= (a77[5] as u32) << 10;
    n28b |= (a77[6] as u32) << 2;
    n28b |= (a77[7] as u32) >> 6;

    let ir = (a77[7] & 0x20) as u16 >> 5;
    let mut igrid4 = ((a77[7] & 0x1F) as u16) << 10;
    igrid4 |= (a77[8] as u16) << 2;
    igrid4 |= (a77[9] as u16) >> 6;

    let call =  ft8_unpack_callsign(n28a >> 1, n28a as u8 & 0x01, i3);
    if call.is_some() {
        ft8_msg.call_to = call.unwrap();
    }

    let call = ft8_unpack_callsign(n28b >> 1, n28b as u8 & 0x01, i3);
    if call.is_some() {
        ft8_msg.call_from = call.unwrap();
    }

    match igrid4 {
        0..=MAXGRID4 => {
            // Extract 4 symbol grid locator
            if ir > 0 {
                // In case of ir=1 add an "R" before grid
                ft8_msg.extra.push_str("R ");
            }

            let mut n = igrid4;
            let mut dst = String::new();

            dst.push((b'0' + (n % 10) as u8) as char);
            n /= 10;
            dst.push((b'0' + (n % 10) as u8) as char);
            n /= 10;
            dst.push((b'A' + (n % 18) as u8) as char);
            n /= 18;
            dst.push((b'A' + (n % 18) as u8) as char);

            ft8_msg.extra.push_str(dst.chars().rev().collect::<String>().trim());
        },
        _ => {
            // Extract report
            let irpt = igrid4 - MAXGRID4;

            // Check special cases first (irpt > 0 always)
            match irpt {
                1 => ft8_msg.extra.push_str(""),
                2 => ft8_msg.extra.push_str("RRR"),
                3 => ft8_msg.extra.push_str("RR73"),
                4 => ft8_msg.extra.push_str("73"),
                _ => {
                    // Extract signal report as a two digit number with a + or - sign
                    if ir > 0 {
                        ft8_msg.extra.push('R')
                    }
                    text::int_to_dd(&mut ft8_msg.extra, irpt as i32 - 35, true);
                }
            }
        }
    }
    return Some(ft8_msg);
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


//-----------------------------------------------------------------------



//none standard for wsjt-x 2.0
//by KD8CEC
fn ft8_unpack_type4(
    a77: &[u8], 
) -> Option<Ft8Msg> {
    dbg!(a77);

    let mut ft8_msg = Ft8Msg::new();

    //let mut n12 = (a77[0] << 4) as u32; //11 ~4  : 8
    //n12 |= (a77[1] as u32) >> 4; //3~0 : 12

    let mut n58 = ((a77[1] & 0x0F) as u64) << 54; //57 ~ 54 : 4
    n58 |= (a77[2] as u64) << 46; //53 ~ 46 : 12
    n58 |= (a77[3] as u64) << 38; //45 ~ 38 : 12
    n58 |= (a77[4] as u64) << 30; //37 ~ 30 : 12
    n58 |= (a77[5] as u64) << 22; //29 ~ 22 : 12
    n58 |= (a77[6] as u64) << 14; //21 ~ 14 : 12
    n58 |= (a77[7] as u64) << 6; //13 ~ 6 : 12
    n58 |= (a77[8] as u64) >> 2; //5 ~ 0 : 765432 10

    let iflip = ((a77[8] as u32) >> 1) & 0x01; //76543210
    let mut nrpt = ((a77[8] as u32) & 0x01) << 1;
    nrpt |= (a77[9] as u32) >> 7; //76543210

    let icq = ((a77[9] as u32) >> 6) & 0x01;

    let mut c11 = String::new();

    for _i in (0..11).rev() {
        c11.push(text::charn((n58 % 38) as u8, 5));
        n58 /= 38;
    }

    let mut call_3 = String::new();
    // should replace with hash12(n12, call_3);
    call_3.push_str("<...>");
    // call_3[0] = '<';
    // int_to_dd(call_3 + 1, n12, 4, false);
    // call_3[5] = '>';
    // call_3[6] = '\0';
    let c11r = c11.chars().rev().collect::<String>();
    let (call_1, call_2) = if iflip != 0 {
        (c11r, call_3)
    } else {
        (call_3, c11r)
    };
    //save_hash_call(c11_trimmed);

    if icq == 0 {
        ft8_msg.call_to.id.push_str(call_1.as_str());
        if nrpt == 1 {
            ft8_msg.extra.push_str("RRR");
        } else if nrpt == 2 {
            ft8_msg.extra.push_str("RR73");
        } else if nrpt == 3 {
            ft8_msg.extra.push_str("73");
        }
    } else {
        ft8_msg.call_to.id.push_str("CQ");
    }

    ft8_msg.call_from.id.push_str(call_2.as_str());

    Some(ft8_msg)
}

pub fn ft8_unpack_buff_to_msg(a77: &[u8]) -> Option<Ft8Msg> {
    // assert_eq!(a77.len(), FT8.ldpc_k_bytes());
    // Extract i3 (bits 74..76)
    let i3 = (a77[9] >> 3) & 0x07;
    dbg!(i3);
    match i3 {
        0 => {
            // Extract n3 (bits 71..73)
            let n3 = ((a77[8] << 2) & 0x04) | ((a77[9] >> 6) & 0x03);
            dbg!(n3);
            match (n3) {
                0 => {
                    match ft8_unpack_type0_0(a77) {
                        Some(field) => {
                            let mut ret = Ft8Msg::new();
                            ret.extra = field;
                            return Some(ret);
                        },
                        None => {},
                    };
                },
                5 => {
                    match ft8_unpack_type0_5(a77) {
                        Some(field) => {
                            let mut ret = Ft8Msg::new();
                            ret.extra = field;
                            return Some(ret);
                        },
                        None => {},
                    };
                },
                _ => {
                    dbg!("unknown subtype", n3);
                }
            }
        },
        1..=2 => {
            // Type 1 (standard message) or Type 2 ("/P" form for EU VHF contest)
            match ft8_unpack_type1or2(a77, i3) {
                Some(fields) => {
                    return Some(fields);
                },
                None => {}
            }
        },
        3 => {
            dbg!("type 3 not yet supported");
        },
        4 => {
            //     // Type 4: Nonstandard calls, e.g. <WA9XYZ> PJ4/KA1ABC RR73
            //     // One hashed call or "CQ"; one compound or nonstandard call with up
            //     // to 11 characters; and (if not "CQ") an optional RRR, RR73, or 73.
            match ft8_unpack_type4(a77) {
                Some(fields) => {
                    return Some(fields);
                },
                None => {}
            }
        },
        5 => {
            dbg!("type 5 not yet supported");
        },
        _ => {
            dbg!("unknown type", i3);
        }
    }
    None // -1
}

pub fn ft8_unpack_to_string(a77: &[u8]) -> Option<String> {
    match ft8_unpack_buff_to_msg(a77) {
        Some(ft8_msg) => {
            dbg!(&ft8_msg);
            ft8_msg.to_string()
        },
        None => { None },
    }
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

    // #[test]
    // fn test_min1() {
    //     test_roundtrip("VK2TRF VK2ZTY QG61");
    //     test_roundtrip("CQ VK2ZTY QG61");
    // }
    
    // #[test]
    // fn test_min2() {
    //     test_roundtrip("CQ K1ABC QG61");
    // }

    #[test]
    fn test_0_0() {
        test_roundtrip("TNX BOB 73 GL"); // 0.0
    }

    // #[test]
    // fn test_0_1() {
    //     test_roundtrip("K1ABC RR73; W9XYZ <KH1/KH7Z> -08"); // 0.1
    // }

    // #[test]
    // fn test_0_3() {
    //     test_roundtrip("K1ABC W9XYZ 6A WI"); // 0.3 
    // }

    // #[test]
    // fn test_0_4() {
    //     test_roundtrip("W9XYZ K1ABC R 17B EMA"); // 0.4
    // }

    // #[test]
    // fn test_0_5() {
    //     test_roundtrip("123456789ABCDEF012"); // 0.5
    // }

    // #[test]
    // fn test_1() {
    //     test_roundtrip("K1ABC W9XYZ R EN37"); // 1.
    // }

    // #[test]
    // fn test_1_r () {
    //     test_roundtrip("K1ABC/R W9XYZ/R R EN37"); // 1.
    // }

    // #[test]
    // fn test_2() {
    //     test_roundtrip("G4ABC/P PA9XYZ JO22"); // 2.
    // }

    // #[test]
    // fn test_3() {
    //     test_roundtrip("K1ABC W9XYZ 579 WI"); // 3.
    // }

    // #[test]
    // fn test_4() {
    //     test_roundtrip("<W9XYZ> PJ4/K1ABC RRR"); // 4.
    // }

    // #[test]
    // fn test_5() {
    //     test_roundtrip("<G4ABC> <PA9XYZ> R 570007 JO22DB"); // 5. 
    // }
}