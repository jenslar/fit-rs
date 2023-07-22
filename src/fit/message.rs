use std::{io::Cursor, ops::Deref, collections::HashMap};

use binrw::{BinReaderExt, BinRead};

use crate::{FitError, FieldDescriptionMessage};

use super::{MessageHeader, DefinitionMessage, DataMessage, Kind};

#[derive(Debug, Clone, BinRead)]
// #[br(import(contains_developer_fields: bool, kind: Kind))]
#[br(import(header: MessageHeader))]
pub enum MessageType {
    Definition(
        // #[br(assert(kind == Kind::Definition))]
        #[br(assert(header.kind() == Kind::Definition))]
        // #[br(args(contains_developer_fields))]
        #[br(args(header.dev() == true))]
        DefinitionMessage
    ),
    Data(
        // currently does not use binrw for parse since this requires a definition message
        // #[br(assert(kind == Kind::Data), default)]
        #[br(assert(header.kind() == Kind::Data), default)]
        DataMessage
    ),
    Default
}

impl Default for MessageType {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Debug, Default, BinRead)]
pub struct Message {
    header: MessageHeader,
    #[br(args(header))]
    message: MessageType,
    // #[br(default)]
    // index: usize
}

impl Message {
    /// Parse FIT message. Results in either a definition message,
    /// or a data message (requires definition).
    pub fn parse(
        cursor: &mut Cursor<Vec<u8>>,
        definitions: &HashMap<u8, DefinitionMessage>
    ) -> Result<Self, FitError> {

        let header: MessageHeader = cursor.read_ne()?;
        let id = header.id();
        let pos = cursor.position();

        // let msgtype = match MessageType::read_args(cursor, (&header,))? {
        let message = match header.kind() {
            Kind::Definition => MessageType::read_ne_args(cursor, (header,))?, // native endian
            Kind::Data => {
                let definition = definitions.get(&id)
                        .ok_or_else(|| FitError::UnknownDefinition {local: id, offset: pos})?;
                let data_message = DataMessage::parse(
                    // data,
                    cursor,
                    definition,
                    // data_index
                )?;
                MessageType::Data(data_message)
            },
        };

        Ok(Message {
            header,
            message
        })
    }

    pub fn message_type(self) -> MessageType {
        self.message
    }

    /// Returns local identifier (0-15)
    /// that matches data with a definition.
    pub fn id(&self) -> u8 {
        self.header.id()
    }

    /// Returns enum denoting whether message
    /// is a definition or data.
    pub fn kind(&self) -> Kind {
        self.header.kind()
    }

    /// Augments definition message with developer
    /// data defintions, a.k.a. field description message
    /// (FIT global ID `206`).
    pub fn with_field_descriptions(
        &mut self,
        field_descriptions: &HashMap<(u8, u8), FieldDescriptionMessage>
    ) {
        if let MessageType::Definition(def) = &mut self.message {
            def.with_field_descriptions(field_descriptions)
        }
    }
}