//! The file module
use std::env::temp_dir;
use std::ffi::{OsStr, OsString};
use std::ops::RangeInclusive;
use std::path::Path;
use std::str;
use std::sync::Arc;

use async_trait::async_trait;
use crypto::mac::Mac;
use phf::{phf_map, Map};
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

use crate::encrypt::cryptor_simple;
use crate::errors::file::FileError;
use crate::{Error, Result};

/// Allowed file size range in bytes
const ALLOWED_FILE_SIZE_RANGE: RangeInclusive<u64> = 20..=200_000;

/// Expected encryption padding
const ENCRYPTED_FILE_PADDING: &str = "####";

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

/// The file structure that holds data to be checked/dispatched.
#[derive(Debug)]
pub struct File {
    pub inner: tokio::fs::File,
    pub path: OsString,
    pub bytes: Vec<u8>,
}

impl File {
    /// Constructs a [`File`] from a reference to [`super::Action`]
    ///
    /// # Errors
    ///
    /// Fails with [`std::io::Error`]
    pub async fn from_path(path: impl AsRef<OsStr> + Sync + Send) -> Result<Self> {
        let path_ref = path.as_ref();
        let mut inner = tokio::fs::File::open(path_ref).await?;
        let mut bytes: Vec<u8> = Vec::new();
        let _bytes_read = tokio::io::AsyncReadExt::read_to_end(&mut inner, &mut bytes).await?;
        inner.rewind().await?;
        println!("unencrypted bytes {:?}\n len {}", bytes, bytes.len());
        Ok(Self {
            inner,
            path: path_ref.to_os_string(),
            bytes,
        })
    }

    /// Constructs a [`File`] from bytes
    ///
    /// # Errors
    ///
    /// Fails with [`std::io::Error`]
    pub async fn from_bytes(bytes: impl AsRef<[u8]> + Send + Sync) -> Result<Self> {
        let tmp_file = temp_dir().join(".gistit_unknown");
        let mut inner = tokio::fs::File::create(&tmp_file).await?;
        inner.write_all(bytes.as_ref()).await?;
        Ok(Self {
            inner,
            path: tmp_file.into_os_string(),
            bytes: bytes.as_ref().to_vec(),
        })
    }

    /// Returns a reference to the file name
    ///
    /// # Errors
    ///
    /// Fails with [`FileError`]
    pub fn name(&self) -> String {
        Path::new(&self.path)
            .file_name()
            .expect("to be valid")
            .to_string_lossy()
            .to_string()
    }

    /// Returns the programming language that maps to this file extension
    pub fn lang(&self) -> &str {
        Path::new(self.path.as_os_str())
            .extension()
            .and_then(OsStr::to_str)
            .map(|t| EXTENSION_TO_LANG_MAPPING.get(t))
            .expect("to be valid")
            .expect("to be valid")
    }

    /// Encrypts the file and return it as a byte vector
    ///
    /// # Errors
    ///
    /// Fails with [`Encryption`] with the encryption process goes wrong
    pub async fn into_encrypted(self, key: impl AsRef<str> + Sync + Send) -> Result<EncryptedFile> {
        let encrypted = cryptor_simple(key.as_ref(), None)
            .into_encryptor()
            .encrypt(self.bytes())?;
        let (hmac_raw, bytes, magic) = (
            encrypted.hmac_raw_default().result().code().to_owned(),
            encrypted.as_bytes().to_owned(),
            encrypted.init_vector().to_vec(),
        );
        println!("encrypted bytes client {:?} len {}", bytes, bytes.len());
        println!("hmac client {:?} len {}", hmac_raw, hmac_raw.len());
        println!("iv client {:?} len {}", magic, magic.len());
        Ok(EncryptedFile {
            hmac_raw,
            prev: Some(Arc::new(self)),
            bytes,
            magic,
        })
    }

    /// Perform needed checks concurrently, consumes `Self` and return.
    ///
    /// # Errors
    ///
    /// Fails with [`UnsuportedFile`]
    pub async fn check_consume(self) -> Result<Self> {
        <Self as Check>::metadata(&self).await?;
        <Self as Check>::extension(&self)?;
        Ok(self)
    }
}

