# gistit-cli

**Multi platform command line application to share `code` snippets**

<p>
  <a href="#"
    ><img
      src="https://img.shields.io/crates/d/gistit?style=flat-square"
      alt="Crates.io"
  /></a>
    <a href="#"
    ><img
      src="https://img.shields.io/crates/v/gistit?style=flat-square"
      alt="Crates.io"
  /></a>
</p>

## Subcommands

gistit-cli has two main subcommands:

### gistit-send

Send `code` snippets to the cloud

![image](https://user-images.githubusercontent.com/46208058/145119185-e723df65-7fa8-4674-8d27-4bc16ec7bb4d.png)

### gistit-fetch

Fetch an external gistit, preview it or save to local filesystem.

![image](https://user-images.githubusercontent.com/46208058/145119320-45ef27ce-cdd1-4209-9030-1a6b18eeb899.png)

## Features

- ### Send

| flag/argument    | value         | does                               |
| :--------------- | :------------ | :--------------------------------- |
| -c --clipboard   | -             | Copies generated hash to clipboard |
| -a --author      | _author name_ | Append an author name              |
| -d --description | _description_ | Append a description               |
| -s --secret      | _secret key_  | Encrypts the gistit with a secret  |
| -t --theme       | _colorscheme_ | Changes the default colorscheme    |

- ### Fetch

| flag/argument | value         | does                                  |
| :------------ | :------------ | :------------------------------------ |
| -x --hash     | _gistit hash_ | Fetches the gistit via its hash       |
| -u --url      | _gistit url_  | Fetches the gistit via its url        |
| -t --theme    | _colorscheme_ | Overwrites the suggested colorscheme  |
| -s --secret   | _secret key_  | Decrypts the protected gistit         |
| --save        | -             | Immediately save to fs after fetching |
| --preview     | -             | Immediately preview after fetching    |

## Installation

_Wait for compiled binaries_

Building from source:

```shell
# Grab gistit source code
$ git clone https://github.com/fabricio7p/gistit.git

# move into /cli folder
$ cd cli

# build
$ cargo build --release
```

Your binary will be in `/target/release` folder

### From [crates.io](https://crates.io/crates/gistit/)

```shell
$ cargo install gistit
```
