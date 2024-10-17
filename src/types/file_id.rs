//! FIT `file_id` message (global ID 0). Required.
//! 
//! Contains general device information, such as device serial (`3/serial_number`),
//! and creation time (`4/time_created`).

use std::ops::Range;

use crate::{fit::DataMessage, Fit, FitError};

pub struct FileId {
    // Field definition number 3
    pub serial_number: u32, // SCL: 1 OFF: 0 UNIT: N/A Uint32z([3936074477])
    // Field definition number 4
    pub time_created: u32, // SCL: 1 OFF: 0 UNIT: N/A Uint32([1045736110])
    // VIRB Field definition number 7
    // UNKNOWN_FIELD   // SCL: 1 OFF: 0 UNIT: N/A Uint32([4294967295])
    // Field definition number 1
    pub manufacturer: u16, // SCL: 1 OFF: 0 UNIT: N/A Uint16([1])
    // Field definition number 2
    pub product: u16,     // SCL: 1 OFF: 0 UNIT: N/A Uint16([65535])
    // Field definition number 5
    pub number: u16,      // SCL: 1 OFF: 0 UNIT: N/A Uint16([65535])
    // Field definition number 0
    pub r#type: u8,       // SCL: 1 OFF: 0 UNIT: N/A Enum([4])
    pub(crate) index: usize,
}

impl FileId {
    /// Parse `DataMessage` as `FileId` (`file_id/0`).
    /// 
    /// Message required once for all devices.
    /// Contains general device information, 
    /// such as device serial (`3/serial_number`),
    /// and creation time (`4/time_created`).
    pub fn new(data_message: &DataMessage) -> Result<FileId, FitError> {
        let global_id = 0_u16; // gps_metadata

        if data_message.global != global_id {
            return Err(FitError::UnexpectedMessageType{expected: global_id, got: data_message.global})
        }

        let mut serial_number: Option<u32> = None; // field no 3
        let mut time_created: Option<u32> = None; // field no 4
        let mut manufacturer: Option<u16> = None; // field no 1
        let mut product: Option<u16> = None; // field no 2
        let mut number: Option<u16> = None; // field no 5
        let mut product_type: Option<u8> = None; // field no 0 enum

        for field in data_message.fields.iter() {
            match field.field_def_no() {
                3 => serial_number = field.data.as_ref().into(),
                4 => time_created = field.data.as_ref().into(),
                1 => manufacturer = field.data.as_ref().into(),
                2 => product = field.data.as_ref().into(),
                5 => number = field.data.as_ref().into(),
                0 => product_type = field.data.as_ref().into(),
                // ignore undocumented id:s,
                // found 7 in VIRB data so far
                _ => (),
            }
        }

        Ok(Self {
            serial_number: serial_number
                .ok_or_else(|| FitError::ErrorAssigningField { global: global_id, field_def_no: 3 })?,
            time_created: time_created
                .ok_or_else(|| FitError::ErrorAssigningField { global: global_id, field_def_no: 4 })?,
            manufacturer: manufacturer
                .ok_or_else(|| FitError::ErrorAssigningField { global: global_id, field_def_no: 1 })?,
            product: product
                .ok_or_else(|| FitError::ErrorAssigningField { global: global_id, field_def_no: 2 })?,
            number: number
                .ok_or_else(|| FitError::ErrorAssigningField { global: global_id, field_def_no: 5 })?,
            r#type: product_type
                .ok_or_else(|| FitError::ErrorAssigningField { global: global_id, field_def_no: 0 })?,
            index: data_message.index
        })
    }

    /// Extract `FileId` (`file_id/0`) from FIT.
    /// 
    /// Message required once for all devices.
    /// Contains general device information, 
    /// such as device serial (`3/serial_number`),
    /// and creation time (`4/time_created`).
    pub fn from_fit(
        fit: &Fit,
        range: Option<&Range<usize>>, // slice indeces for session
    ) -> Result<Self, FitError> {
        let global = 0_u16;

        let range = range.cloned().unwrap_or(0 .. fit.len());

        fit.records[range].iter() // should be very early so linear is fine
            .find(|rec| rec.global == global)
            .map(Self::new)
            .ok_or_else(|| FitError::ErrorParsingMessage(global))?
    }
}