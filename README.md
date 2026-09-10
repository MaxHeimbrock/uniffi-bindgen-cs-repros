# uniffi-bindgen-cs: multi-crate reproductions

Independent, self-contained reproductions of bugs in the C# that
[uniffi-bindgen-cs](https://github.com/NordSecurity/uniffi-bindgen-cs) generates in library mode when the
library contains two UniFFI crates and one uses a type defined in the other. The generated code is used
as-is, without post-processing. Each folder can be copied out as its own repository.

| folder | bug |
|--------|-----|
| [`issue1_external-custom-type-not-aliased/`](issue1_external-custom-type-not-aliased/README.md) | a custom type of another crate is never aliased in the consuming file (CS0246) |
| [`issue2_external-object-converter-wrong-stream/`](issue2_external-object-converter-wrong-stream/README.md) | the converter of another crate's object is passed the wrong crate's `BigEndianStream` (CS1503) |

Folders are named `issueN_<what-goes-wrong>`; each folder's README describes the bug, the generated code
and a hand-verified fix.

Both follow the same layout: `crate_a` defines the type, `crate_b` uses it in its exported API and is the
cdylib, `csharp/` is a minimal .NET console project that compiles the generated bindings, and `build.sh`
builds the dylib, runs `uniffi-bindgen-cs --library` into `csharp/Generated/` and runs `dotnet build`.

## Prerequisites

- Rust toolchain (tested with 1.94) on macOS; `build.sh` looks for `.dylib`, adjust for other OSes
- `cargo install uniffi-bindgen-cs --git https://github.com/NordSecurity/uniffi-bindgen-cs --tag v0.11.0+v0.31.0`
- .NET SDK 10 (change `TargetFramework` in `csharp/*.csproj` for another version)

## Run

```
(cd issue1_external-custom-type-not-aliased && ./build.sh)
(cd issue2_external-object-converter-wrong-stream && ./build.sh)
```
