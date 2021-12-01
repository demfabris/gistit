//! The file module
//!
//! Here we define file structures and methods. It is implemented using [`tokio`] so we don't block
//! progress output during the process.

use std::env::temp_dir;
use std::ffi::OsStr;
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};
use std::str;

use async_trait::async_trait;
use phf::{phf_map, Map};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

use crate::encrypt::{decrypt_aes256_u12nonce, encrypt_aes256_u12nonce};
use crate::errors::file::FileError;
use crate::{Error, Result};

#[cfg(doc)]
use crate::errors::{encryption::EncryptionError, file::FileError, io::IoError};

/// Allowed file size range in bytes
const ALLOWED_FILE_SIZE_RANGE: RangeInclusive<u64> = 20..=200_000;

/// The expected file header encryption padding
const FILE_HEADER_ENCRYPTION_PADDING: &str = "########";

/// Type alias for a base64 encoded and AES256 encrypted file with embedded header
pub type HeadfulEncryptedB64String = String;

/// Type alias for the fully processed file data
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct EncodedFileData {
    pub inner: HeadfulEncryptedB64String,
}

/// Supported file extensions
/// This is a compile time built hashmap to check incomming file extensions against.
/// Follows the extensions supported by currently UI syntax highlighting lib:
/// [`react-syntax-highlighter`](https://gist.github.com/ppisarczyk/43962d06686722d26d176fad46879d41)
//TODO: fill this https://gist.github.com/ppisarczyk/43962d06686722d26d176fad46879d41
const EXTENSION_TO_LANG_MAPPING: Map<&'static str, &'static str> = phf_map! {
    "abap" => "abap",
    "as" => "actionscript",
    "ada" => "ada",
    "adb" => "ada",
    "ads" => "ada",
    "agda" => "agda",
    "als" => "al",
    "g4" => "antlr4",
    "apacheconf" => "apacheconf",
    "vhost" => "apacheconf",
    "apl" => "apl",
    "dyalog" => "apl",
    "applescript" => "applescript",
    "scpt" => "applescript",
    "ino" => "arduino",
    "asciidoc" => "asciidoc",
    "adoc" => "asciidoc",
    "asc" => "asciidoc",
    "asm" => "asm6502",
    "a51" => "asm6502",
    "inc" => "asm6502",
    "nasm" => "asm6502",
    "asp" => "aspnet",
    "asax" => "aspnet",
    "ascx" => "aspnet",
    "ashx" => "aspnet",
    "asmx" => "aspnet",
    "aspx" => "aspnet",
    "axd" => "aspnet",
    "md" => "markdown",
    "ts" => "typescript",
    "rs" => "rust",
    "toml" => "toml",
    // "" => "autohotkey",
    // "" => "autoit",
    // "" => "bash",
    // "" => "basic",
    // "" => "batch",
    // "" => "bbcode",
    // "" => "birb",
    // "" => "bison",
    // "" => "bnf",
    // "" => "brainfuck",
    // "" => "brightscript",
    // "" => "bro",
    // "" => "bsl",
    // "" => "c",
    // "" => "cil",
    // "" => "clike",
    // "" => "clojure",
    // "" => "cmake",
    // "" => "coffeescript",
    // "" => "concurnas",
    // "" => "cpp",
    // "" => "crystal",
    // "" => "csharp",
    // "" => "csp",
    // "" => "cssExtras",
    // "" => "css",
    // "" => "cypher",
    // "" => "d",
    // "" => "dart",
    // "" => "dax",
    // "" => "dhall",
    // "" => "diff",
    // "" => "django",
    // "" => "dnsZoneFile",
    // "" => "docker",
    // "" => "ebnf",
    // "" => "editorconfig",
    // "" => "eiffel",
    // "" => "ejs",
    // "" => "elixir",
    // "" => "elm",
    // "" => "erb",
    // "" => "erlang",
    // "" => "etlua",
    // "" => "excelFormula",
    // "" => "factor",
    // "" => "firestoreSecurityRules",
    // "" => "flow",
    // "" => "fortran",
    // "" => "fsharp",
    // "" => "ftl",
    // "" => "gcode",
    // "" => "gdscript",
    // "" => "gedcom",
    // "" => "gherkin",
    // "" => "git",
    // "" => "glsl",
    // "" => "gml",
    // "" => "go",
    // "" => "graphql",
    // "" => "groovy",
    // "" => "haml",
    // "" => "handlebars",
    // "" => "haskell",
    // "" => "haxe",
    // "" => "hcl",
    // "" => "hlsl",
    // "" => "hpkp",
    // "" => "hsts",
    // "" => "http",
    // "" => "ichigojam",
    // "" => "icon",
    // "" => "iecst",
    // "" => "ignore",
    // "" => "inform7",
    // "" => "ini",
    // "" => "io",
    // "" => "j",
    // "" => "java",
    // "" => "javadoc",
    // "" => "javadoclike",
    // "" => "javascript",
    // "" => "javastacktrace",
    // "" => "jolie",
    // "" => "jq",
    // "" => "jsExtras",
    // "" => "jsTemplates",
    // "" => "jsdoc",
    // "" => "json",
    // "" => "json5",
    // "" => "jsonp",
    // "" => "jsstacktrace",
    // "" => "jsx",
    // "" => "julia",
    // "" => "keyman",
    // "" => "kotlin",
    // "" => "latex",
    // "" => "latte",
    // "" => "less",
    // "" => "lilypond",
    // "" => "liquid",
    // "" => "lisp",
    // "" => "livescript",
    // "" => "llvm",
    // "" => "lolcode",
    // "" => "lua",
    // "" => "makefile",
    // "" => "markupTemplating",
    // "" => "markup",
    // "" => "matlab",
    // "" => "mel",
    // "" => "mizar",
    // "" => "mongodb",
    // "" => "monkey",
    // "" => "moonscript",
    // "" => "n1ql",
    // "" => "n4js",
    // "" => "nand2tetrisHdl",
    // "" => "naniscript",
    // "" => "nasm",
    // "" => "neon",
    // "" => "nginx",
    // "" => "nim",
    // "" => "nix",
    // "" => "nsis",
    // "" => "objectivec",
    // "" => "ocaml",
    // "" => "opencl",
    // "" => "oz",
    // "" => "parigp",
    // "" => "parser",
    // "" => "pascal",
    // "" => "pascaligo",
    // "" => "pcaxis",
    // "" => "peoplecode",
    // "" => "perl",
    // "" => "phpExtras",
    // "" => "php",
    // "" => "phpdoc",
    // "" => "plsql",
    // "" => "powerquery",
    // "" => "powershell",
    // "" => "processing",
    // "" => "prolog",
    // "" => "properties",
    // "" => "protobuf",
    // "" => "pug",
    // "" => "puppet",
    // "" => "pure",
    // "" => "purebasic",
    // "" => "purescript",
    // "" => "python",
    // "" => "q",
    // "" => "qml",
    // "" => "qore",
    // "" => "r",
    // "" => "racket",
    // "" => "reason",
    // "" => "regex",
    // "" => "renpy",
    // "" => "rest",
    // "" => "rip",
    // "" => "roboconf",
    // "" => "robotframework",
    // "" => "ruby",
    // "" => "sas",
    // "" => "sass",
    // "" => "scala",
    // "" => "scheme",
    // "" => "scss",
    // "" => "shellSession",
    // "" => "smali",
    // "" => "smalltalk",
    // "" => "smarty",
    // "" => "sml",
    // "" => "solidity",
    // "" => "solutionFile",
    // "" => "soy",
    // "" => "sparql",
    // "" => "splunkSpl",
    // "" => "sqf",
    // "" => "sql",
    // "" => "stan",
    // "" => "stylus",
    // "" => "swift",
    // "" => "t4Cs",
    // "" => "t4Templating",
    // "" => "t4Vb",
    // "" => "tap",
    // "" => "tcl",
    // "" => "textile",
    // "" => "tsx",
    // "" => "tt2",
    // "" => "turtle",
    // "" => "twig",
    // "" => "typoscript",
    // "" => "unrealscript",
    // "" => "vala",
    // "" => "vbnet",
    // "" => "velocity",
    // "" => "verilog",
    // "" => "vhdl",
    // "" => "vim",
    // "" => "visualBasic",
    // "" => "warpscript",
    // "" => "wasm",
    // "" => "wiki",
    // "" => "xeora",
    // "" => "xmlDoc",
    // "" => "xojo",
    // "" => "xquery",
    // "" => "yaml",
    // "" => "yang",
    // "" => "zig",
};

