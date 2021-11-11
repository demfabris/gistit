//! The file module
use std::ffi::{OsStr, OsString};
use std::ops::RangeInclusive;
use std::path::Path;

use async_trait::async_trait;
use crypto::mac::Mac;
use phf::{phf_map, Map};

use crate::encrypt::cryptor_simple;
use crate::errors::file::FileError;
use crate::Result;

/// Allowed file size range in bytes
const ALLOWED_FILE_SIZE_RANGE: RangeInclusive<u64> = 20..=200_000;

/// Supported file extensions
/// This is a compile time built hashmap to check incomming file extensions against.
/// Follows the extensions supported by currently UI syntax highlighting lib:
/// [`react-syntax-highlighter`](https://gist.github.com/ppisarczyk/43962d06686722d26d176fad46879d41)
//TODO: fill this https://gist.github.com/ppisarczyk/43962d06686722d26d176fad46879d41
const SUPPORTED_FILE_EXTENSIONS: Map<&'static str, &'static str> = phf_map! {
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
    inner: tokio::fs::File,
    path: OsString,
}

/// The file after encryption
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct EncryptedFile {
    hmac_raw: Vec<u8>,
    bytes: Vec<u8>,
    prev: Box<File>,
}

#[async_trait]
pub trait FileReady {
    async fn to_bytes(&self) -> Result<Vec<u8>>;
}

#[async_trait]
impl FileReady for EncryptedFile {
    async fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.bytes.clone())
    }
}

#[async_trait]
impl FileReady for File {
    async fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        let mut file = self.inner.try_clone().await?;
        let _bytes_read = tokio::io::AsyncReadExt::read_to_end(&mut file, &mut buffer).await?;
        Ok(buffer)
    }
}

impl File {
    /// Constructs a [`File`] from a reference to [`super::Action`]
    ///
    /// # Errors
    ///
    /// Fails with [`std::io::Error`]
    pub async fn from_path(path: impl AsRef<OsStr> + Sync + Send) -> Result<Self> {
        let path_ref = path.as_ref();
        let inner = tokio::fs::File::open(path_ref).await?;
        Ok(Self {
            inner,
            path: path_ref.to_os_string(),
        })
    }

    /// Encrypts the file and return it as a byte vector
    ///
    /// # Errors
    ///
    /// Fails with [`Encryption`] with the encryption process goes wrong
    pub async fn into_encrypted(self, key: impl AsRef<str> + Sync + Send) -> Result<EncryptedFile> {
        let file_buf = self.to_bytes().await?;
        let encrypted = cryptor_simple(key.as_ref())
            .into_encryptor()
            .encrypt(&file_buf)?;
        let (hmac_raw, bytes) = (
            encrypted.hmac_raw_default().result().code().to_owned(),
            encrypted.as_bytes().to_owned(),
        );
        Ok(EncryptedFile {
            hmac_raw,
            bytes,
            prev: Box::new(self),
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

#[async_trait]
trait Check {
    /// Checks the file metadata for type and size
    ///
    /// # Errors
    ///
    /// Fails with [`UnsuportedFile`] if size and type isn't allowed.
    async fn metadata(&self) -> Result<()>;

    /// Checks the file extension against [`SUPPORTED_FILE_EXTENSIONS`]
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

        if SUPPORTED_FILE_EXTENSIONS.contains_key(ext) {
            Ok(())
        } else {
            Err(FileError::UnsupportedExtension(ext.to_owned()).into())
        }
    }
}
