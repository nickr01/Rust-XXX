pub mod cpal_helper;
pub mod debug;
pub mod error;
pub mod types;

#[cfg(any(feature = "enable_rx", test))]
pub mod rx_pipeline;
#[cfg(any(feature = "enable_rx", test))]
mod receiver;

#[cfg(any(feature = "enable_tx", test))]
pub mod tx_pipeline;

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