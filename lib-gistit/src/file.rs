//! The file module
//!
//! Here we define file structures and methods. It is implemented using [`tokio`] so we don't block
//! progress output during the process.

use std::env::temp_dir;
use std::ffi::OsStr;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};
use std::str;

use phf::{phf_map, Map};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

use crate::Result;

/// Allowed file size range in bytes
const ALLOWED_FILE_SIZE_RANGE: RangeInclusive<u64> = 20..=50_000;

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

#[derive(Debug)]
pub struct File {
    inner: fs::File,
    path: PathBuf,
    bytes: Vec<u8>,
    size: usize,
    temp: bool,
}

impl Drop for File {
    fn drop(&mut self) {
        if self.temp {
            fs::remove_file(&self.path).expect("failed to remove temp file");
        }      
    }
}

pub fn name_from_path(path: &Path) -> String {
    path.file_name()
        .unwrap_or(OsStr::new("unknown"))
        .to_string_lossy()
        .to_string()
}

fn spawn_from_bytes(bytes: &[u8], name: &str) -> Result<(fs::File, PathBuf)> {
    let path = rng_temp_file(name);
    let mut handler = fs::File::create(&path)?;
    handler.write_all(bytes)?;
    Ok((handler, path))
}

fn rng_temp_file(suffix: &str) -> PathBuf {
    let rng_string: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();

    let mut rng_name = "__gistit-".to_owned();
    rng_name.push_str(&rng_string);
    rng_name.push_str(suffix);

    temp_dir().join(&rng_name)
}

impl File {
    pub fn from_path(path: &Path) -> Result<Self> {
        let mut handler = fs::File::open(path)?;

        let mut buf: Vec<u8> = Vec::new();
        let size = handler.read(&mut buf)?;

        Ok(Self {
            inner: handler,
            path: path.to_path_buf(),
            bytes: buf,
            size,
            temp: false,
        })
    }

    pub fn from_bytes(decoded_bytes: Vec<u8>, name: &str) -> Result<Self> {
        let (handler, path) = spawn_from_bytes(&decoded_bytes, name)?;
        let size = decoded_bytes.len();

        Ok(Self {
            inner: handler,
            path: path.to_path_buf(),
            bytes: decoded_bytes,
            size,
            temp: true,
        })
    }

    pub fn from_bytes_encoded(bytes: &[u8], name: &str) -> Result<Self> {
        let decoded_bytes = base64::decode(bytes)?;
        File::from_bytes(decoded_bytes, name)
    }

    pub fn inner(&self) -> &fs::File {
        &self.inner
    }

    pub fn data(&self) -> &[u8] {
        &self.bytes
    }

    pub fn to_encoded_data(&self) -> EncodedFileData {
        EncodedFileData {
            inner: base64::encode(&self.bytes),
        }
    }

    pub fn save_as(&self, file_path: &Path) -> Result<()> {
        Ok(fs::write(file_path, &self.bytes)?)
    }

    pub fn name(&self) -> String {
        name_from_path(&self.path)
    }

