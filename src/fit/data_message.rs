//! FIT data message.

use std::collections::HashMap;
use std::fmt;
use std::io::Cursor;

use binrw::BinRead;

use crate::errors::FitError;

use super::DefinitionMessage;
use super::data_field::DataField;

#[derive(Debug, Default, Clone, BinRead)]
#[br(import(field_count: usize, dev_field_count: usize))]
/// FIT data message. Contains raw data.
pub struct DataMessage {
    // header: MessageHeader,
    #[br(ignore)]
    pub global: u16,
    /// Name in FIT SDK `Profile.xlsx`.
    /// Optionally set after initial parse.
    #[br(ignore)]
    pub name: Option<String>,
    /// Data fields.
    #[br(count = field_count)]
    pub fields: Vec<DataField>,
    /// Developer data fields.
    #[br(count = dev_field_count)]
    pub dev_fields: Vec<DataField>,
    /// Message index.
    #[br(default)]
    pub index: usize // slight performance decrease (50ms -> 52ms for large.fit)
}

impl DataMessage {
    pub fn with_global(self, global: u16) -> Self {
        Self {
            global,
            ..self
        }
    }

    pub fn with_name(self, name: &str) -> Self {
        Self {
            name: Some(name.to_owned()),
            ..self
        }
    }

    pub fn with_index(&mut self, index: usize) {
        self.index = index;
    }

    /// New FIT data message. `cursor` represents the full FIT file data load,
    /// but its offset must be at the start of a data message.
    pub fn parse(
        cursor: &mut Cursor<Vec<u8>>,
        definition: &DefinitionMessage,
    ) -> Result<Self, FitError> {
        
        let arch = definition.architecture;
        
        let fields = definition.fields.iter()
            .map(|def| DataField::new(cursor, def, arch)) // slightly slower than direct init
            .collect::<Result<Vec<DataField>, FitError>>()?;

        let dev_fields = definition.dev_fields.iter()
            .map(|dev_def| DataField::new(cursor, dev_def, arch)) // slightly slower than direct init
            .collect::<Result<Vec<DataField>, FitError>>()?;

        Ok(Self {
            global: definition.global,
            name: None,
            fields,
            dev_fields,
            index: usize::default(),
        })
    }

    /// Returns name/message type if set,
    /// and defaults to `UNKNOWN_TYPE_<ID>` if not.
    pub fn name(&self) -> String {
        self.name
            .as_ref()
            .map_or(format!("UNKNOWN_TYPE_{}", self.global), |u| u.to_owned())
    }
}

impl fmt::Display for DataMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Global: {} | {}",
            self.global,
            self.name.as_ref().map_or("UNKNOWN_TYPE", |n| n),
        )?;
        for fld in self.fields.iter() {
            writeln!(f, "      {}", fld)?;
        }
        for fld in self.dev_fields.iter() {
            writeln!(f, "  DEV {}", fld)?;
        }
        Ok(())
    }
}
