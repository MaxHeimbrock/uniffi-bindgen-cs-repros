# Issue 1: custom type of another crate is never aliased in the consuming file

Upstream: root cause described in [NordSecurity/uniffi-bindgen-cs#40](https://github.com/NordSecurity/uniffi-bindgen-cs/issues/40); fix proposed in [PR #185](https://github.com/NordSecurity/uniffi-bindgen-cs/pull/185).

Minimal, self-contained reproduction against
[uniffi-bindgen-cs](https://github.com/NordSecurity/uniffi-bindgen-cs) `v0.11.0+v0.31.0`, uniffi `0.31`.
The generated code is used as-is.

| crate     | role                                                               |
|-----------|--------------------------------------------------------------------|
| `crate_a` | defines `Blob`, a `uniffi::custom_type!` over `Vec<u8>`            |
| `crate_b` | uses `Blob` in two exported functions; the cdylib that bindgen reads |

## Run

```
./build.sh
```

Builds `target/release/libcrate_b.dylib` (`.so` on Linux, `crate_b.dll` on Windows), runs `uniffi-bindgen-cs --library` into `csharp/Generated/`
(one unmodified `.cs` file per crate), copies the library to `csharp/native/` so `dotnet run` can load it
later, and runs `dotnet build csharp`. Expected:

```
csharp/Generated/crate_b.cs(1475,52): error CS0246: The type or namespace name 'Blob' could not be found (are you missing a using directive or an assembly reference?)
   (5 occurrences)
```

## What bindgen generates

uniffi-bindgen-cs represents a custom type as a `using` alias. `crate_a.cs` gets, right after its
namespace declaration:

```csharp
namespace uniffi.crate_a;

using Blob = byte[];
using FfiConverterTypeBlob = FfiConverterByteArray;
```

`crate_b.cs` imports the namespace and uses the name `Blob` in its signatures and in the forwarding
converter that `ExternalTypeTemplate.cs` generates, but declares no alias. C# `using` aliases are per
file, so `Blob` is unknown here:

```csharp
using uniffi.crate_a;
namespace uniffi.crate_b;
// no `using Blob = ...;`

class FfiConverterTypeBlob: FfiConverterRustBuffer<Blob> {
    public static FfiConverterTypeBlob INSTANCE = new FfiConverterTypeBlob();

    public override Blob Read(BigEndianStream stream) {
        return uniffi.crate_a.FfiConverterTypeBlob.INSTANCE.Read(
            new uniffi.crate_a.BigEndianStream(stream.InnerStream)
        );
    }
    ...
}

public static Blob MakeBlob(uint @len) { ... }
```

## Second facet: the converter is an alias as well

`uniffi.crate_a.FfiConverterTypeBlob` in the snippet above is not a class either, only the alias shown in
`crate_a.cs`, so it cannot be referenced from another file. Add `using Blob = byte[];` to `crate_b.cs`
and the build fails with:

```
csharp/Generated/crate_b.cs(1480,16): error CS0234: The type or namespace name 'FfiConverterTypeBlob' does not exist in the namespace 'uniffi.crate_a' (are you missing an assembly reference?)
   (3 occurrences)
```

## With a custom-type config

`BINDGEN_CONFIG=uniffi.toml ./build.sh` maps `Blob` to `System.ReadOnlyMemory<byte>`. `crate_a.cs` then
gets `using Blob = System.ReadOnlyMemory<byte>;` and a real
`class FfiConverterTypeBlob: FfiConverter<Blob, RustBuffer>`, so the CS0234 facet disappears. The
missing type alias in `crate_b.cs` remains: the same five CS0246 errors.

## Verified fix

By hand, on the unconfigured output: replace the forwarding `FfiConverterTypeBlob` class in `crate_b.cs`
with the same two aliases `crate_a.cs` has:

```csharp
namespace uniffi.crate_b;
using Blob = byte[];
using FfiConverterTypeBlob = FfiConverterByteArray;
```

`dotnet build csharp` then reports zero errors and `dotnet run --project csharp` prints
`blob length: 4`. In other words, rendering `CustomTypeTemplate.cs` for external custom types too,
instead of `ExternalTypeTemplate.cs`, produces working code.
