//! The file module

use async_trait::async_trait;
use phf::{phf_map, Map};
use std::ffi::{OsStr, OsString};
use std::path::Path;

use crate::{Error, Result};

#[cfg(doc)]
use std::io::ErrorKind;

/// Max file size in bytes
const MAX_FILE_SIZE: u64 = 200_000;
/// Min file size in bytes
const MIN_FILE_SIZE: u64 = 20;

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

    /// Access the file in read mode and dump contents in a buffer
    ///
    /// # Errors
    ///
    /// Fails with [`std::io::Error`]
    pub async fn as_buf(&mut self) -> Result<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        let bytes_read = tokio::io::AsyncReadExt::read_to_end(&mut self.inner, &mut buffer).await?;
        log::debug!("read {} bytes from file", bytes_read);
        Ok(buffer)
    }

    pub async fn as_buf_encrypt(&mut self) -> Result<Vec<u8>> {
        todo!()
    }

    /// Perform needed checks concurrently, consumes `Self` and return.
    ///
    /// # Errors
    ///
    /// Fails with [`UnsuportedFile`]
    pub async fn check_consume(self) -> Result<Self> {
        let _ = tokio::try_join! {
            <Self as Check>::metadata(&self),
            <Self as Check>::extension(&self)
        }?;
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
    async fn extension(&self) -> Result<()>;
}

#[async_trait]
impl Check for File {
    async fn metadata(&self) -> Result<()> {
        let attr = self.inner.metadata().await?;
        let size_allowed = (MIN_FILE_SIZE..=MAX_FILE_SIZE).contains(&attr.len());
        let type_allowed = attr.is_file();

        if !size_allowed {
            return Err(Error::UnsuportedFile {
                message: UNSUPPORTED_FILE_SIZE.to_owned(),
            });
        } else if !type_allowed {
            return Err(Error::UnsuportedFile {
                message: UNSUPPORTED_FILE_TYPE.to_owned(),
            });
        }
        Ok(())
    }
    async fn extension(&self) -> Result<()> {
        let ext = Path::new(self.path.as_os_str())
            .extension()
            .and_then(OsStr::to_str)
            .ok_or(Error::UnsuportedFile {
                message: UNSUPPORTED_FILE_HAS_EXTENSION.to_owned(),
            })?;

        if SUPPORTED_FILE_EXTENSIONS.contains_key(ext) {
            log::trace!("File ext: {}", ext);
            Ok(())
        } else {
            Err(Error::UnsuportedFile {
                message: UNSUPPORTED_FILE_EXTENSION.to_owned(),
            })
        }
    }
}

#[doc(hidden)]
const UNSUPPORTED_FILE_SIZE: &str = "file size is not in allowed range. MIN = 20bytes MAX = 200kb";

#[doc(hidden)]
const UNSUPPORTED_FILE_TYPE: &str = "input is not a file";

#[doc(hidden)]
const UNSUPPORTED_FILE_HAS_EXTENSION: &str = "provided file must have an extension";

#[doc(hidden)]
const UNSUPPORTED_FILE_EXTENSION: &str = "file extension not currently supported";
