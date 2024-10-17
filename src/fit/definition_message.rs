//! FIT definition message.
//! Holds definitions for data messages documented in [FIT SDK](https://developer.garmin.com/fit).

use std::collections::HashMap;

use binrw::BinRead;
use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};

use crate::FieldDescriptionMessage;
use super::DefinitionField;

/// FIT definition message.
#[derive(Debug, Default, BinRead, Clone)]
#[br(import(contains_developer_fields: bool))]
pub struct DefinitionMessage {
    /// Reserved. Byte 0.
    _reserved: u8,
    /// Architecture/Endianess. Byte 1.
    /// Possible values: 0=LE, 1=BE
    pub(crate) architecture: u8,
    /// FIT global message ID. Bytes 2-3.
    #[br(is_little = (architecture == 0))]
    pub global: u16,
    _number_of_fields: u8,
    /// Definition fields. Bytes 3.. (varying length)
    #[br(count = _number_of_fields)]
    pub fields: Vec<DefinitionField>,
    /// Number of developer fields. Optional.
    #[br(if(contains_developer_fields, 0))]
    _number_of_devfields: u8,
    /// Developer definition fields (3 bytes/each). Optional.
    #[br(count = _number_of_devfields)]
    pub dev_fields: Vec<DefinitionField>,
}

impl DefinitionMessage {
    pub fn with_field_descriptions(
        &mut self,
        field_descriptions: &HashMap<(u8, u8), FieldDescriptionMessage>
    ) {
        self.dev_fields.par_iter_mut()
            .for_each(|field| {
                if let Some(field_descr) = field_descriptions.get(&(field.field_def_no, field.base_type.number())) {
                    field.augment(field_descr);
                }
            });
    }

    /// Returns the size of the data in bytes the definition describes,
    /// excluding the 1 byte header.
    pub fn data_size(&self) -> i64 {
        // technically lacking 1 byte for header,
        // but not needed since the cursor has already moved
        // 1 byte for the header
        self.fields.iter()
            .chain(self.dev_fields.iter())
            .map(|def| def.size as i64)
            .sum()
    }
}
