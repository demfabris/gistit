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
use crate::{Error, ErrorKind, Result};

/// Allowed file size range in bytes
const ALLOWED_FILE_SIZE_RANGE: RangeInclusive<u64> = 20..=50_000;

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
///
/// `Bat` does autodetection so this doesn't affect it.
///
/// Filled with [Programming languages](https://gist.github.com/ppisarczyk/43962d06686722d26d176fad46879d41)
/// and some google help.
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
    "ahk" => "autohotkey",
    "ahkl" => "autohotkey",
    "au3" => "autoit",
    "sh" => "bash",
    "bash" => "bash",
    "bats" => "bash",
    "cgi" => "bash",
    "command" => "bash",
    "fcgi" => "bash",
    "ksh" => "bash",
    "sh.in" => "bash",
    "tmux" => "bash",
    "tool" => "bash",
    "zsh" => "bash",
    "vb" => "basic",
    "bas" => "basic",
    "cls" => "basic",
    "frm" => "basic",
    "frx" => "basic",
    "vba" => "basic",
    "vbhtml" => "basic",
    "vbs" => "basic",
    "bat" => "batch",
    "cmd" => "batch",
    "bb" => "bbcode",
    "bison" => "bison",
    "b" => "brainfuck",
    "bf" => "brainfuck",
    "brs" => "brightscript",
    "bro" => "bro",
    "bsl" => "bsl",
    "c" => "c",
    "cats" => "c",
    "idc" => "c",
    "w" => "c",
    "cil" => "cil",
    "clike" => "clike",
    "clj" => "clojure",
    "boot" => "clojure",
    "cl2" => "clojure",
    "cljc" => "clojure",
    "cljs" => "clojure",
    "cljs.hl" => "clojure",
    "cljscm" => "clojure",
    "cljx" => "clojure",
    "hic" => "clojure",
    "cmake" => "cmake",
    "cmake.in" => "cmake",
    "coffee" => "coffeescript",
    "_coffee" => "coffeescript",
    "cjsx" => "coffeescript",
    "cson" => "coffeescript",
    "iced" => "coffeescript",
    "conc" => "concurnas",
    "cpp" => "cpp",
    "c++" => "cpp",
    "cc" => "cpp",
    "cp" => "cpp",
    "cxx" => "cpp",
    "h" => "cpp",
    "h++" => "cpp",
    "hh" => "cpp",
    "hpp" => "cpp",
    "hxx" => "cpp",
    "inc" => "cpp",
    "inl" => "cpp",
    "ipp" => "cpp",
    "tcc" => "cpp",
    "tpp" => "cpp",
    "cr" => "crystal",
    "csx" => "csharp",
    "cshtml" => "csharp",
    "csp" => "csp",
    "css" => "css",
    "cypher" => "cypher",
    "d" => "d",
    "di" => "d",
    "dart" => "dart",
    "dax" => "dax",
    "dhall" => "dhall",
    "diff" => "diff",
    "patch" => "diff",
    "mustache" => "django",
    "jinja" => "django",
    "zone" => "dnsZoneFile",
    "arpa" => "dnsZoneFile",
    "dockerfile" => "docker",
    "ebnf" => "ebnf",
    "editorconfig" => "editorconfig",
    "e" => "eiffel",
    "ejs" => "ejs",
    "ex" => "elixir",
    "exs" => "elixir",
    "elm" => "elm",
    "erb" => "erb",
    "erb.deface" => "erb",
    "erl" => "erlang",
    "es" => "erlang",
    "escript" => "erlang",
    "hrl" => "erlang",
    "xrl" => "erlang",
    "yrl" => "erlang",
    "etlua" => "etlua",
    "xls" => "excelFormula",
    "xlsx" => "excelFormula",
    "factor" => "factor",
    "rules" => "firestoreSecurityRules",
    "flow" => "flow",
    "f90" => "fortran",
    "f" => "fortran",
    "f03" => "fortran",
    "f08" => "fortran",
    "f77" => "fortran",
    "f95" => "fortran",
    "for" => "fortran",
    "fpp" => "fortran",
    "fs" => "fsharp",
    "fsi" => "fsharp",
    "fsx" => "fsharp",
    "ftl" => "ftl",
    "g" => "gcode",
    "gco" => "gcode",
    "gcode" => "gcode",
    "gd" => "gdscript",
    "gedcom" => "gedcom",
    "feature" => "gherkin",
    "git" => "git",
    "glsl" => "glsl",
    "fp" => "glsl",
    "frag" => "glsl",
    "frg" => "glsl",
    "fsh" => "glsl",
    "fshader" => "glsl",
    "geo" => "glsl",
    "geom" => "glsl",
    "glslv" => "glsl",
    "gshader" => "glsl",
    "shader" => "glsl",
    "vert" => "glsl",
    "vrx" => "glsl",
    "vsh" => "glsl",
    "vshader" => "glsl",
    "gml" => "gml",
    "go" => "go",
    "graphql" => "graphql",
    "groovy" => "groovy",
    "grt" => "groovy",
    "gtpl" => "groovy",
    "gvy" => "groovy",
    "haml" => "haml",
    "haml.deface" => "haml",
    "handlebars" => "handlebars",
    "hbs" => "handlebars",
    "hs" => "haskell",
    "hsc" => "haskell",
    "hx" => "haxe",
    "hxsl" => "haxe",
    "hcl" => "hcl",
    "tf" => "hcl",
    "hlsl" => "hlsl",
    "fx" => "hlsl",
    "fxh" => "hlsl",
    "hlsli" => "hlsl",
    "hpkp" => "hpkp",
    "hsts" => "hsts",
    "http" => "http",
    "ico" => "icon",
    "ni" => "inform7",
    "i7x" => "inform7",
    "ini" => "ini",
    "cfg" => "ini",
    "prefs" => "ini",
    "pro" => "ini",
    "io" => "io",
    "j" => "j",
    "java" => "java",
    "js" => "javascript",
    "_js" => "javascript",
    "bones" => "javascript",
    "es6" => "javascript",
    "gs" => "javascript",
    "jake" => "javascript",
    "jsb" => "javascript",
    "jscad" => "javascript",
    "jsfl" => "javascript",
    "jsm" => "javascript",
    "jss" => "javascript",
    "njs" => "javascript",
    "pac" => "javascript",
    "sjs" => "javascript",
    "ssjs" => "javascript",
    "sublime-build" => "javascript",
    "sublime-commands" => "javascript",
    "sublime-completions" => "javascript",
    "sublime-keymap" => "javascript",
    "sublime-macro" => "javascript",
    "sublime-menu" => "javascript",
    "sublime-mousemap" => "javascript",
    "sublime-project" => "javascript",
    "sublime-settings" => "javascript",
    "sublime-theme" => "javascript",
    "sublime-workspace" => "javascript",
    "sublime_metrics" => "javascript",
    "sublime_session" => "javascript",
    "xsjs" => "javascript",
    "xsjslib" => "javascript",
    "jolie" => "jolie",
    "jq" => "jq",
    "json" => "json",
    "geojson" => "json",
    "lock" => "json",
    "topojson" => "json",
    "json5" => "json5",
    "jsonp" => "jsonp",
    "jsx" => "jsx",
    "jl" => "julia",
    "keyman" => "keyman",
    "kt" => "kotlin",
    "ktm" => "kotlin",
    "kts" => "kotlin",
    "latex" => "latex",
    "latte" => "latte",
    "less" => "less",
    "ly" => "lilypond",
    "ily" => "lilypond",
    "liquid" => "liquid",
    "nl" => "lisp",
    "lisp" => "lisp",
    "lsp" => "lisp",
    "ls" => "livescript",
    "_ls" => "livescript",
    "ll" => "llvm",
    "lol" => "lolcode",
    "lua" => "lua",
    "nse" => "lua",
    "pd_lua" => "lua",
    "rbxs" => "lua",
    "wlua" => "lua",
    "mak" => "makefile",
    "mk" => "makefile",
    "mkfile" => "makefile",
    "matlap" => "matlab",
    "m" => "matlab",
    "mel" => "mel",
    "mizar" => "mizar",
    "monkey" => "monkey",
    "moon" => "moonscript",
    "n1ql" => "n1ql",
    "n4js" => "n4js",
    "nand2tetrisHdl" => "nand2tetrisHdl",
    "naniscript" => "naniscript",
    "neon" => "neon",
    "nginxconf" => "nginx",
    "nim" => "nim",
    "nimrod" => "nim",
    "nix" => "nix",
    "nsi" => "nsis",
    "nsh" => "nsis",
    "mm" => "objectivec",
    "ml" => "ocaml",
    "eliom" => "ocaml",
    "eliomi" => "ocaml",
    "ml4" => "ocaml",
    "mli" => "ocaml",
    "mll" => "ocaml",
    "mly" => "ocaml",
    "opencl" => "opencl",
    "cl" => "opencl",
    "oz" => "oz",
    "parigp" => "parigp",
    "parser" => "parser",
    "pas" => "pascal",
    "dfm" => "pascal",
    "dpr" => "pascal",
    "ipr" => "pascal",
    "pcaxis" => "pcaxis",
    "peoplecode" => "peoplecode",
    "pl" => "perl",
    "al" => "perl",
    "perl" => "perl",
    "ph" => "perl",
    "plx" => "perl",
    "pm" => "perl",
    "pod" => "perl",
    "psgi" => "perl",
    "6pl" => "perl",
    "6pm" => "perl",
    "nqd" => "perl",
    "p6" => "perl",
    "p6l" => "perl",
    "p6m" => "perl",
    "pm6" => "perl",
    "php" => "php",
    "pls" => "plsql",
    "pck" => "plsql",
    "pkb" => "plsql",
    "pks" => "plsql",
    "plb" => "plsql",
    "plsql" => "plsql",
    "powerquery" => "powerquery",
    "ps1" => "powershell",
    "psd1" => "powershell",
    "psm1" => "powershell",
    "pde" => "processing",
    "prolog" => "prolog",
    "yap" => "prolog",
    "properties" => "properties",
    "proto" => "protobuf",
    "pug" => "pug",
    "pp" => "puppet",
    "pd" => "pure",
    "pb" => "purebasic",
    "pbi" => "purebasic",
    "purs" => "purescript",
    "py" => "python",
    "bzl" => "python",
    "gyp" => "python",
    "lmi" => "python",
    "pyde" => "python",
    "pyp" => "python",
    "pyt" => "python",
    "pyw" => "python",
    "rpy" => "python",
    "tac" => "python",
    "wsgi" => "python",
    "xpy" => "python",
    "q" => "q",
    "qml" => "qml",
    "qbs" => "qml",
    "qore" => "qore",
    "r" => "r",
    "rd" => "r",
    "rsx" => "r",
    "rkt" => "racket",
    "rktd" => "racket",
    "rktl" => "racket",
    "scrbl" => "racket",
    "re" => "reason",
    "regex" => "regex",
    "renpy" => "renpy",
    "rst" => "rest",
    "rest" => "rest",
    "rest.txt" => "rest",
    "rst.txt" => "rest",
    "rip" => "rip",
    "roboconf" => "roboconf",
    "robotframework" => "robotframework",
    "rb" => "ruby",
    "builder" => "ruby",
    "gemspec" => "ruby",
    "god" => "ruby",
    "irbrc" => "ruby",
    "jbuilder" => "ruby",
    "mspec" => "ruby",
    "pluginspec" => "ruby",
    "podspec" => "ruby",
    "rabl" => "ruby",
    "rake" => "ruby",
    "rbuild" => "ruby",
    "rbw" => "ruby",
    "rbx" => "ruby",
    "ru" => "ruby",
    "ruby" => "ruby",
    "thor" => "ruby",
    "watchr" => "ruby",
    "sas" => "sas",
    "sass" => "sass",
    "sbt" => "scala",
    "scala" => "scala",
    "sc" => "scala",
    "scm" => "scheme",
    "sld" => "scheme",
    "sls" => "scheme",
    "sps" => "scheme",
    "ss" => "scheme",
    "scss" => "scss",
    "sh-session" => "shellSession",
    "smali" => "smali",
    "st" => "smalltalk",
    "cs" => "smalltalk",
    "tpl" => "smarty",
    "sml" => "sml",
    "sol" => "solidity",
    "soy" => "soy",
    "sparql" => "sparql",
    "rq" => "sparql",
    "splunk" => "splunkSpl",
    "sqf" => "sqf",
    "hqf" => "sqf",
    "sql" => "sql",
    "cql" => "sql",
    "ddl" => "sql",
    "prc" => "sql",
    "tab" => "sql",
    "udf" => "sql",
    "viw" => "sql",
    "stan" => "stan",
    "styl" => "stylus",
    "swift" => "swift",
    "t4cs" => "t4Cs",
    "t4" => "t4Vb",
    "tap" => "tap",
    "tcl" => "tcl",
    "adp" => "tcl",
    "tm" => "tcl",
    "textile" => "textile",
    "tsx" => "tsx",
    "tt2" => "tt2",
    "ttl" => "turtle",
    "twig" => "twig",
    "typoscript" => "typoscript",
    "uc" => "unrealscript",
    "vala" => "vala",
    "vapi" => "vala",
    "vbnet" => "vbnet",
    "velocity" => "velocity",
    "v" => "verilog",
    "veo" => "verilog",
    "vhdl" => "vhdl",
    "vhd" => "vhdl",
    "vhf" => "vhdl",
    "vhi" => "vhdl",
    "vho" => "vhdl",
    "vhs" => "vhdl",
    "vht" => "vhdl",
    "vhw" => "vhdl",
    "vim" => "vim",
    "warpscript" => "warpscript",
    "wasm" => "wasm",
    "wiki" => "wiki",
    "xeora" => "xeora",
    "xml" => "xmlDoc",
    "ant" => "xmlDoc",
    "axml" => "xmlDoc",
    "ccxml" => "xmlDoc",
    "clixml" => "xmlDoc",
    "cproject" => "xmlDoc",
    "csl" => "xmlDoc",
    "csproj" => "xmlDoc",
    "ct" => "xmlDoc",
    "dita" => "xmlDoc",
    "ditamap" => "xmlDoc",
    "ditaval" => "xmlDoc",
    "dll.config" => "xmlDoc",
    "dotsettings" => "xmlDoc",
    "filters" => "xmlDoc",
    "fsproj" => "xmlDoc",
    "fxml" => "xmlDoc",
    "glade" => "xmlDoc",
    "grxml" => "xmlDoc",
    "iml" => "xmlDoc",
    "ivy" => "xmlDoc",
    "jelly" => "xmlDoc",
    "jsproj" => "xmlDoc",
    "kml" => "xmlDoc",
    "launch" => "xmlDoc",
    "mdpolicy" => "xmlDoc",
    "mod" => "xmlDoc",
    "mxml" => "xmlDoc",
    "nproj" => "xmlDoc",
    "nuspec" => "xmlDoc",
    "odd" => "xmlDoc",
    "osm" => "xmlDoc",
    "plist" => "xmlDoc",
    "props" => "xmlDoc",
    "ps1xml" => "xmlDoc",
    "psc1" => "xmlDoc",
    "pt" => "xmlDoc",
    "rdf" => "xmlDoc",
    "rss" => "xmlDoc",
    "scxml" => "xmlDoc",
    "srdf" => "xmlDoc",
    "storyboard" => "xmlDoc",
    "stTheme" => "xmlDoc",
    "sublime-snippet" => "xmlDoc",
    "targets" => "xmlDoc",
    "tmCommand" => "xmlDoc",
    "tml" => "xmlDoc",
    "tmLanguage" => "xmlDoc",
    "tmPreferences" => "xmlDoc",
    "tmSnippet" => "xmlDoc",
    "tmTheme" => "xmlDoc",
    "ui" => "xmlDoc",
    "urdf" => "xmlDoc",
    "ux" => "xmlDoc",
    "vbproj" => "xmlDoc",
    "vcxproj" => "xmlDoc",
    "vssettings" => "xmlDoc",
    "vxml" => "xmlDoc",
    "wsdl" => "xmlDoc",
    "wsf" => "xmlDoc",
    "wxi" => "xmlDoc",
    "wxl" => "xmlDoc",
    "wxs" => "xmlDoc",
    "x3d" => "xmlDoc",
    "xacro" => "xmlDoc",
    "xaml" => "xmlDoc",
    "xib" => "xmlDoc",
    "xlf" => "xmlDoc",
    "xliff" => "xmlDoc",
    "xmi" => "xmlDoc",
    "xml.dist" => "xmlDoc",
    "xproj" => "xmlDoc",
    "xsd" => "xmlDoc",
    "xul" => "xmlDoc",
    "zcml" => "xmlDoc",
    "xojo_code" => "xojo",
    "xojo_menu" => "xojo",
    "xojo_report" => "xojo",
    "xojo_script" => "xojo",
    "xojo_toolbar" => "xojo",
    "xojo_window" => "xojo",
    "xquery" => "xquery",
    "xq" => "xquery",
    "xql" => "xquery",
    "xqm" => "xquery",
    "xqy" => "xquery",
    "yaml" => "yaml",
    "yml" => "yaml",
    "reek" => "yaml",
    "rviz" => "yaml",
    "yaml-tmlanguage" => "yaml",
    "sublime-syntax" => "yaml",
    "syntax" => "yaml",
    "yang" => "yang",
    "zig" => "zig",
    "txt" => "text",
    "" => "text",
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
    /// File should be discarded when dropped
    is_temp: bool,
}

