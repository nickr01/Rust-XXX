pub mod cpal_helper;
pub mod debug;
pub mod error;
pub mod pipeline;
pub mod types;

mod receiver;
mod detector;
mod waterfall;
mod correlator;
mod decoder;
mod subtractor;
mod candidate;

mod test_generator;
mod test_parity;
mod test_support;

mod l0_audio; // Audio
mod l1_sync; // sync
mod l2_gray_code; // gray
mod l3_ecc; // Ecc
mod l4_crc; // Crc
mod l5_top; // Top - UCP-API or FT8 app layer connect here

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}