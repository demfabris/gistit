# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [Unreleased] - ReleaseDate
# Added
- Add `gistit-reference`
- Add `gistit-proto`
- Add `gistit` (install crate)
- P2p file sharing working
- More cli flags (`host`, `port`, `dial`)

# Changed
- BREAKING: Gistit hash is now 64bits (sha256)
- BREAKING: Moved to protobuf encodings

- Use `tokio::UnixDatagram` in gistit-ipc
- Refactors to `gistit-daemon` to be more independent
- Refactor `gistit-cli`, `gistit-ipc`, and `gistit-daemon` to support protobuf
  encodings
- Inner file handler now only support UTF-8 data


## [0.1.51] - 2022-02-03
# Added
- Add code checking workflow
- Add release assets workflow
- Add /hash page to web app
