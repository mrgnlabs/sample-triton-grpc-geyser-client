use itertools::Itertools;
use solana_sdk::{
    hash::Hash,
    instruction::CompiledInstruction,
    message::{
        legacy,
        v0::{self, MessageAddressTableLookup},
        MessageHeader, VersionedMessage,
    },
    pubkey::Pubkey,
    signature::Signature,
    transaction::VersionedTransaction,
};

pub mod solana {
    pub mod storage {
        pub mod confirmed_block {
            tonic::include_proto!("solana.storage.confirmed_block");
        }
    }
}

pub mod geyser {
    tonic::include_proto!("geyser");
}

impl From<solana::storage::confirmed_block::CompiledInstruction> for CompiledInstruction {
    fn from(instruction_proto: solana::storage::confirmed_block::CompiledInstruction) -> Self {
        Self {
            program_id_index: instruction_proto.program_id_index as u8,
            accounts: instruction_proto.accounts,
            data: instruction_proto.data,
        }
    }
}

impl From<solana::storage::confirmed_block::MessageHeader> for MessageHeader {
    fn from(header_proto: solana::storage::confirmed_block::MessageHeader) -> Self {
        Self {
            num_required_signatures: header_proto.num_required_signatures as u8,
            num_readonly_signed_accounts: header_proto.num_readonly_signed_accounts as u8,
            num_readonly_unsigned_accounts: header_proto.num_readonly_unsigned_accounts as u8,
        }
    }
}

impl From<solana::storage::confirmed_block::MessageAddressTableLookup>
    for MessageAddressTableLookup
{
    fn from(lut_proto: solana::storage::confirmed_block::MessageAddressTableLookup) -> Self {
        Self {
            account_key: Pubkey::new(&lut_proto.account_key),
            writable_indexes: lut_proto.writable_indexes,
            readonly_indexes: lut_proto.readonly_indexes,
        }
    }
}

impl From<solana::storage::confirmed_block::Message> for legacy::Message {
    fn from(message_proto: solana::storage::confirmed_block::Message) -> Self {
        let message_header_proto = message_proto.header.expect("missing message header");
        Self {
            account_keys: message_proto
                .account_keys
                .iter()
                .map(|sig_bytes| Pubkey::new(&sig_bytes))
                .collect_vec(),
            header: message_header_proto.into(),
            recent_blockhash: Hash::new(&message_proto.recent_blockhash),
            instructions: message_proto.instructions.into_iter().map_into().collect(),
        }
    }
}

impl From<solana::storage::confirmed_block::Message> for v0::Message {
    fn from(message_proto: solana::storage::confirmed_block::Message) -> Self {
        let message_header_proto = message_proto.header.expect("missing message header");
        Self {
            header: message_header_proto.into(),
            account_keys: message_proto
                .account_keys
                .iter()
                .map(|sig_bytes| Pubkey::new(&sig_bytes))
                .collect_vec(),
            recent_blockhash: Hash::new(&message_proto.recent_blockhash),
            instructions: message_proto.instructions.into_iter().map_into().collect(),
            address_table_lookups: message_proto
                .address_table_lookups
                .into_iter()
                .map_into()
                .collect(),
        }
    }
}

impl From<solana::storage::confirmed_block::Message> for VersionedMessage {
    fn from(message_proto: solana::storage::confirmed_block::Message) -> Self {
        match message_proto.versioned {
            false => VersionedMessage::Legacy(message_proto.into()),
            true => VersionedMessage::V0(message_proto.into()),
        }
    }
}

impl From<solana::storage::confirmed_block::Transaction> for VersionedTransaction {
    fn from(transaction_proto: solana::storage::confirmed_block::Transaction) -> Self {
        let message_proto = transaction_proto.message.expect("missing message");
        Self {
            signatures: transaction_proto
                .signatures
                .iter()
                .map(|sig_bytes| Signature::new(&sig_bytes))
                .collect_vec(),
            message: message_proto.into(),
        }
    }
}
