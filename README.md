# uniffi-bindgen-cs: multi-crate reproductions

Independent, self-contained reproductions of bugs in the C# that
[uniffi-bindgen-cs](https://github.com/NordSecurity/uniffi-bindgen-cs) generates in library mode when the
library contains two UniFFI crates and one uses a type defined in the other. The generated code is used
as-is, without post-processing. Each folder can be copied out as its own repository.

| folder | bug | upstream |
|--------|-----|----------|
| [`issue1_external-custom-type-not-aliased/`](issue1_external-custom-type-not-aliased/README.md) | a custom type of another crate is never aliased in the consuming file (CS0246) | root cause described in [#40](https://github.com/NordSecurity/uniffi-bindgen-cs/issues/40); fix PR pending |
| [`issue2_external-object-converter-wrong-stream/`](issue2_external-object-converter-wrong-stream/README.md) | the converter of another crate's object is passed the wrong crate's `BigEndianStream` (CS1503) | issue pending; fix PR pending |

Folders are named `issueN_<what-goes-wrong>`; each folder's README describes the bug, the generated code
and a hand-verified fix. `N` is a local sequence number, not an upstream issue number; the upstream column
above links the tracking issue and the fix PR.

Both follow the same layout: `crate_a` defines the type, `crate_b` uses it in its exported API and is the
cdylib, `csharp/` is a minimal .NET console project that compiles the generated bindings, and `build.sh`
builds the shared library, runs `uniffi-bindgen-cs --library` into `csharp/Generated/` and runs `dotnet build`.

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
