<p align="center">
  <img
    width="200"
    src="https://user-images.githubusercontent.com/46208058/145101071-d186a89d-0193-4deb-acfb-ecc93e172943.png"
    alt="Gistit - Share user snippets"
  />
</p>
<h3 align="center">⚡️ Quick and easy <code>code</code> snippet sharing tool</h3>
<h1></h1>
<p align="center">
  <a href="https://crates.io/crates/gistit/"
    ><img
      src="https://img.shields.io/crates/d/gistit?style=flat-square"
      alt="Crates.io"
  /></a>
    <a href="https://crates.io/crates/gistit/"
    ><img
      src="https://img.shields.io/crates/v/gistit?style=flat-square"
      alt="Crates.io"
  /></a>
</p>

A feature packed, hash based `code` snippet sharing tool focused on ease of use and simplicity.

## :star: Features

<img
  src="https://user-images.githubusercontent.com/46208058/152258956-fa9f685f-637e-462c-8708-35b54a925f7a.gif"
  alt="send and fetch gif"
  align="right"
  width="60%"
/>

- **TUI support** - send and preview gistits without leaving the terminal. _(uses [bat](https://github.com/sharkdp/bat))_ :bat:
- **Easy to use** - command line API made for humans, shell completion and fancy spinners. :man_artist:
- **Open source** - Independent web application and server, open source top to bottom.
- **Integrated** - Integration with GitHub Gists.
- **Handy** - system clipboard integration that actually works.
- **Trendy** - peer to peer file sharing through [IPFS](https://ipfs.io/) _(uses [libp2p](https://github.com/libp2p/rust-libp2p))_. :globe_with_meridians: **Soon**

### Feature requests

[I want a feature](https://github.com/fabricio7p/gistit/issues/new)
_Windows support comming soon_

## CLI

### Basic Usage

You can send a local file or stdin.

```shell
# Local file
$ gistit myfile.txt

# Stdin
$ ls | gistit

# Additional info
$ ls | gistit -a "Matthew McConaughey" -d "My ls, lol"
```

Post to GitHub Gists.

```shell
$ gistit myfile.txt --github
# A browser window will open to authorize Github OAuth.
# Hit **authorize** and wait for the CLI to resume automatically.
```

Copy hash to system clipboard.

```shell
$ gistit myfile.txt -c
# Hash is now on your <Ctrl-V>
```

Fetching gistits

```shell
# Fetch and preview
$ gistit f 8765d324ddd800f1112e77fece3d3ff2

# Fetch and save to local data directory
$ gistit f 8765d324ddd800f1112e77fece3d3ff2 --save
```

## Installation

**Compiled binaries**
_soon_

**With** _\<Insert package manager here\>_
_soon_

**From [crates.io](https://crates.io/crates/gistit/)**

```shell
cargo install gistit
```

**From source** _(msrv 1.58)_

```shell
# Clone
$ git clone https://github.com/fabricio7p/gistit.git

# Move
$ cd gistit

# Build
$ cargo build --release
```

Your binary will be inside `target/release` folder.

## License

Licensed under either of [MIT](https://choosealicense.com/licenses/mit) or [Apache-2.0](https://github.com/dtolnay/cargo-expand/blob/master/LICENSE-APACHE) at your option.