    pub fn lang(&self) -> &str {
        //TODO: Refactor this after removing Check trait
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

    pub fn size(&self) -> usize {
        self.size
    }

    // /// Perform needed checks concurrently, consumes `Self` and return.
    // ///
    // /// # Errors
    // ///
    // /// Fails with [`FileError`] if user input isn't valid
    // pub async fn check_consume(self) -> Result<Self> {
    //     <Self as Check>::metadata(&self).await?;
    //     <Self as Check>::extension(&self)?;
    //
    //     Ok(self)
    // }
}

// #[async_trait]
// trait Check {
//     /// Checks the file metadata for type and size
//     ///
//     /// # Errors
//     ///
//     /// Fails with [`UnsuportedFile`] if size and type isn't allowed.
//     async fn metadata(&self) -> Result<()>;
//
//     /// Checks the file extension against [`EXTENSION_TO_LANG_MAPPING`]
//     ///
//     /// # Errors
//     ///
//     /// Fails with [`UnsuportedFile`] if file extension isn't supported.
//     fn extension(&self) -> Result<()>;
// }
//
// #[async_trait]
// impl Check for File {
//     async fn metadata(&self) -> Result<()> {
//         let attr = self.handler.metadata().await?;
//         let size_allowed = ALLOWED_FILE_SIZE_RANGE.contains(&attr.len());
//         let type_allowed = attr.is_file();
//
//         if !size_allowed {
//             return Err(ErrorKind::FileSize.into());
//         } else if !type_allowed {
//             return Err(ErrorKind::NotAFile.into());
//         }
//         Ok(())
//     }
//     fn extension(&self) -> Result<()> {
//         let ext = Path::new(self.path.as_os_str())
//             .extension()
//             .and_then(OsStr::to_str)
//             .ok_or(ErrorKind::FileExtension)?;
//
//         if EXTENSION_TO_LANG_MAPPING.contains_key(ext) {
//             Ok(())
//         } else {
//             Err(ErrorKind::FileExtension.into())
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ErrorKind;
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
        let (file, path) = spawn_from_bytes(data.as_bytes(), "foo.txt").unwrap();
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

        let file = File::from_path(&input_file).unwrap();
        assert_eq!(file.size(), 512);
        assert_eq!(file.bytes, data.as_bytes());
        assert_eq!(file.name(), "foo.txt".to_owned());
    }

    #[tokio::test]
    async fn file_structure_new_from_path_fails_if_is_dir() {
        let tmp = assert_fs::TempDir::new().unwrap();
        let read_err = File::from_path(&tmp).unwrap_err();

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
        let file = File::from_bytes_encoded(encoded_data.as_bytes(), "nameless").unwrap();

        assert_eq!(file.bytes, data.as_bytes());

        // Fails when encoding is corrupted
        let mut corrupted_data = "¨¨¨¨".to_owned();
        corrupted_data.extend(encoded_data.chars());
        let decode_err =
            File::from_bytes_encoded(corrupted_data.as_bytes(), "nameless").unwrap_err();

        assert!(matches!(decode_err.kind, ErrorKind::Encoding(_)));
    }

    #[tokio::test]
    async fn file_structure_new_from_bytes() {
        let data: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(512)
            .map(char::from)
            .collect();
        let file = File::from_bytes(data.as_bytes().to_owned(), "nameless").unwrap();

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

        let file = File::from_path(&tmp_file).unwrap();
        file.save_as(&tmp.join("bar.txt")).unwrap();
        tmp.assert(predicates::path::exists());

        let other = tmp.child("bar.txt");
        let other = fs::read(other).unwrap();

        assert_eq!(data.as_bytes(), other);
    }

    #[tokio::test]
    async fn file_structure_extension_to_lang_mapping() {
        let tmp = assert_fs::TempDir::new().unwrap();

        let rust = tmp.child("foo.rs");
        rust.touch().unwrap();

        let js = tmp.child("bar.js");
        js.touch().unwrap();

        let cpp = tmp.child("lol.cpp");
        cpp.touch().unwrap();

        let brainfuck = tmp.child("rly.bf");
        brainfuck.touch().unwrap();

        assert_eq!(File::from_path(&rust).unwrap().lang(), "rust");
        assert_eq!(File::from_path(&js).unwrap().lang(), "javascript");
        assert_eq!(File::from_path(&cpp).unwrap().lang(), "cpp");
        assert_eq!(File::from_path(&brainfuck).unwrap().lang(), "brainfuck");
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

        let file = File::from_path(&tmp_file).unwrap();

        assert_eq!(file.name(), "foo");
        assert_eq!(file.size(), 512);
    }

    #[tokio::test]
    async fn file_temp_fs_file_deleted_on_drop() {
        let data = "Matthew McConaughey".to_owned();
        let file = File::from_bytes(data.as_bytes().to_owned(), "nameless").unwrap();
        let path = file.path.clone();

        assert!(fs::metadata(&path).unwrap().is_file());

        drop(file);
        let not_found = fs::metadata(&path).unwrap_err().kind();

        assert!(matches!(not_found, std::io::ErrorKind::NotFound));
    }
}
