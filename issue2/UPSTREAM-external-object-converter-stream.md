# uniffi-bindgen-cs: external object/callback-interface converter is passed the wrong crate's BigEndianStream (CS1503)

Bug report for [NordSecurity/uniffi-bindgen-cs](https://github.com/NordSecurity/uniffi-bindgen-cs).
Worked around in `downgrade_uniffi_bindings.py` (`wrap_external_object_converters`); every inserted
line is marked `WORKAROUND(uniffi-bindgen-cs)`. Remove the workaround once a fixed release is pinned in
`generate_uniffi_bindings.sh`.

## Environment

- uniffi-bindgen-cs `v0.11.0+v0.31.0` (cargo git install), uniffi `0.31.2`
- Library mode: `uniffi-bindgen-cs --library liblivekit_uniffi.dylib --config uniffi.toml --out-dir out`
- Library built from [livekit/rust-sdks](https://github.com/livekit/rust-sdks) (`livekit-uniffi` crate,
  which links `livekit-common`, `livekit-datatrack` and `livekit-net`; all four are UniFFI components)
- Consumer: Unity 2022.3, C# 9, Roslyn 4.3.1. The error is independent of the language version.

## Summary

For an object or callback interface declared in crate A and used by crate B, B's file gets
`using FfiConverterTypeX = <A namespace>.FfiConverterTypeX;`. Every generated file has its own
`BigEndianStream` class, so B's code that calls `FfiConverterTypeX.INSTANCE.Read(stream)` or
`.Write(value, stream)` with B's stream does not compile against A's converter. Records, enums, errors and
custom types do not have this problem because `ExternalTypeTemplate.cs` generates a local forwarding
converter that re-wraps the stream; the object template does not.

## Reproduction

Rust side (as in rust-sdks): `livekit-datatrack` declares the foreign-implementable traits
`DecryptionProvider` and `EncryptionProvider`; `livekit-uniffi` exposes functions and records that take
them as `Option<Arc<dyn DecryptionProvider>>`. Both crates are compiled into one cdylib; generate in
library mode.

## Generated output

`livekit_uniffi.cs` (consuming crate):

```csharp
namespace uniffi.livekit_uniffi;

using FfiConverterTypeDecryptionProvider = uniffi.livekit_datatrack.FfiConverterTypeDecryptionProvider;  // ExternalObjectTypeTemplate.cs
using FfiConverterTypeEncryptionProvider = uniffi.livekit_datatrack.FfiConverterTypeEncryptionProvider;

// OptionalTemplate.cs, same file
class FfiConverterOptionalTypeDecryptionProvider : FfiConverterRustBuffer<DecryptionProvider?>
{
    public override DecryptionProvider? Read(BigEndianStream stream)      // uniffi.livekit_uniffi.BigEndianStream
    {
        if (stream.ReadByte() == 0) { return null; }
        return FfiConverterTypeDecryptionProvider.INSTANCE.Read(stream);  // expects uniffi.livekit_datatrack.BigEndianStream
    }

    public override void Write(DecryptionProvider? value, BigEndianStream stream)
    {
        ...
        FfiConverterTypeDecryptionProvider.INSTANCE.Write((DecryptionProvider)value, stream);
    }
}
```

`livekit_datatrack.cs` (defining crate) declares
`class FfiConverterTypeDecryptionProvider : FfiConverter<DecryptionProvider, ulong>` against its own
`BigEndianStream`.

Compiler output (4 occurrences in our build: Read and Write for each of the two providers):

```
livekit_uniffi.cs(13414,69): error CS1503: Argument 1: cannot convert from 'uniffi.livekit_uniffi.BigEndianStream' to 'uniffi.livekit_datatrack.BigEndianStream'
livekit_uniffi.cs(13484,94): error CS1503: Argument 2: cannot convert from 'uniffi.livekit_uniffi.BigEndianStream' to 'uniffi.livekit_datatrack.BigEndianStream'
```

Only positions that serialise the object through a `RustBuffer` are affected (optional, sequence and
map converters, record fields, callback arguments). Plain function parameters and return values go
through `Lower`/`Lift` with a `ulong` handle, which is namespace-independent, so a test suite that only
passes external objects as direct arguments does not catch this.

## Cause

`bindgen/templates/Types.cs` includes `ExternalObjectTypeTemplate.cs` for external `Type::Object` and
`Type::CallbackInterface`. That template only aliases the remote converter:

```
{{- self.add_import(package_name) }}
{{- self.add_type_alias(local_ffi_converter_name, fully_qualified_ffi_converter) }}
```

`ExternalTypeTemplate.cs`, used for every other external type, instead generates a local
`FfiConverterType{name} : FfiConverterRustBuffer<T>` whose `Read`/`Write` forward to the remote converter
with `new {{ package_name }}.BigEndianStream(stream.InnerStream)`. Objects need the same treatment; the
alias approach can only work while all files share one `BigEndianStream`, which library mode never does.

## Expected behaviour / proposed fix

Replace the alias in `ExternalObjectTypeTemplate.cs` with a forwarding converter (handles are `ulong`
in this release):

```
{%- let namespace = ci.namespace_for_module_path(module_path)? %}
{%- let package_name = self.external_type_package_name(module_path, namespace) %}
{%- let local_ffi_converter_name = "FfiConverterType{}"|format(name) %}
{%- let type_label = name|class_name(ci) %}
{%- let remote = "{}.{}.INSTANCE"|format(package_name, local_ffi_converter_name) %}

{{- self.add_import(package_name) }}

class {{ local_ffi_converter_name }} : FfiConverter<{{ type_label }}, ulong> {
    public static {{ local_ffi_converter_name }} INSTANCE = new {{ local_ffi_converter_name }}();

    public override {{ type_label }} Lift(ulong value) { return {{ remote }}.Lift(value); }
    public override ulong Lower({{ type_label }} value) { return {{ remote }}.Lower(value); }
    public override int AllocationSize({{ type_label }} value) { return {{ remote }}.AllocationSize(value); }

    public override {{ type_label }} Read(BigEndianStream stream) {
        return {{ remote }}.Read(new {{ package_name }}.BigEndianStream(stream.InnerStream));
    }

    public override void Write({{ type_label }} value, BigEndianStream stream) {
        {{ remote }}.Write(value, new {{ package_name }}.BigEndianStream(stream.InnerStream));
    }
}
```

This is exactly what our post-processor inserts; with it (and the fix for the sibling report about
external custom-type aliases) the four-crate library compiles with zero errors.

## Reproduction project

`upstream-repro/issue2/` next to this file: `crate_a` defines the object `Widget`, `crate_b` exports a
function taking `Option<Arc<Widget>>` and is the cdylib, `csharp/` compiles the unmodified bindgen output
(see its README.md for prerequisites and details).

```
cd upstream-repro/issue2 && ./build.sh
...
csharp/Generated/crate_b.cs(1464,53): error CS1503: Argument 1: cannot convert from 'uniffi.crate_b.BigEndianStream' to 'uniffi.crate_a.BigEndianStream'
csharp/Generated/crate_b.cs(1480,66): error CS1503: Argument 2: cannot convert from 'uniffi.crate_b.BigEndianStream' to 'uniffi.crate_a.BigEndianStream'
```

Verified by hand: replacing the alias in `crate_b.cs` with the forwarding converter proposed above compiles
with zero errors, and `dotnet run` prints `widget id: 7` and `no widget: 0`.
