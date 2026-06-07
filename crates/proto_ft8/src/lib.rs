pub mod protocol;
pub mod generator;
pub mod parity;
// pub mod pack_ft8;
// pub mod unpack_ft8;
mod text;
mod payload;

// mod oldconst;
// mod oldtext;
// mod oldpack;
// mod oldunpack;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}