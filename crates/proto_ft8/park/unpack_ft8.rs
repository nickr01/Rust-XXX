use crate::text;
use crate::protocol::*;

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

fn ft8_unpack_type0_0(
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
    use crate::unpack_ft8;

    // fn test_roundtrip(modem: &mut rustxxx::Modem, s: &str) {
    //     for loopback_at in 0..6 {
    //         let cw = pack_ft8::_pack77(s);
    //         assert_eq!(cw.len(), modem.protocol._ldpc_p_bytes().0);
    //         let loopback_result = modem.l5_top_outbound(loopback_at, &cw).unwrap();
    //         assert_eq!(loopback_result, cw, "failed at loopback {}", loopback_at);
    //     }
    // }

    #[test]
    fn test() {
        
        // let mut modem: rustxxx::Modem = rustxxx::Modem::new(
        //     &rustxxx::TEST_PROTOCOL, 
        //     &rustxxx::TEST_FT8_RUNTIME, 
        //     rustxxx::TEST_FREQUENCY
        // );

        // const M_0: &str = "CQ VK2ZTY QG61";        
        // test_roundtrip(&mut modem, M_0);
    }

}