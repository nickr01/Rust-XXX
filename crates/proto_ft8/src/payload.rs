// use rustfft::num_traits::clamp;

// Pack and Unpack with unit tests
use crate::ft8::*;
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

struct CharSet {
    msg_len: usize,
    pad: char,
    set: &'static str,
}

impl CharSet {
    pub fn modulus(&self) -> usize {
        self.set.len()
    }
}

const FREE_CHARSET: CharSet = CharSet {
    msg_len: 13,
    pad: ' ',
    set: " 0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ+-./?",
};

const TELEM_CHARSET: CharSet = CharSet {
    msg_len: 18,
    pad: '0',
    set: "0123456789ABCDEF",
};

// These are not elegant
const CALL1_CHARSET: CharSet = CharSet {
    msg_len: 6,
    pad: ' ',
    set: " 0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ"
};

const CALL2_CHARSET: CharSet = CharSet {
    msg_len: 6,
    pad: ' ',
    set: "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ"
};

const CALL3_CHARSET: CharSet = CharSet {
    msg_len: 6,
    pad: ' ',
    set: "0123456789"
};

const CALL4_CHARSET: CharSet = CharSet {
    msg_len: 6,
    pad: ' ',
    set: " ABCDEFGHIJKLMNOPQRSTUVWXYZ"
};

