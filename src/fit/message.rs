use std::{io::Cursor, collections::HashMap};

use binrw::{BinReaderExt, BinRead};

use crate::{FitError, FieldDescriptionMessage};

use super::{MessageHeader, DefinitionMessage, DataMessage, Kind};

#[derive(Debug, Clone, BinRead)]
// #[br(import(header: MessageHeader))]
#[br(import(header: MessageHeader))]
pub enum MessageType {
    Definition(
        #[br(assert(header.kind() == Kind::Definition))]
        // #[br(args(header.dev() == true))]
        #[br(args(header.dev()))]
        DefinitionMessage
    ),
    Data(
        // data message currently does not parse via binrw since this requires a definition message
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
    pub(crate) header: MessageHeader,
    #[br(args(header))]
    pub(crate) message: MessageType,
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

        let message = match header.kind() {
            Kind::Definition => MessageType::read_ne_args(cursor, (header,))?,
            Kind::Data => {
                let definition = definitions.get(&id)
                    .ok_or_else(|| FitError::UnknownDefinition {local: id, offset: pos})?;
                let data_message = DataMessage::parse(
                    cursor,
                    definition,
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