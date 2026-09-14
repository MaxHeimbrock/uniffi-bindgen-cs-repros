# uniffi-bindgen-cs: multi-crate reproductions

Independent, self-contained reproductions of bugs in the C# that I found in 
[uniffi-bindgen-cs](https://github.com/NordSecurity/uniffi-bindgen-cs).

| folder | bug | 
|--------|-----|
| [`issue1_external-custom-type-not-aliased/`](issue1_external-custom-type-not-aliased/README.md) | a custom type of another crate is never aliased in the consuming file (CS0246). Root cause described in [#40](https://github.com/NordSecurity/uniffi-bindgen-cs/issues/40) | 
| [`issue2_external-object-converter-wrong-stream/`](issue2_external-object-converter-wrong-stream/README.md) | the converter of another crate's object is passed the wrong crate's `BigEndianStream` (CS1503) | 

## Prerequisites

- Rust toolchain (tested with 1.94); `build.sh` picks the shared-library extension for macOS, Linux and Windows
  (verified on macOS)
- `cargo install uniffi-bindgen-cs --git https://github.com/NordSecurity/uniffi-bindgen-cs --tag v0.11.0+v0.31.0`
- .NET SDK 10 (change `TargetFramework` in `csharp/*.csproj` for another version)

## Run

```
(cd issue1_external-custom-type-not-aliased && ./build.sh)
(cd issue2_external-object-converter-wrong-stream && ./build.sh)
```