/// Represents a gistit file handler and some extra data
#[derive(Debug)]
pub struct File {
    /// Opened file handler
    handler: tokio::fs::File,
    /// Path in system
    path: PathBuf,
    /// Bytes read from file handler
    bytes: Vec<u8>,
    /// A custom file name
    name: Option<String>,
}

fn _name_from_path(path: &Path) -> String {
    path.file_name()
        // Checked previously
        .expect("File name to be valid")
        .to_string_lossy()
        .to_string()
}

impl File {
    /// Opens a file from the given `path` and returns a [`File`] handler.
    ///
    /// # Errors
    ///
    /// Fails with [`IoError`] if the file can't be opened, which probably means the file doesn't
    /// exist, path is invalid, or file handler is blocked.
    pub async fn from_path(path: &Path) -> Result<Self> {
        let mut handler = tokio::fs::File::open(path).await?;
        let mut bytes: Vec<u8> = Vec::new();
        tokio::io::AsyncReadExt::read_to_end(&mut handler, &mut bytes).await?;

        Ok(Self {
            handler,
            path: path.to_path_buf(),
            bytes,
            name: Some(_name_from_path(path)),
        })
    }

    /// Creates a new file in your system `temp` with a random name and writes provided `bytes`
    /// into it. Returns the new [`File`] handler
    ///
    /// # Errors
    ///
    /// Fails with [`IoError`] if the file can't be created for some reason. Also if it can't be
    /// written to.
    pub async fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let path = rng_temp_file();

