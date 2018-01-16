# Codesign [![Build status](https://ci.appveyor.com/api/projects/status/ollapsvtxks5ngu5?svg=true)](https://ci.appveyor.com/project/forbjok/rust-codesign) [![Crates.io](https://img.shields.io/crates/v/codesign.svg)](https://crates.io/crates/codesign)

Microsoft code signing library (and utility) for Rust.

This library is a convenience wrapper around Microsoft's signing tool and requires the Windows SDK to be installed.

It provides a simple way to sign Windows binaries without having to manually mess with figuring out where signtool.exe is located or which one to use, which can be a bit of a pain due to it changing with pretty much every new Windows SDK version. Currently all versions of the Windows 10 SDK are supported, and the newest one installed will be used.

## How to use the library

```rust
// Locate signing tool
let signtool = match SignTool::locate_latest().unwrap();

// Set up signing parameters
let sign_params = SignParams {
    digest_algorithm: "sha256".to_owned(),
    certificate_thumbprint: "<your certificate sha-1 thumbprint here>".to_owned(),
    timestamp_url: Some("<timestamp server url here>".to_owned()),
};

// Sign yourapp.exe
signtool.sign("yourapp.exe", &sign_params).unwrap();
```

## How to use the commandline utility

```
> codesign.exe -c <your certificate sha-1 thumbprint here> yourapp.exe
```
