pub mod rustxxx;
pub mod pipeline;
mod receiver;
mod detector;
mod waterfall;
mod debug;
mod correlator;
mod decoder;
mod subtractor;
mod candidate;

mod test_generator;
mod test_parity;

mod layer0; // Audio
mod layer1; // sync
mod layer2; // gray
mod layer3; // Ecc
mod layer4; // Crc
mod layer5; // Top - UCP-API or FT8 app layer connect here

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}