        let mut handler = tokio::fs::File::create(&path).await?;
        handler.write_all(bytes).await?;

        Ok(Self {
            handler,
            name: Some(_name_from_path(&path)),
            path,
            bytes: bytes.to_vec(),
        })
    }

    /// Set the file name, useful when creating a [`File`] using [`from_bytes`].
    /// if the [`File`] was created using [`from_path`] it will use the provided file name.
    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn with_name(self, name: String) -> Self {
        Self {
            name: Some(name),
            ..self
        }
    }

    /// Returns the file name
    pub fn name(&self) -> String {
        self.name
            .clone()
            .expect("Opened file to have at least temp name")
    }

    /// Returns the programming language that maps to this file extension
    pub fn lang(&self) -> &str {
        self.path
            .extension()
            .and_then(OsStr::to_str)
            .map(|t| EXTENSION_TO_LANG_MAPPING.get(t))
            // Checked previously
            .expect("Valid file extension")
            .expect("Extension to be supported")
    }

    /// Returns the file size in bytes, not encoded.
    pub async fn size(&self) -> u64 {
        self.handler
            .metadata()
            .await
            .expect("The file to be open")
            .len()
    }

    /// Consumes the [`File`] encrypting it and returning a new instance of [`EncryptedFile`]
    ///
    /// # Errors
    ///
    /// Fails with [`EncryptionError`] if something goes wrong during the encryption process. This
    /// includes unexpected sizes of the nounce, hashed key.
    /// Will also error out if the provided key and nounce is incorrect.
    pub async fn into_encrypted(self, secret: &str) -> Result<EncryptedFile> {
        let (encrypted_bytes, nounce) = encrypt_aes256_u12nonce(secret.as_bytes(), self.data())?;
        let name = self.name.clone();

        Ok(EncryptedFile {
            encrypted_bytes,
            nounce,
            prev: Some(Box::new(self)),
            name,
        })
    }

    /// Perform needed checks concurrently, consumes `Self` and return.
    ///
    /// # Errors
    ///
    /// Fails with [`FileError`] if user input isn't valid
    pub async fn check_consume(self) -> Result<Self> {
        <Self as Check>::metadata(&self).await?;
        <Self as Check>::extension(&self)?;

        Ok(self)
    }
}

/// Represents a encrypted gistit file data.
/// This data structure is expected to hold encrypted but not encoded bytes in `encrypted_bytes`, the `nounce`
/// which is a 12 bytes randomly generated byte array, and a pointer to the previous unencrypted [`File`]
/// handler.
#[derive(Debug)]
pub struct EncryptedFile {
    /// The encrypted bytes
    encrypted_bytes: Vec<u8>,
    /// The random sequence used to encrypt
    nounce: Vec<u8>,
    /// Pointer to maybe the previous unencrypted
    prev: Option<Box<File>>,
    /// Overwrite the random file name during decryption
    name: Option<String>,
}

/// Extract and verify the encrypted file header which contains the `nounce` and a expected 8 bytes
/// long padding defined in [`FILE_HEADER_ENCRYPTION_PADDING`].
///
/// # Errors
///
/// Fails with [`FileError`] if the padding is invalid or the `nounce` is incorrectly sized.
fn parse_encryption_header(bytes: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    let (header, rest) = bytes.split_at(20);
    let (nounce, padding) = header.split_at(12);

    if padding == FILE_HEADER_ENCRYPTION_PADDING.as_bytes() {
        Ok((nounce.to_vec(), rest.to_vec()))
    } else {
        Err(Error::File(FileError::InvalidEncryptionPadding))
    }
}

