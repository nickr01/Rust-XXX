use crate::error;
use crate::types;
use crate::types::CodeWord;

pub struct TxPipeLine {
    // if keeps any state then would need to structure like the receiver 
    modem: types::Modem,
}

impl TxPipeLine {
    pub fn new (
        protocol: &'static types::Protocol,
        runtime: &'static types::Runtime,

    ) -> TxPipeLine {
        TxPipeLine { 
            modem: types::Modem::new(protocol, runtime),
        }
    }

    fn modulate(&self, cw: &[u8], freq_hz: types::Hz) -> Result<Vec<f32>, error::XxxError> {
        self.modem.modulate(cw, freq_hz)
    }
}