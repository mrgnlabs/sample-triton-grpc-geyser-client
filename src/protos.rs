// These macros rely on protobuf sources generated during the `prebuild` step (c.f. `build.rs`)

// Module nesting needs to match the `package` name as per proto file (1 dot <=> 1 level)
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
