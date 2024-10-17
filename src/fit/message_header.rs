//! FIT 1 byte message header.
//!
//! Layout, normal header:
//! ```
//! Bit idx    7 6 5 4 3 2 1 0
//! ```
//! Bit idx    7 6 5 4 3 2 1 0
//! Header   | 0 x x x x x x x |
//!            | | | | └┬────┘
//!            | | | |  |
//!            | | | |  ╰- Local ID, 0-15
//!            | | | |     maps a data messge to the corresponding definition
//!            | | | ╰- Reserved (?)
//!            | | ╰- 1 = contains custom developer field definitions
//!            | ╰--- 0 = Data, 1 = Definition
//!            ╰----- 0 = Normal header, 1 = Compressed time stamp header
//! ```
//!
//! Layout, compressed timestamp header:
//! ```
//! Bit idx    7 6 5 4 3 2 1 0
//! Header   | 1 x x x x x x x |
//!            | | | | └┬────┘
//!            | | | |  |
//!            | | | |  ╰- Local ID, 0-15
//!            | | | |     maps a data messge to the corresponding definition
//!            | | | ╰- Reserved (?)
//!            | | ╰- 1 = contains custom developer field definitions
//!            | ╰--- 0 = Data, 1 = Definition
//!            ╰----- 1 = Compressed time stamp header,
//!                       changes the rest of the header layout if set.
//! ```

use binrw::BinRead;

use crate::Fit;

#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Definition,
    Data
}

// CURRENTLY NOT USED, EVALUATING
// #[derive(Debug, Clone)]
// pub enum Message {
//     Definition(DefinitionMessage),
//     Data(DataMessage)
// }

/// FIT message header.
#[derive(Debug, Default, Copy, Clone, BinRead)]
pub struct MessageHeader(u8);

impl MessageHeader {
    /// Checks whether message is a definition
    /// or a data message.
    // pub fn kind(&self) -> MessageType {
    pub fn kind(&self) -> Kind {
        match Fit::bit_set(self.0, 6) {
            true => Kind::Definition,
            false => Kind::Data,
        }
    }

    /// FIT local ID (0-15) for matching
    /// definition messages with data message
    /// during parse.
    ///
    /// Note that while `id()` supports compressed
    /// timestamp headers the rest of `fit-rs` does not.
    pub fn id(&self) -> u8 {
        // check for compressed time stamp header
        match self.comp_time() {
            Some(_) => (0b0110_0000 & self.0) >> 5,
            None => 0b0000_1111 & self.0
        }
    }

    /// Checks if header is a compressed
    /// time stamp header and returns the
    /// value if so.
    ///
    /// Note that while `comp_time()` supports compressed
    /// timestamp headers the rest of `fit-rs` does not.
    pub fn comp_time(&self) -> Option<u8> {
        match Fit::bit_set(self.0, 7) {
            true => Some(0b0001_1111 & self.0),
            false => None
        }
    }

    /// Returns `true` if definition contains developer fields.
    /// Only relvant for definition messages and always returns `false`
    /// for data messages, since there is no flag for developer data in data
    /// messages, this has to be looked up via the corresponding definition.
    pub fn dev(&self) -> bool {
        match self.kind() {
            Kind::Definition => Fit::bit_set(self.0, 5),
            Kind::Data => false
        }
    }
}