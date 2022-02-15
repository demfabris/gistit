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

pub use bytes;
pub use prost;

pub use ipc::Instruction;
pub use payload::{gistit::Inner, Gistit};

pub mod payload {
    use super::prost::Message;
    use super::Result;
    use sha2::{Digest, Sha256};

    include!(concat!(env!("OUT_DIR"), "/gistit.payload.rs"));

    pub fn hash(author: &str, description: Option<&str>, data: impl AsRef<[u8]>) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.update(author);
        hasher.update(description.unwrap_or(""));

        format!("{:x}", hasher.finalize())
    }

    impl Gistit {
        #[must_use]
        pub fn new(
            hash: String,
            author: String,
            description: Option<String>,
            timestamp: String,
            inner: Vec<gistit::Inner>,
        ) -> Self {
            Self {
                hash,
                author,
                description,
                timestamp,
                inner,
            }
        }

        #[must_use]
        pub const fn new_inner(
            name: String,
            lang: String,
            size: u32,
            data: String,
        ) -> gistit::Inner {
            gistit::Inner {
                name,
                lang,
                size,
                data,
            }
        }

        /// Decodes a buffer into [`Self`]
        ///
        /// # Errors
        ///
        /// Fails if buffer doesn't contain protobuf encoded data
        pub fn from_bytes(bytes: impl AsRef<[u8]>) -> Result<Self> {
            Ok(Self::decode(bytes.as_ref())?)
        }
    }
}

pub mod ipc {
    use super::Gistit;
    use super::{Error, Result};

    include!(concat!(env!("OUT_DIR"), "/gistit.ipc.rs"));

    impl Instruction {
        #[must_use]
        pub const fn request_status() -> Self {
            Self {
                kind: Some(instruction::Kind::StatusRequest(
                    instruction::StatusRequest {},
                )),
            }
        }

        #[must_use]
        pub const fn request_fetch(hash: String) -> Self {
            Self {
                kind: Some(instruction::Kind::FetchRequest(instruction::FetchRequest {
                    hash,
                })),
            }
        }

        #[must_use]
        pub const fn request_provide(gistit: Gistit) -> Self {
            Self {
                kind: Some(instruction::Kind::ProvideRequest(
                    instruction::ProvideRequest {
                        gistit: Some(gistit),
                    },
                )),
            }
        }

        #[must_use]
        pub const fn request_shutdown() -> Self {
            Self {
                kind: Some(instruction::Kind::ShutdownRequest(
                    instruction::ShutdownRequest {},
                )),
            }
        }

        /// Unwraps [`Self`] expecting a response kind
        ///
        /// # Errors
        ///
        /// Fails if instruction is not a response or is none
        #[allow(clippy::missing_const_for_fn)]
        pub fn expect_response(self) -> Result<instruction::Kind> {
            match self {
                Self {
                    kind:
                        Some(
                            instruction::Kind::FetchRequest(_)
                            | instruction::Kind::StatusRequest(_)
                            | instruction::Kind::ShutdownRequest(_)
                            | instruction::Kind::ProvideRequest(_),
                        )
                        | None,
                } => Err(Error::Other("instruction is not a response")),
                Self {
                    kind: Some(response),
                } => Ok(response),
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("decode error {0}")]
    Decode(#[from] prost::DecodeError),

    #[error("encode error {0}")]
    Encode(#[from] prost::EncodeError),

    #[error("server error {0}")]
    Server(String),

    #[error("other error {0}")]
    Other(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;
}
