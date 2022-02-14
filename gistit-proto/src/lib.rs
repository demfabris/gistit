//
//   ________.__          __  .__  __
//  /  _____/|__| _______/  |_|__|/  |_
// /   \  ___|  |/  ___/\   __\  \   __\
// \    \_\  \  |\___ \  |  | |  ||  |
//  \______  /__/____  > |__| |__||__|
//         \/        \/
//
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![cfg_attr(
    test,
    allow(
        unused,
        clippy::all,
        clippy::pedantic,
        clippy::nursery,
        clippy::dbg_macro,
        clippy::unwrap_used,
        clippy::missing_docs_in_private_items,
    )
)]

pub mod hash;

pub use bytes;
pub use prost;

pub mod gistit {
    include!(concat!(env!("OUT_DIR"), "/gistit.gistit.rs"));
}

pub mod ipc {
    include!(concat!(env!("OUT_DIR"), "/gistit.ipc.rs"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;

    #[test]
    fn test_basic_encoding() {
        let mut instruction = ipc::Instruction::default();
        instruction.kind = Some(ipc::instruction::Kind::Status(ipc::instruction::Status {}));
        // instruction.kind = Some(ipc::instruction::Kind::Fetch(ipc::instruction::Fetch {
        //     hash: "asdas".to_string(),
        // }));

        let mut buf = bytes::BytesMut::new();
        instruction.encode(&mut buf).unwrap();
        dbg!(&buf);

        let result = ipc::Instruction::decode(buf).unwrap();
        assert_eq!(instruction, result);
        dbg!(&result);
        assert!(matches!(
            result,
            ipc::Instruction {
                kind: Some(ipc::instruction::Kind::Status(ipc::instruction::Status {}))
            }
        ));
    }
}