impl Drop for File {
    fn drop(&mut self) {
        if self.is_temp {
            if let Err(err) = std::fs::remove_file(&self.path) {
                eprintln!(
                    "couldn't delete temp file {:?}\n{}",
                    self.path,
                    err.to_string()
                );
            }
        }
    }
}

#[must_use]
pub fn name_from_path(path: &Path) -> String {
    path.file_name()
        .unwrap_or(OsStr::new("unknown"))
        .to_string_lossy()
        .to_string()
}

async fn rng_file_with_name(bytes: &[u8], name: &str) -> Result<(tokio::fs::File, PathBuf)> {
    let path = rng_temp_file(name);
    let mut handler = tokio::fs::File::create(&path).await?;
    handler.write_all(bytes).await?;
    Ok((handler, path))
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

        tokio::io::AsyncReadExt::read_to_end(&mut handler, &mut bytes)
            .await
            .map_err(|err| {
                // Intercept OS error 21 (EISDIR), meaning we can't write to a directory
                #[cfg(target_family = "unix")]
                if err.raw_os_error() == Some(21) {
                    Error::from(ErrorKind::NotAFile)
                } else {
                    Error::from(err)
                }

                // TODO: This needs testing
                #[cfg(target_os = "windows")]
                if let Some(1003) = err.raw_os_error() {
                    Error::from(ErrorKind::NotAFile)
                } else {
                    Error::from(err)
                }
            })?;

        Ok(Self {
            handler,
            path: path.to_path_buf(),
            bytes,
            name: Some(name_from_path(path)),
            is_temp: false,
        })
    }

    /// Creates a new file in your system `temp` with a random name and writes provided `bytes`
    /// into it. Returns the new [`File`] handler
    ///
    /// # Errors
    ///
    /// Fails with [`IoError`] if the file can't be created for some reason. Also if it can't be
    /// written to.
    pub async fn from_bytes_encoded(bytes: &[u8], name: &str) -> Result<Self> {
        let decoded_bytes = base64::decode(bytes)?;
        let (handler, path) = rng_file_with_name(&decoded_bytes, name).await?;

        Ok(Self {
            handler,
            name: Some(name_from_path(&path)),
            path,
            bytes: decoded_bytes,
            is_temp: true,
        })
    }

    /// Same as [`Self::from_bytes_encoded`] but doesn't expect encoded bytes
    ///
    /// # Errors
    ///
    /// Fails with [`IoError`] if the file can't be created for some reason. Also if it can't be
    /// written to.
    pub async fn from_bytes(decoded_bytes: &[u8]) -> Result<Self> {
        let (handler, path) = rng_file_with_name(decoded_bytes, "").await?;

        Ok(Self {
            handler,
            name: Some(name_from_path(&path)),
            path,
            bytes: decoded_bytes.to_vec(),
            is_temp: true,
        })
    }

    /// Creates/writes the contents as string to the given file path.
    ///
    /// # Errors
    ///
    /// Fails with [`IoError`] if the file can't be written to.
    pub async fn save_as(&self, file_path: &Path) -> Result<()> {
        Ok(tokio::fs::write(file_path, &self.bytes).await?)
    }

    /// Set the file name, useful when creating a [`File`] using [`Self::from_bytes`].
    /// if the [`File`] was created using [`Self::from_path`] it will use the provided file name.
    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_owned());
        self
    }

    /// Returns the file name
    pub fn name(&self) -> String {
        self.name.clone().unwrap_or("unknown".to_owned())
    }

    /// Returns the programming language that maps to this file extension
    pub fn lang(&self) -> &str {
        // SAFETY: If [`Self`] exists, these values are guaranteed to be checked
        unsafe {
            self.path
                .extension()
                .and_then(OsStr::to_str)
                .map(|t| EXTENSION_TO_LANG_MAPPING.get(t))
                .unwrap_unchecked()
                .unwrap_unchecked()
        }
    }

    /// Returns the file size in bytes, not encoded.
    pub async fn size(&self) -> u64 {
        // SAFETY: If [`Self`] exists, these values are guaranteed to be checked
        unsafe { self.handler.metadata().await.unwrap_unchecked().len() }
    }

    /// Consumes the [`File`] encrypting it and returning a new instance of [`EncryptedFile`]
    ///
    /// # Errors
    ///
    /// Fails with [`EncryptionError`] if something goes wrong during the encryption process. This
    /// includes unexpected sizes of the nonce, hashed key.
    /// Will also error out if the provided key and nonce is incorrect.
    pub async fn into_encrypted(self, secret: &str) -> Result<EncryptedFile> {
        let (encrypted_bytes, nonce) = encrypt_aes256_u12nonce(secret.as_bytes(), self.data())?;
        let name = self.name.clone();

        Ok(EncryptedFile {
            encrypted_bytes,
            nonce,
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
/// This data structure is expected to hold encrypted but not encoded bytes in `encrypted_bytes`, the `nonce`
/// which is a 12 bytes randomly generated byte array, and a pointer to the previous unencrypted [`File`]
/// handler.
#[derive(Debug)]
pub struct EncryptedFile {
    /// The encrypted bytes
    encrypted_bytes: Vec<u8>,
    /// The random sequence used to encrypt
    nonce: Vec<u8>,
    /// Pointer to maybe the previous unencrypted
    prev: Option<Box<File>>,
    /// Overwrite the random file name during decryption
    name: Option<String>,
}

/// Extract and verify the encrypted file header which contains the `nonce` and a expected 8 bytes
/// long padding defined in [`FILE_HEADER_ENCRYPTION_PADDING`].
///
/// # Errors
///
/// Fails with [`FileError`] if the padding is invalid or the `nonce` is incorrectly sized.
fn parse_encryption_header(bytes: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    let (header, rest) = bytes.split_at(20);
    let (nonce, padding) = header.split_at(12);

    if padding == FILE_HEADER_ENCRYPTION_PADDING.as_bytes() {
        Ok((nonce.to_vec(), rest.to_vec()))
    } else {
        Err(ErrorKind::EncryptionPadding.into())
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
    pub async fn from_bytes_encoded(encoded_bytes: &[u8], name: &str) -> Result<Self> {
        let decoded_bytes = base64::decode(encoded_bytes)?;
        let path = rng_temp_file(name);

        let (nonce, encrypted_bytes) = parse_encryption_header(&decoded_bytes)?;
        let mut handler = tokio::fs::File::create(&path).await?;
        handler.write_all(&encrypted_bytes).await?;

        Ok(Self {
            encrypted_bytes,
            nonce,
            prev: None,
            name: Some(name_from_path(&path)),
        })
    }

    /// Set a file name, if attempt to decrypt without a file name this will be set to a random
    /// string.
    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_owned());
        self
    }

    /// Converts [`Self`] into [`File`] handler by applying the decryption process with the
    /// provided secret.
    ///
    /// # Errors
    ///
    /// Fails with [`EncryptionError`] if `nonce` or `secret` is incorrect
    pub async fn into_decrypted(self, secret: &str) -> Result<File> {
        // SAFETY: If [`Self`] exists then `self.nonce` is for sure bigger than 12 bytes
        let nonce: [u8; 12] = unsafe { self.nonce.clone().try_into().unwrap_unchecked() };

        let decrypted_bytes = decrypt_aes256_u12nonce(secret.as_bytes(), self.data(), &nonce)?;
        let file = File::from_bytes(&decrypted_bytes).await?.with_name(
            // SAFETY: Opened file will have at least a random temp name
            unsafe { self.name.as_ref().unwrap_unchecked() },
        );
        Ok(file)
    }
}

/// Returns a new randomnly generated file path in your system `temp` directory
fn rng_temp_file(suffix: &str) -> PathBuf {
    let rng_string: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();

    let mut rng_file_name = "__gistit_tmp_".to_owned();
    rng_file_name.push_str(&rng_string);
    rng_file_name.push_str(suffix);

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
        let mut headful_data = self.nonce.clone();
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
            return Err(ErrorKind::FileSize.into());
        } else if !type_allowed {
            return Err(ErrorKind::NotAFile.into());
        }
        Ok(())
    }
    fn extension(&self) -> Result<()> {
        let ext = Path::new(self.path.as_os_str())
            .extension()
            .and_then(OsStr::to_str)
            .ok_or(ErrorKind::FileExtension)?;

        if EXTENSION_TO_LANG_MAPPING.contains_key(ext) {
            Ok(())
        } else {
            Err(ErrorKind::FileExtension.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;
    use predicates::prelude::*;
    use tokio::task::spawn_blocking;

    #[test]
    fn file_name_from_path_edge_cases() {
        let n1 = name_from_path(Path::new("foo.txt"));
        let n2 = name_from_path(Path::new("~/foo.txt"));
        let n3 = name_from_path(Path::new("/foo.txt"));
        let n4 = name_from_path(Path::new("/😁.txt"));
        let n5 = name_from_path(Path::new("/lol/what/foo/😁.txt"));
        let n6 = name_from_path(Path::new("😁"));

        assert_eq!(n1, "foo.txt");
        assert_eq!(n2, "foo.txt");
        assert_eq!(n3, "foo.txt");
        assert_eq!(n4, "😁.txt");
        assert_eq!(n5, "😁.txt");
        assert_eq!(n6, "😁");
    }

    #[tokio::test]
    async fn file_spawn_random_and_write() {
        let data: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(512)
            .map(char::from)
            .collect();
        let (file, path) = rng_file_with_name(data.as_bytes(), "foo.txt")
            .await
            .unwrap();
        let read_bytes = tokio::fs::read(path).await.unwrap();
        assert_eq!(data, std::str::from_utf8(&read_bytes).unwrap());
    }

    #[tokio::test]
    async fn file_structure_new_from_path() {
        let data: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(512)
            .map(char::from)
            .collect();
        let tmp = assert_fs::TempDir::new().unwrap();
        let input_file = tmp.child("foo.txt");

        input_file.touch().unwrap();
        input_file.write_binary(data.as_bytes()).unwrap();

        let file = File::from_path(&input_file).await.unwrap();
        assert_eq!(file.size().await, 512);
        assert_eq!(file.bytes, data.as_bytes());
        assert_eq!(file.name(), "foo.txt".to_owned());
    }

    #[tokio::test]
    async fn file_structure_new_from_path_fails_if_is_dir() {
        let tmp = assert_fs::TempDir::new().unwrap();
        let read_err = File::from_path(&tmp).await.unwrap_err();
        assert!(matches!(read_err.kind, ErrorKind::NotAFile));
    }

    #[tokio::test]
    async fn file_structure_new_from_bytes_encoded() {
        let data: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(512)
            .map(char::from)
            .collect();
        let encoded_data = base64::encode(&data);
        let file = File::from_bytes_encoded(encoded_data.as_bytes(), "nameless")
            .await
            .unwrap();
        assert_eq!(file.bytes, data.as_bytes());

        // Fails when encoding is corrupted
        let mut corrupted_data = "¨¨¨¨".to_owned();
        corrupted_data.extend(encoded_data.chars());
        let decode_err = File::from_bytes_encoded(corrupted_data.as_bytes(), "nameless")
            .await
            .unwrap_err();
        assert!(matches!(decode_err.kind, ErrorKind::Encoding(_)));
    }

    #[tokio::test]
    async fn file_structure_new_from_bytes() {
        let data: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(512)
            .map(char::from)
            .collect();
        let file = File::from_bytes(data.as_bytes()).await.unwrap();
        assert_eq!(file.bytes, data.as_bytes());
    }

    #[tokio::test]
    async fn file_structure_save_as() {
        let data: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(512)
            .map(char::from)
            .collect();
        let tmp = assert_fs::TempDir::new().unwrap();
        let tmp_file = tmp.child("foo.txt");

        tmp_file.touch().unwrap();
        tmp_file.write_binary(data.as_bytes()).unwrap();

        let file = File::from_path(&tmp_file).await.unwrap();
        file.save_as(&tmp.join("bar.txt")).await.unwrap();
        tmp.assert(predicates::path::exists());

        let other = tmp.child("bar.txt");
        let other = tokio::fs::read(other).await.unwrap();
        assert_eq!(data.as_bytes(), other);
    }

    #[tokio::test]
    async fn file_structure_extension_to_lang_mapping() {
        let tmp = assert_fs::TempDir::new().unwrap();

        let rust = tmp.child("foo.rs");
        let js = tmp.child("bar.js");
        let cpp = tmp.child("lol.cpp");
        let brainfuck = tmp.child("rly.bf");
        rust.touch().unwrap();
        js.touch().unwrap();
        cpp.touch().unwrap();
        brainfuck.touch().unwrap();

        assert_eq!(File::from_path(&rust).await.unwrap().lang(), "rust");
        assert_eq!(File::from_path(&js).await.unwrap().lang(), "javascript");
        assert_eq!(File::from_path(&cpp).await.unwrap().lang(), "cpp");
        assert_eq!(
            File::from_path(&brainfuck).await.unwrap().lang(),
            "brainfuck"
        );
    }

    #[tokio::test]
    async fn file_structure_support_methods() {
        let data: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(512)
            .map(char::from)
            .collect();
        let tmp = assert_fs::TempDir::new().unwrap();
        let tmp_file = tmp.child("foo");
        tmp_file.touch().unwrap();
        tmp_file.write_binary(data.as_bytes()).unwrap();

        let file = File::from_path(&tmp_file).await.unwrap();
        assert_eq!(file.name(), "foo");
        let file = file.with_name("bar");
        assert_eq!(file.name(), "bar");
        assert_eq!(file.size().await, 512);
    }

    #[tokio::test]
    async fn file_structure_into_encrypted() {
        let secret = "secret";
        let data: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(512)
            .map(char::from)
            .collect();
        let tmp = assert_fs::TempDir::new().unwrap();
        let tmp_file = tmp.child("foo");
        tmp_file.touch().unwrap();
        tmp_file.write_binary(data.as_bytes()).unwrap();

        let file = Box::new(File::from_path(&tmp_file).await.unwrap());
        let encrypted = File::from_path(&tmp_file)
            .await
            .unwrap()
            .into_encrypted(secret)
            .await
            .unwrap();
        // original file is intact
        tmp_file.assert(predicates::str::contains(&data));
        assert_ne!(file.data(), encrypted.data());
        // file prev encrypted is the same decrypted one
        assert_eq!(file.data(), encrypted.prev.unwrap().data());
    }

    #[tokio::test]
    async fn file_encrypted_structure_from_bytes_encoded() {
        let secret = "secret";
        let data = "I'm a decrypted string".to_owned();
        let tmp = assert_fs::TempDir::new().unwrap();
        let tmp_file = tmp.child("foo");
        tmp_file.touch().unwrap();
        tmp_file.write_binary(data.as_bytes()).unwrap();

        let encoded_data = File::from_path(&tmp_file)
            .await
            .unwrap()
            .into_encrypted(secret)
            .await
            .unwrap()
            .to_encoded_data();
        let encrypted =
            EncryptedFile::from_bytes_encoded(encoded_data.inner.as_bytes(), "nameless")
                .await
                .unwrap();
        assert_ne!(encrypted.data(), data.as_bytes());
        assert!(matches!(encrypted.prev, None));
    }

    #[tokio::test]
    async fn file_encrypted_structure_into_decrypted() {
        let secret = "secret";
        let data = "I'm a decrypted string".to_owned();
        let tmp = assert_fs::TempDir::new().unwrap();
        let tmp_file = tmp.child("foo");

        tmp_file.touch().unwrap();
        tmp_file.write_binary(data.as_bytes()).unwrap();

        let encrypted_file = File::from_path(&tmp_file)
            .await
            .unwrap()
            .into_encrypted(secret)
            .await
            .unwrap();
        let encoded_encrypted_data = encrypted_file.to_encoded_data();
        let decrypted_file = encrypted_file.into_decrypted(secret).await.unwrap();

        assert_eq!(decrypted_file.data(), data.as_bytes());
    }

    #[tokio::test]
    async fn file_encryption_header_data() {
        let secret = "secret";
        let data = "Matthew McConaughey".to_owned();
        let tmp = assert_fs::TempDir::new().unwrap();
        let tmp_file = tmp.child("foo");

        tmp_file.touch().unwrap();
        tmp_file.write_binary(data.as_bytes()).unwrap();

        let encoded_encrypted_data = File::from_path(&tmp_file)
            .await
            .unwrap()
            .into_encrypted(secret)
            .await
            .unwrap()
            .to_encoded_data();
        let (nonce, rest) = parse_encryption_header(
            base64::decode(encoded_encrypted_data.inner)
                .unwrap()
                .as_slice(),
        )
        .unwrap();
        let nonce: [u8; 12] = nonce.try_into().unwrap();
        let decrypted_data = decrypt_aes256_u12nonce(secret.as_bytes(), &rest, &nonce).unwrap();
        assert_eq!(decrypted_data, data.as_bytes());
    }

    #[tokio::test]
    async fn file_temp_fs_file_deleted_on_drop() {
        let data = "Matthew McConaughey".to_owned();
        let file = File::from_bytes(data.as_bytes()).await.unwrap();
        let path = file.path.clone();
        assert!(tokio::fs::metadata(&path).await.unwrap().is_file());
        {
            file;
        }
        let not_found = tokio::fs::metadata(&path).await.unwrap_err().kind();
        assert!(matches!(not_found, std::io::ErrorKind::NotFound));
    }
}