/// The file after encryption
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct EncryptedFile {
    pub hmac_raw: Vec<u8>,
    pub magic: Vec<u8>,
    pub bytes: Vec<u8>,
    pub prev: Option<Arc<File>>,
}

impl EncryptedFile {
    /// Create a [`EncryptedFile`] structure from encrypted bytes.
    /// The embedded hmac is expected to be length 16 followed by 4 padding characters '#'
    ///
    /// # Errors
    ///
    /// Fails with read error
    pub async fn from_bytes(bytes: impl AsRef<[u8]> + Send + Sync) -> Result<Self> {
        let bytes = bytes.as_ref();
        let (header, file_bytes) = bytes.split_at(40);
        let hmac: [u8; 16] = header[0..16].try_into().expect("to read");
        let first_padding: [u8; 4] = header[16..20].try_into().expect("to read");
        let magic_iv: [u8; 16] = header[20..36].try_into().expect("to read");
        let second_padding: [u8; 4] = header[36..].try_into().expect("to read");
        println!("hmac server {:?} len {}", hmac, hmac.len());
        println!("first_padding {:?}", first_padding);
        println!("magic_iv {:?} len {}", magic_iv, magic_iv.len());
        println!("second_padding {:?}", second_padding);
        println!("encrypted bytes {:?} len {}", file_bytes, file_bytes.len());

        if first_padding == ENCRYPTED_FILE_PADDING.as_bytes() && first_padding == second_padding {
            Ok(Self {
                hmac_raw: hmac.to_vec(),
                bytes: file_bytes.to_vec(),
                magic: magic_iv.to_vec(),
                prev: None,
            })
        } else {
            Err(Error::File(FileError::InvalidEncryptionPadding))
        }
    }

    /// # Errors
    /// asd
    pub async fn into_decrypted(self, key: impl AsRef<str> + Sync + Send) -> Result<File> {
        let iv: [u8; 16] = self.magic.clone().try_into().expect("To fit byte slice");
        println!("iv {:?}", iv);
        let decrypted = cryptor_simple(key.as_ref(), Some(iv))
            .into_decryptor()
            .decrypt(self.bytes())?;
        println!("{:?}", decrypted.as_bytes());
        if decrypted.verify(self.hmac_raw) {
            println!("valid");
        }
        Ok(File::from_bytes(decrypted.as_bytes()).await?)
    }
}

#[async_trait]
pub trait FileReady {
    async fn to_formatted(&self) -> Result<String>;

    async fn inner(&self) -> Option<Arc<File>>;

    fn bytes(&self) -> &[u8];
}

#[async_trait]
impl FileReady for EncryptedFile {
    async fn to_formatted(&self) -> Result<String> {
        let mut bytes_prefixed = self.hmac_raw.clone();
        // Encryption header
        bytes_prefixed.extend(ENCRYPTED_FILE_PADDING.as_bytes());
        bytes_prefixed.extend(self.magic.clone());
        bytes_prefixed.extend(ENCRYPTED_FILE_PADDING.as_bytes());
        bytes_prefixed.extend(self.bytes.clone());
        Ok(base64::encode(bytes_prefixed))
    }

    async fn inner(&self) -> Option<Arc<File>> {
        (&self.prev).clone()
    }

    fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[async_trait]
impl FileReady for File {
    async fn to_formatted(&self) -> Result<String> {
        let mut buffer: Vec<u8> = Vec::new();
        let mut file = self.inner.try_clone().await?;
        file.rewind().await?;
        let _bytes_read = tokio::io::AsyncReadExt::read_to_end(&mut file, &mut buffer).await?;
        Ok(base64::encode(buffer))
    }

    async fn inner(&self) -> Option<Arc<Self>> {
        Some(Arc::new(Self {
            inner: self.inner.try_clone().await.ok()?,
            path: self.path.clone(),
            bytes: self.bytes().to_vec(),
        }))
    }

    fn bytes(&self) -> &[u8] {
        &self.bytes
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
        let attr = self.inner.metadata().await?;
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