// - Leading and trailing whitespace removed
// - Trim to len
// - Right justify with pad_chars
// maybe need to get rid of these messy str<>String conversions
fn left_pad(input_string: &str, charset: &CharSet) -> String {
    let mut trimmed = input_string.trim().to_string();
    trimmed.truncate(charset.msg_len);
    let mut trimmed = trimmed.trim().to_string(); // yep, trim again
    {
        let left_pad_len: isize = charset.msg_len as isize - trimmed.len() as isize ;
        if left_pad_len > 0 {
            let pad_string = String::from(charset.pad); 
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
// - Assume can build the output bitmap in UWork - might as well pass that around
fn ft8_pack_0_stg (
    cn: &str,
    charset: &CharSet,
    bits: usize
) -> Option<U71> {
    let cn = left_pad(&cn, charset);
    let mut val: U71 = U71(0);
    for c in cn.chars() {
        match charset.set.find(c).map_or(None, |i| Some(i as u8)) {
            Some(j) => {
                val.0 = val.0 * U71(charset.modulus() as u128).0 + U71(j as u128).0;
            },
            None => {
                dbg!("invalid character in 0_0:free text");
                return None;
            }
        }
    }
    assert!(bits < 128);
    let val_max = 2u128.pow(bits as u32);
    // dbg!(val);
    if val.0 < val_max {
        Some(val)
    } else {
        dbg!("packed data out of range");
        None
    }
}

fn ft8_pack_0_0 (
    c13: &str,
) -> Option<U71> {
    ft8_pack_0_stg(c13, &FREE_CHARSET, FT8_MESSAGE_BITS)
}

fn ft8_pack_0_5 (
    c18: &str,
) -> Option<U71> {
    ft8_pack_0_stg(c18, &TELEM_CHARSET, FT8_MESSAGE_BITS)
}

//---------------------------------------------------------

fn ft8_unpack_0_stg(
    a71: U71,
    bits: usize,
    charset: &CharSet,
) -> Option<String> {
    // dbg!(a71);
    assert!(bits < 128);
    let val_max = 2u128.pow(bits as u32);
    let mut a71 = a71.0 & val_max - 1; // sanitise

    let mut text = String::new();
    
    while text.len() < charset.msg_len {
        let n = a71 % U71(charset.modulus() as u128).0;
        a71 /= U71(charset.modulus() as u128).0;
        let oc = charset.set.chars().nth(n as usize);
        match oc {
            Some(c) => {
                text.push(c);
            },
            None => {
                dbg!("algorithm error");
                return None;
            }
        }
    }
    let text: String = text.chars().rev().collect();
    // not sure whether useful to trim here - probably
    Some(text.trim().to_string())
}

fn ft8_unpack_0_0(
    a71: U71,
) -> Option<String> {
    ft8_unpack_0_stg(a71, FT8_MESSAGE_BITS, &FREE_CHARSET)
}

fn ft8_unpack_0_5(
    a71: U71,
) -> Option<String> {
    ft8_unpack_0_stg(a71, FT8_MESSAGE_BITS, &TELEM_CHARSET)
}


/// std_call_to_c28.f90 does not tell the whole story with call packing
/// Note that 4 letter calls would be failing this algorithm
/// pack28 from wsjtx fortran prepares to find the digits and match properly
fn ft8_try_pack_stdcall(c28: &str) -> Option<U28> {
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

    // dbg!(&c28);

    let char6: Vec<char> = c28.chars().collect();
    assert_eq!(char6.len(), 6);
    let mut i = [U28(0); 6]; 

    // NBNB the behaviour with unexpected characters differs to the
    // behaviour of reference packer std_call_to_c28.f90
    // which does not fail on chars in unexpected positions
    // can't let through as round trip would fail anyway
    i[0] = match CALL1_CHARSET.set.find(char6[0]) {
        Some(n) => {
            U28(n as u32)
        },
        None => {
            return None;
        }
    };
    i[1] = match CALL2_CHARSET.set.find(char6[1]) {
        Some(n) => {
            U28(n as u32)
        },
        None => { 
            return None;
        }
    };
    i[2] = match CALL3_CHARSET.set.find(char6[2]) {
        Some(n) => {
            U28(n as u32)
        },
        None => { 
            return None;
         }
    };
    i[3] = match CALL4_CHARSET.set.find(char6[3]) {
        Some(n) => {
            U28(n as u32)
        },
        None => {
            return None;
        }
    };
    i[4] = match CALL4_CHARSET.set.find(char6[4]) {
        Some(n) => {
            U28(n as u32)
        },
        None => { 
            return None;
        }
    };
    i[5] = match CALL4_CHARSET.set.find(char6[5]) {
        Some(n) => {
            U28(n as u32)
        },
        None => { 
            return None;
        }
    };

    let n28: U28 = U28(
        C28_STD_CALLS
        + 36 * 10 * 27 * 27 * 27 * i[0].0
        + 10 * 27 * 27 * 27 * i[1].0
        + 27 * 27 * 27 * i[2].0
        + 27 * 27 * i[3].0
        + 27 * i[4].0
        + i[5].0
    );

    // dbg!(n28);

    Some(n28)
}

fn ft8_pack_stdcall(c28: &str) -> Option<U28> {
    let c28 = c28.trim();
    
    let trimmed_length = c28.len();
    if trimmed_length < 3 {
        return None;
    }

    // try 0 leading space
    let mut s = format!("{}    ", c28);
    s.truncate(6);
    let result = ft8_try_pack_stdcall(&s);
    if result.is_some() {
        return result;
    }

    // try 1 leading space
    let mut s = format!(" {}   ", c28);
    s.truncate(6);
    let result = ft8_try_pack_stdcall(&s);
    if result.is_some() {
        return result;
    }

    // try 2 leading spaces
    let mut s = format!("  {}   ", c28);
    s.truncate(6);
    let result = ft8_try_pack_stdcall(&s);
    if result.is_some() {
        return result;
    }

    None
}

fn ft8_pack_h22call(c28: &str) -> Option<U28> {
    todo!("Pack h22 call")
}

// Pack a special token, a 22-bit hash code call, or a valid base call
// into a 28-bit integer.
fn ft8_pack_c28(c28: &str) -> Option<U28> {
    dbg!(c28);
    match &c28[0..3] {
        "DE " => {
            Some(U28(C28_DE))
        },
        "QRZ" => {
            Some(U28(C28_QRZ))
        },
        "CQ " => {
            Some(U28(C28_CQ))
        },
        "CQ_" => {
            //int nnum = 0, nlet = 0;
            todo!("Support CQ_ ins c28");
        },
        "<.." => {
            ft8_pack_h22call(c28)
        },
        _ => {
            ft8_pack_stdcall(c28)
        }
    }
}

// fn ft8_pack_c28r1(c28r1: &str) -> Option<U28, bool> {
//     Some(None, false)
// }

fn ft8_unpack_stdcall(
    c28: U28,
    ip: bool, 
    i3: u8
) -> Option<String> {
    // Standard callsign
    let mut n: U28 = U28(c28.0 - C28_STD_CALLS);

    let mut callsign = String::new();
    callsign.push(text::charn((n.0 % 27) as u8, 4));
    n.0 /= 27;
    callsign.push(text::charn((n.0 % 27) as u8, 4));
    n.0 /= 27;
    callsign.push(text::charn((n.0 % 27) as u8, 4));
    n.0 /= 27;
    callsign.push(text::charn((n.0 % 10) as u8, 3));
    n.0 /= 10;
    callsign.push(text::charn((n.0 % 36) as u8, 2));
    n.0 /= 36;
    callsign.push(text::charn((n.0 % 37) as u8, 1));

    // Skip trailing and leading whitespace in case of a short callsign
    let mut result = String::new();
    result.push_str(callsign.chars().rev().collect::<String>().trim());

    if !result.is_empty() {
        // Check if should append /R or /P suffix
        if ip {
            if i3 == 1 {
                result.push_str("/R");
            } else if i3 == 2 {
                result.push_str("/P");
            }
        }
    }
    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

// n28 is a 28-bit integer, e.g. n28a or n28b, containing all the
// call sign bits from a packed message.
fn ft8_unpack_c28(
    c28: U28, 
    ip: bool, 
    i3: u8
) -> Option<CallId> {
    dbg!(c28, ip, i3);
    let mut result = CallId::new();

    match c28.0 {
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
            text::int_to_dd(&mut result.id, c28.0 as i32 - 3, false);
            result.special = true;
        },
        C28_CQ_DDD_UNDEF..C28_CQ_A => {
            dbg!("undefined cq_ddd");
        },
        C28_CQ_A..C28_CQ_A_UNDEF => {
            // CQ_aaaa with 4 alphanumeric symbols
            let mut n = c28.0 - (C28_CQ_A - 1); // - 1003;
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
            let mut n = c28.0 - (C28_CQ_AA - 1); // - 1003;
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
            let mut n = c28.0 - (C28_CQ_AAA - 1); // - 1003;
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
            let mut n = c28.0 - (C28_CQ_AAAA - 1); // - 1003;
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
            dbg!("undefined cq_aaaa");
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

        C28_STD_CALLS..=U28_MAX => {
            result.id = ft8_unpack_stdcall(c28, ip, i3)?;

        },

        _ => {
            dbg!("undefined value above max for bit width");
        }
    }
    if result.id.is_empty() {
        None
    } else {
        Some(result)
    }
}

fn ft8_pack_grid4(grid4: &str) -> Option<U15> {
    assert_ne!(grid4, "RR73");
    let gstr: Vec<char> = grid4.chars().collect();
    if 
        text::in_range_inclusive(gstr[0], 'A', 'R')
        && text::in_range_inclusive(gstr[1], 'A', 'R')
        && gstr[2].is_ascii_digit() 
        && gstr[3].is_ascii_digit()
    {
        let mut igrid4: u16 = gstr[0] as u16 - 'A' as u16;
        igrid4 = igrid4 * 18 + (gstr[1] as u16 - 'A' as u16);
        igrid4 = igrid4 * 10 + (gstr[2] as u16 - '0' as u16);
        igrid4 = igrid4 * 10 + (gstr[3] as u16 - '0' as u16);
        return Some(U15(igrid4));
    }
    None
}

fn ft8_pack_report(g15: &str) -> Option<U15> {
    // Parse report: +dd /-dd /R+dd /R-dd
    todo!("check the range of dd");
    let gstr: Vec<char> = g15.chars().collect();
    if gstr[0] == 'R' {
        todo!("this is in the wrong place - R detection should be higher in parse");
        let dd = text::_dd_to_int(&g15.chars().take(1).collect::<String>());
        let irpt = (35 + dd) as u16;
        return Some(U15((MAXGRID4 + irpt) | 0x8000)); // ir = 1
    } else {
        let dd = text::_dd_to_int(g15);
        let irpt = (35 + dd) as u16;
        return Some(U15(MAXGRID4 + irpt)); // ir = 0
    }
}

fn ft8_pack_g15(g15: &str) -> Option<U15> {
    dbg!(g15);
    let g15 = g15.trim();

    match g15 {
        "" => {
            return Some(U15(MAXGRID4 + 1)); // !!!!!
        }
        "RRR" => {
            return Some(U15(MAXGRID4 + 2));
        },
        "RR73" => {
            return Some(U15(MAXGRID4 + 3));
        },
        "73" => {
            return Some(U15(MAXGRID4 + 4));
        },
        _ => {
            match ft8_pack_grid4(g15) {
                Some(result) => {
                    return Some(result);
                },
                None => {
                    return ft8_pack_report(g15);
                }
            }
        }
    }
}

// Pack Type 1 (Standard 77-bit message) and Type 2 (ditto, with a "/P" call)
fn ft8_pack_type1(type1_msg: &str) -> Option<Vec<u8>> {
    dbg!(type1_msg);

    // Locate the first delimiter
    // c28[/r1] c28[/r1] [R1 ]g15
    let token: Vec<&str> = type1_msg.split(' ').collect();

    let n28a = ft8_pack_stdcall(token[0]);
    if n28a.is_none() {
        return None;
    }

    let n28b = ft8_pack_stdcall(token[1]);
    if n28b.is_none() {
        return None;
    }

    let n28a = n28a.unwrap();
    let n28b = n28b.unwrap();

    let igrid4 =  ft8_pack_grid4(if token.len() > 2 { token[2] } else { " " })
        .expect("expected a packed grid4");

    let i3 = 1u8; // No suffix or /R

    // TODO: check for suffixes

    // Shift in ipa and ipb bits into n28a and n28b
    let n28a = (n28a.0 as u32) << 1; // ipa = 0
    let n28b = (n28b.0 as u32) << 1; // ipb = 0

    // Pack into (28 + 1) + (28 + 1) + (1 + 15) + 3 bits
    let mut b77 = Vec::with_capacity(FT8._ldpc_k_bytes().0);
    b77.push((n28a >> 21) as u8);
    b77.push((n28a >> 13) as u8);
    b77.push((n28a >> 5) as u8);
    b77.push((n28a << 3) as u8 | (n28b >> 26) as u8);
    b77.push((n28b >> 18) as u8);
    b77.push((n28b >> 10) as u8);
    b77.push((n28b >> 2) as u8);
    b77.push((n28b << 6) as u8 | (igrid4.0 >> 10) as u8);
    b77.push((igrid4.0 >> 2) as u8);
    b77.push((igrid4.0 << 6) as u8 | (i3 << 3));
    
    Some(b77)
}


// fn ft8_unpack_type1or2(
//     a77: &[u8],
//     i3: u8
// ) -> Option<Ft8Msg> {
//     dbg!(a77, i3);

//     let mut ft8_msg = Ft8Msg::new();
    
//     // Extract packed fields
//     let mut n28a = (a77[0] as u32) << 21;
//     n28a |= (a77[1] as u32) << 13;
//     n28a |= (a77[2] as u32) << 5;
//     n28a |= (a77[3] as u32) >> 3;

//     let mut n28b = ((a77[3] & 0x07) as u32) << 26;
//     n28b |= (a77[4] as u32) << 18;
//     n28b |= (a77[5] as u32) << 10;
//     n28b |= (a77[6] as u32) << 2;
//     n28b |= (a77[7] as u32) >> 6;

//     let ir = (a77[7] & 0x20) as u16 >> 5;
//     let mut igrid4 = ((a77[7] & 0x1F) as u16) << 10;
//     igrid4 |= (a77[8] as u16) << 2;
//     igrid4 |= (a77[9] as u16) >> 6;

//     let call =  ft8_unpack_callsign(n28a >> 1, n28a as u8 & 0x01, i3);
//     if call.is_some() {
//         ft8_msg.call_to = call.unwrap();
//     }

//     let call = ft8_unpack_callsign(n28b >> 1, n28b as u8 & 0x01, i3);
//     if call.is_some() {
//         ft8_msg.call_from = call.unwrap();
//     }

//     match igrid4 {
//         0..=MAXGRID4 => {
//             // Extract 4 symbol grid locator
//             if ir > 0 {
//                 // In case of ir=1 add an "R" before grid
//                 ft8_msg.extra.push_str("R ");
//             }

//             let mut n = igrid4;
//             let mut dst = String::new();

//             dst.push((b'0' + (n % 10) as u8) as char);
//             n /= 10;
//             dst.push((b'0' + (n % 10) as u8) as char);
//             n /= 10;
//             dst.push((b'A' + (n % 18) as u8) as char);
//             n /= 18;
//             dst.push((b'A' + (n % 18) as u8) as char);

//             ft8_msg.extra.push_str(dst.chars().rev().collect::<String>().trim());
//         },
//         _ => {
//             // Extract report
//             let irpt = igrid4 - MAXGRID4;

//             // Check special cases first (irpt > 0 always)
//             match irpt {
//                 1 => ft8_msg.extra.push_str(""),
//                 2 => ft8_msg.extra.push_str("RRR"),
//                 3 => ft8_msg.extra.push_str("RR73"),
//                 4 => ft8_msg.extra.push_str("73"),
//                 _ => {
//                     // Extract signal report as a two digit number with a + or - sign
//                     if ir > 0 {
//                         ft8_msg.extra.push('R')
//                     }
//                     text::int_to_dd(&mut ft8_msg.extra, irpt as i32 - 35, true);
//                 }
//             }
//         }
//     }
//     return Some(ft8_msg);
// }

// fn ft8_pack_type3(type3_msg: &str) -> Option<Vec<u8>> {
//     dbg!(type3_msg);

//     let text = type3_msg.trim();

//     let mut b77 = Vec::with_capacity(FT8._ldpc_k_bytes().0);
//     // Clear the first 72 bits representing a long number
//     b77.resize(FT8._ldpc_k_bytes().0, 0);
//     // for i in 0..9 {
//     //     b77[i] = 0;
//     // }

//     // Now express the text as base-42 number stored
//     // in the first 72 bits of b77
//     for j in 0..13 {
//         // Multiply the long integer in b77 by 42
//         let mut x = 0u16;
//         for i in (0..8).rev() {
//             x += b77[i] as u16 * 42u16;
//             b77[i] = (x & 0xFF) as u8;
//             x >>= 8;
//         }

//         // Get the index of the current char
//         if j < text.len() {
//             if let Some(c) = text.chars().nth(j) {
//                 if let Some(q) = CALL_A0.find(c) {
//                     x = if q > 0 { q as u16 } else { 0 };
//                 } else {
//                     x = 0;
//                 }
//             } else {
//                 x = 0;
//             }
//         } else {
//             x = 0;
//         }
//         // Here we double each added number in order to have the result multiplied
//         // by two as well, so that it's a 71 bit number left-aligned in 72 bits (9 bytes)
//         x <<= 1;

//         // Now add the number to our long number
//         for i in (0..8).rev() {
//             if x == 0 {
//                 break;
//             }

//             x += b77[i] as u16;
//             b77[i] = (x & 0xFF) as u8;
//             x >>= 8;
//         }
//     }
//     // Set n3=0 (bits 71..73) and i3=0 (bits 74..76)
//     b77[8] &= 0xFE;
//     b77[9] &= 0x00;

//     Some(b77)
// }

// //-----------------------------------------------------------------------

// //none standard for wsjt-x 2.0
// //by KD8CEC
// fn ft8_unpack_type4(
//     a77: &[u8], 
// ) -> Option<Ft8Msg> {
//     dbg!(a77);

//     let mut ft8_msg = Ft8Msg::new();

//     //let mut n12 = (a77[0] << 4) as u32; //11 ~4  : 8
//     //n12 |= (a77[1] as u32) >> 4; //3~0 : 12

//     let mut n58 = ((a77[1] & 0x0F) as u64) << 54; //57 ~ 54 : 4
//     n58 |= (a77[2] as u64) << 46; //53 ~ 46 : 12
//     n58 |= (a77[3] as u64) << 38; //45 ~ 38 : 12
//     n58 |= (a77[4] as u64) << 30; //37 ~ 30 : 12
//     n58 |= (a77[5] as u64) << 22; //29 ~ 22 : 12
//     n58 |= (a77[6] as u64) << 14; //21 ~ 14 : 12
//     n58 |= (a77[7] as u64) << 6; //13 ~ 6 : 12
//     n58 |= (a77[8] as u64) >> 2; //5 ~ 0 : 765432 10

//     let iflip = ((a77[8] as u32) >> 1) & 0x01; //76543210
//     let mut nrpt = ((a77[8] as u32) & 0x01) << 1;
//     nrpt |= (a77[9] as u32) >> 7; //76543210

//     let icq = ((a77[9] as u32) >> 6) & 0x01;

//     let mut c11 = String::new();

//     for _i in (0..11).rev() {
//         c11.push(text::charn((n58 % 38) as u8, 5));
//         n58 /= 38;
//     }

//     let mut call_3 = String::new();
//     // should replace with hash12(n12, call_3);
//     call_3.push_str("<...>");
//     // call_3[0] = '<';
//     // int_to_dd(call_3 + 1, n12, 4, false);
//     // call_3[5] = '>';
//     // call_3[6] = '\0';
//     let c11r = c11.chars().rev().collect::<String>();
//     let (call_1, call_2) = if iflip != 0 {
//         (c11r, call_3)
//     } else {
//         (call_3, c11r)
//     };
//     //save_hash_call(c11_trimmed);

//     if icq == 0 {
//         ft8_msg.call_to.id.push_str(call_1.as_str());
//         if nrpt == 1 {
//             ft8_msg.extra.push_str("RRR");
//         } else if nrpt == 2 {
//             ft8_msg.extra.push_str("RR73");
//         } else if nrpt == 3 {
//             ft8_msg.extra.push_str("73");
//         }
//     } else {
//         ft8_msg.call_to.id.push_str("CQ");
//     }

//     ft8_msg.call_from.id.push_str(call_2.as_str());

//     Some(ft8_msg)
// }

// // pub fn ft8_unpack_buff_to_msg(a77: &[u8]) -> Option<Ft8Msg> {
// //     // assert_eq!(a77.len(), FT8.ldpc_k_bytes());
// //     // Extract i3 (bits 74..76)
// //     let i3 = (a77[9] >> 3) & 0x07;
// //     dbg!(i3);
// //     match i3 {
// //         0 => {
// //             // Extract n3 (bits 71..73)
// //             let n3 = ((a77[8] << 2) & 0x04) | ((a77[9] >> 6) & 0x03);
// //             dbg!(n3);
// //             match (n3) {
// //                 0 => {
// //                     match ft8_unpack_type0_0(a77) {
// //                         Some(field) => {
// //                             let mut ret = Ft8Msg::new();
// //                             ret.extra = field;
// //                             return Some(ret);
// //                         },
// //                         None => {},
// //                     };
// //                 },
// //                 5 => {
// //                     match ft8_unpack_type0_5(a77) {
// //                         Some(field) => {
// //                             let mut ret = Ft8Msg::new();
// //                             ret.extra = field;
// //                             return Some(ret);
// //                         },
// //                         None => {},
// //                     };
// //                 },
// //                 _ => {
// //                     dbg!("unknown subtype", n3);
// //                 }
// //             }
// //         },
// //         1..=2 => {
// //             // Type 1 (standard message) or Type 2 ("/P" form for EU VHF contest)
// //             match ft8_unpack_type1or2(a77, i3) {
// //                 Some(fields) => {
// //                     return Some(fields);
// //                 },
// //                 None => {}
// //             }
// //         },
// //         3 => {
// //             dbg!("type 3 not yet supported");
// //         },
// //         4 => {
// //             //     // Type 4: Nonstandard calls, e.g. <WA9XYZ> PJ4/KA1ABC RR73
// //             //     // One hashed call or "CQ"; one compound or nonstandard call with up
// //             //     // to 11 characters; and (if not "CQ") an optional RRR, RR73, or 73.
// //             match ft8_unpack_type4(a77) {
// //                 Some(fields) => {
// //                     return Some(fields);
// //                 },
// //                 None => {}
// //             }
// //         },
// //         5 => {
// //             dbg!("type 5 not yet supported");
// //         },
// //         _ => {
// //             dbg!("unknown type", i3);
// //         }
// //     }
// //     None // -1
// // }

// // pub fn ft8_unpack_to_string(a77: &[u8]) -> Option<String> {
// //     match ft8_unpack_buff_to_msg(a77) {
// //         Some(ft8_msg) => {
// //             dbg!(&ft8_msg);
// //             ft8_msg
// //         },
// //         None => { None },
// //     }
// // }

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::{unpack_ft8::ft8_unpack_to_string};

    #[test]
    fn test_left_pad() {
        assert_eq!(left_pad(&"fred", &FREE_CHARSET), "         fred");
        assert_eq!(left_pad(&"fred         ", &FREE_CHARSET), "         fred");
        assert_eq!(left_pad(&"fred         and more", &FREE_CHARSET), "         fred");
        assert_eq!(left_pad(&"   fred             and much more", &FREE_CHARSET), "         fred");
    }

    #[test]
    fn test_pack_0_0() {
        assert_eq!(ft8_pack_0_0(&"0").unwrap(), U71(1));
        assert_eq!(ft8_pack_0_0(&"1").unwrap(), U71(2));
        assert_eq!(ft8_pack_0_0(&"00").unwrap(), U71(1 + FREE_CHARSET.modulus() as UType71));
        assert_eq!(ft8_pack_0_0(&"01").unwrap(), U71(0b00101100));
        assert_eq!(
            ft8_pack_0_0(&"TNX BOB 73 GL").unwrap(),
            U71(0b01100011111011011100111011100010101001001010111000000111111101010000000)
        );
    }

    #[test]
    fn test_unpack_0_0() {
        assert_eq!(ft8_unpack_0_0(U71(0)).unwrap(), "");     
        assert_eq!(ft8_unpack_0_0(U71(1)).unwrap(), "0");
        assert_eq!(
            ft8_unpack_0_0(U71(0b01100011111011011100111011100010101001001010111000000111111101010000000)).unwrap(), 
            "TNX BOB 73 GL");
    }

    fn test_round_0_0(s: &str) {
        assert_eq!(ft8_unpack_0_0(ft8_pack_0_0(s).unwrap()).unwrap(), left_pad(s, &FREE_CHARSET).trim());
    }

    #[test]
    fn test_0_0() {
        test_round_0_0(&"");
        test_round_0_0(&"TNX BOB 73 GL");
        test_round_0_0(&"1");
        test_round_0_0(&"FRED 1 2 3");
    }

    #[test]
    fn test_pack_0_5() {
        assert_eq!(ft8_pack_0_5(&"").unwrap(), U71(0));
        assert_eq!(ft8_pack_0_5(&"0").unwrap(), U71(0));
        assert_eq!(ft8_pack_0_5(&"1").unwrap(), U71(1));
        assert_eq!(ft8_pack_0_5(&"12").unwrap(), U71(0x12));
        assert_eq!(ft8_pack_0_5(&"123456781234567800").unwrap(), U71(0x123456781234567800));
    }

    #[test]
    fn test_unpack_0_5() {
        assert_eq!(ft8_unpack_0_5(U71(0x123456781234567800)).unwrap(), "123456781234567800");
    }

    fn test_round_0_5(s: &str) {
        assert_eq!(ft8_unpack_0_5(ft8_pack_0_5(s).unwrap()).unwrap(), left_pad(s, &TELEM_CHARSET));
    }

    #[test]
    fn test_0_5() {
        test_round_0_5(&"0");
        // test_round_0_5(&"1");
        // test_round_0_5(&"12");
        // test_round_0_5(&"123456789ABCDEF");
    }

    // fn test_0_5_bad_char() {
    //     test_round_0_5(&"123456789ABCDEF");
    // }

    #[test]
    fn test_pack_stdcall() {
        assert_eq!(ft8_pack_stdcall(&"VK2ZTY").unwrap(), U28(237001541));
        assert_eq!(ft8_pack_stdcall(&"VK2EA").unwrap(), U28(236985694));
        assert_eq!(ft8_pack_stdcall(&"K1JT").unwrap(), U28(10222009));
        assert_eq!(ft8_pack_stdcall(&"E2A").unwrap(), U28(9053611));
        assert_eq!(ft8_pack_stdcall(&"3D0XYZ").unwrap(), U28(37178403));
        assert_eq!(ft8_pack_stdcall(&"3D0").unwrap(), U28(37160206));
        assert_eq!(ft8_pack_stdcall(&"A33A").unwrap(), U28(84852844));
    }

    #[test]
    fn test_unpack_stdcall() {
        assert_eq!(ft8_unpack_stdcall(U28(237001541), false, 0).unwrap(), "VK2ZTY");
        assert_eq!(ft8_unpack_stdcall(U28(236985694), false, 0).unwrap(), "VK2EA");
        assert_eq!(ft8_unpack_stdcall(U28(10222009), false, 0).unwrap(), "K1JT");
        assert_eq!(ft8_unpack_stdcall(U28(9053611), false, 0).unwrap(), "E2A");
        assert_eq!(ft8_unpack_stdcall(U28(37178403), false, 0).unwrap(), "3D0XYZ");
        assert_eq!(ft8_unpack_stdcall(U28(37160206), false, 0).unwrap(), "3D0");
        assert_eq!(ft8_unpack_stdcall(U28(84852844), false, 0).unwrap(), "A33A");
    }

    fn test_round_stdcall(s: &str) {
        assert_eq!(ft8_unpack_stdcall(ft8_pack_stdcall(s).unwrap(), false, 0).unwrap(), s);
    }

    #[test]
    fn test_stdcall() {
        test_round_stdcall("VK2ZTY");
        test_round_stdcall("VK2EA");
        test_round_stdcall("K1JT");
        test_round_stdcall("E2A");
        test_round_stdcall("3D0XYZ");
        test_round_stdcall("3D0");
        test_round_stdcall("A33A");
    }
    // fn test_pack_0_5(msg: &str) {        
    // }

    // fn test_unpack_0_5(p: UWork) {        
    // }

    // fn test_round_0_5(msg: &str) {

    // }
    
    // #[test]
    // fn test_min1() {
    //     test_roundtrip("VK2TRF VK2ZTY QG61");
    //     test_roundtrip("CQ VK2ZTY QG61");
    // }
    
    // #[test]
    // fn test_min2() {
    //     test_roundtrip("CQ K1ABC QG61");
    // }

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