impl EncryptedFile {
    /// Creates a new [`EncryptedFile`] handler from encrypted and **encoded** byte array.
    /// That means it should contain the expected encryption header and be base64 encoded.
    ///
    /// Will create a new temporary file in your system `temp` directory and write **decoded** and
    /// still encrypted contents into it.
    ///
    /// **note** that there is no `prev` unencrypted [`File`] handler
    ///
    /// # Errors
    ///
    /// Fails with [`IoError`] if can't create or write to the file handler.
    /// Fails with [`FileError`] if the encryption header is invalid.
    pub async fn from_bytes(encoded_bytes: &[u8]) -> Result<Self> {
        let decoded_bytes = base64::decode(encoded_bytes)?;
        let path = rng_temp_file();

        let (nounce, encrypted_bytes) = parse_encryption_header(&decoded_bytes)?;
        let mut handler = tokio::fs::File::create(&path).await?;
        handler.write_all(&encrypted_bytes).await?;

        Ok(Self {
            encrypted_bytes,
            nounce,
            prev: None,
            name: Some(_name_from_path(&path)),
        })
    }

    /// Set a file name, if attempt to decrypt without a file name this will be set to a random
    /// string.
    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn with_name(self, name: String) -> Self {
        Self {
            name: Some(name),
            ..self
        }
    }

    /// Converts [`Self`] into [`File`] handler by applying the decryption process with the
    /// provided secret.
    ///
    /// # Errors
    ///
    /// Fails with [`EncryptionError`] if `nonce` or `secret` is incorrect
    pub async fn into_decrypted(self, secret: &str) -> Result<File> {
        let nounce: [u8; 12] = self
            .nounce
            .clone()
            .try_into()
            .expect("Shrink nounce to 12 bytes");

        let decrypted_bytes = decrypt_aes256_u12nonce(secret.as_bytes(), self.data(), &nounce)?;
        let file = File::from_bytes(&decrypted_bytes)
            .await?
            .with_name(self.name.expect("Opened file to have at least temp name"));
        Ok(file)
    }
}

/// Returns a new randomnly generated file path in your system `temp` directory
fn rng_temp_file() -> PathBuf {
    let rng_string: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();

    let mut rng_file_name = "__gistit_tmp_".to_owned();
    rng_file_name.push_str(&rng_string);

    temp_dir().join(&rng_file_name)
}

/// Represents the opened file handler
#[async_trait]
pub trait FileReady {
    /// Returns a reference to the original [`File`] handler if any.
    async fn inner(self: Box<Self>) -> Option<Box<File>>;

    /// Returns a reference to the underlying data. Can be encrypted depending on the source
    fn data(&self) -> &[u8];

    /// Converts [`Self`] into the sendable form of the data
    fn to_encoded_data(&self) -> EncodedFileData;
}

#[async_trait]
impl FileReady for EncryptedFile {
    async fn inner(self: Box<Self>) -> Option<Box<File>> {
        self.prev
    }

    fn data(&self) -> &[u8] {
        &self.encrypted_bytes
    }

    fn to_encoded_data(&self) -> EncodedFileData {
        let mut headful_data = self.nounce.clone();
        headful_data.extend(FILE_HEADER_ENCRYPTION_PADDING.as_bytes());
        headful_data.extend_from_slice(&self.encrypted_bytes);

        EncodedFileData {
            inner: base64::encode(headful_data),
        }
    }
}

#[async_trait]
impl FileReady for File {
    async fn inner(self: Box<Self>) -> Option<Box<Self>> {
        Some(self)
    }

    fn data(&self) -> &[u8] {
        &self.bytes
    }

    fn to_encoded_data(&self) -> EncodedFileData {
        EncodedFileData {
            inner: base64::encode(&self.bytes),
        }
    }
}

#[async_trait]
trait Check {
    /// Checks the file metadata for type and size
    ///
    /// # Errors
    ///
    /// Fails with [`UnsuportedFile`] if size and type isn't allowed.
    async fn metadata(&self) -> Result<()>;

    /// Checks the file extension against [`EXTENSION_TO_LANG_MAPPING`]
    ///
    /// # Errors
    ///
    /// Fails with [`UnsuportedFile`] if file extension isn't supported.
    fn extension(&self) -> Result<()>;
}

#[async_trait]
impl Check for File {
    async fn metadata(&self) -> Result<()> {
        let attr = self.handler.metadata().await?;
        let size_allowed = ALLOWED_FILE_SIZE_RANGE.contains(&attr.len());
        let type_allowed = attr.is_file();

        if !size_allowed {
            return Err(FileError::UnsupportedSize(attr.len()).into());
        } else if !type_allowed {
            return Err(FileError::UnsupportedType(self.path.to_string_lossy().to_string()).into());
        }
        Ok(())
    }
    fn extension(&self) -> Result<()> {
        let ext = Path::new(self.path.as_os_str())
            .extension()
            .and_then(OsStr::to_str)
            .ok_or(FileError::MissingExtension)?;

        if EXTENSION_TO_LANG_MAPPING.contains_key(ext) {
            Ok(())
        } else {
            Err(FileError::UnsupportedExtension(ext.to_owned()).into())
        }
    }
}
