use thiserror::Error;

// use crate::rx_streamed::StreamReceiver; // - for library level errors

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum XxxError {
    #[error("TODO")]
    _ToDo,
    // #[error("Bad CRC: crc={0}")]
    // _BadCrc(u32),
    #[error("Bad CRC")]
    _BadCrc,
    #[error("Bad ECC")]
    _BadEcc,
    #[error("Bad Msg")]
    _BadMsg,
    #[error("Incomplete Data")]
    _DataIncomplete,
    #[error("Error Message: {0}")]
    _ErrorMessage(String),
    // Configuration(Box<dyn Error + Sync + Send>),
    // InvalidArgument(String),
    // Database(Box<dyn DatabaseError>),
    // Io(Error),
    // Tls(Box<dyn Error + Sync + Send>),
    // Protocol(String),
    #[error("Col not found {0}")]
    _ColNotFound(usize),
    #[error("Row not found {0}")]
    _RowNotFound(usize),
    #[error("Index too high {0}")]
    _IndexTooHigh(usize),
    #[error("Index too low {0}")]
    _IndexTooLow(usize),
    // TypeNotFound {
    //     type_name: String,
    // },
    // ColumnIndexOutOfBounds {
    //     index: usize,
    //     len: usize,
    // },
    // ColumnNotFound(String),
    // ColumnDecode {
    //     index: String,
    //     source: Box<dyn Error + Sync + Send>,
    // },
    // Encode(Box<dyn Error + Sync + Send>),
    // Decode(Box<dyn Error + Sync + Send>),
    // AnyDriverError(Box<dyn Error + Sync + Send>),
    // PoolTimedOut,
    // PoolClosed,
    // WorkerCrashed,
    // Migrate(Box<MigrateError>),
    // InvalidSavePointStatement,
    // BeginFailed,
}
