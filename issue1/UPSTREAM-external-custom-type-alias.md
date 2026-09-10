# uniffi-bindgen-cs: custom type from another crate is never aliased in the consuming crate (CS0246)

Bug report for [NordSecurity/uniffi-bindgen-cs](https://github.com/NordSecurity/uniffi-bindgen-cs).
Worked around in `downgrade_uniffi_bindings.py` (`import_external_custom_type_aliases`); every
inserted line is marked `WORKAROUND(uniffi-bindgen-cs)`. Remove the workaround once a fixed release is
pinned in `generate_uniffi_bindings.sh`.

## Environment

- uniffi-bindgen-cs `v0.11.0+v0.31.0` (cargo git install), uniffi `0.31.2`
- Library mode: `uniffi-bindgen-cs --library liblivekit_uniffi.dylib --config uniffi.toml --out-dir out`
- Library built from [livekit/rust-sdks](https://github.com/livekit/rust-sdks) (`livekit-uniffi` crate,
  which links `livekit-common`, `livekit-datatrack` and `livekit-net`; all four are UniFFI components)
- Consumer: Unity 2022.3, C# 9, Roslyn 4.3.1. The error is independent of the language version.

## Summary

A `custom_type!` declared in crate A and used by crate B (via `uniffi::use_remote_type!`) is only aliased
(`using Bytes = ...;`) in A's generated file. B's file uses the name `Bytes` in signatures and in its
forwarding converter but declares no alias. C# `using` aliases are per file, so B does not compile.

## Reproduction

Rust side (as in rust-sdks):

```rust
// crate livekit-common, src/ffi_types.rs
uniffi::custom_type!(Bytes, Vec<u8>, { remote });

// crate livekit-datatrack, src/e2ee.rs
uniffi::use_remote_type!(livekit_common::Bytes);
// ... exported functions / trait methods taking and returning `Bytes`
```

Both crates are compiled into one cdylib. Generate in library mode with this `uniffi.toml` passed via
`--config` (the bug also reproduces without any custom-type config; then the alias in A is
`using Bytes = byte[];`):

```toml
[bindings.csharp.custom_types.Bytes]
type_name = "System.ReadOnlyMemory<byte>"
into_custom = "new System.ReadOnlyMemory<byte>({})"
from_custom = "{}.ToArray()"
```

## Generated output

`livekit_common.cs` (defining crate) has the alias right after its namespace:

```csharp
namespace uniffi.livekit_common;

using Bytes = System.ReadOnlyMemory<byte>;
```

`livekit_datatrack.cs` and `livekit_uniffi.cs` (consuming crates) import the namespace but get no alias,
while using the name everywhere:

```csharp
using uniffi.livekit_common;

namespace uniffi.livekit_datatrack;
// (no `using Bytes = ...;` here)

class FfiConverterTypeBytes : FfiConverterRustBuffer<Bytes>          // from ExternalTypeTemplate.cs
{
    public override Bytes Read(BigEndianStream stream)
    {
        return uniffi.livekit_common.FfiConverterTypeBytes.INSTANCE.Read(
            new uniffi.livekit_common.BigEndianStream(stream.InnerStream)
        );
    }
    ...
}

public async Task<Bytes> ReadAll() ...
public async Task Write(Bytes @data) ...
```

Compiler output (46 occurrences in our build, all of the same kind):

```
livekit_datatrack.cs(1805,9): error CS0246: The type or namespace name 'Bytes' could not be found (are you missing a using directive or an assembly reference?)
livekit_uniffi.cs(4968,14): error CS0246: The type or namespace name 'Bytes' could not be found (are you missing a using directive or an assembly reference?)
```

## Cause

`bindgen/templates/Types.cs` dispatches `Type::Custom`:

```
{%- when Type::Custom { module_path, name, builtin } %}
{%- if ci.is_external(type_) %}
{% include "ExternalTypeTemplate.cs" %}
{%- else %}
{% include "CustomTypeTemplate.cs" %}
{%- endif %}
```

Only `CustomTypeTemplate.cs` calls `self.add_type_alias(name, ...)` (for the builtin or for
`config.type_name`). `ExternalTypeTemplate.cs` emits the forwarding `FfiConverterType{name}` and
`self.add_import(package_name)`, but no alias. The comment in `CustomTypeTemplate.cs` even says the alias
"is also what we have [when] an external type ... references a custom type", yet that branch is never
reached for external custom types. Importing the defining namespace cannot help because an alias is not
a member of a namespace.

Second facet, visible once the type alias exists: without a `custom_types` entry, the defining crate's
file also represents the *converter* as an alias (`using FfiConverterTypeBlob = FfiConverterByteArray;`),
so the forwarding converter that `ExternalTypeTemplate.cs` generates in the consuming file cannot resolve
`uniffi.crate_a.FfiConverterTypeBlob` either:

```
crate_b.cs(1510,16): error CS0234: The type or namespace name 'FfiConverterTypeBlob' does not exist in the namespace 'uniffi.crate_a' (are you missing an assembly reference?)
```

With a `custom_types` entry (as in the LiveKit case above), `CustomTypeTemplate.cs` emits a real
`class FfiConverterTypeBlob : FfiConverter<Blob, RustBuffer>` in the defining file and only the missing
type alias remains.

## Expected behaviour / proposed fix

A custom type is an alias plus a converter over a builtin; nothing about it has to be forwarded to the
defining crate. Rendering `CustomTypeTemplate.cs` for external custom types as well, instead of
`ExternalTypeTemplate.cs`, gives every consuming file the same self-contained `using Blob = ...;` /
`FfiConverterTypeBlob` pair the defining file has:

```
{%- when Type::Custom { module_path, name, builtin } %}
{% include "CustomTypeTemplate.cs" %}
```

Verified by hand on the reproduction project below: replacing the forwarding class in `crate_b.cs` with
`using Blob = byte[]; using FfiConverterTypeBlob = FfiConverterByteArray;` compiles with zero errors and
`dotnet run` prints `blob length: 4`.

Two details for the real fix. `config.custom_types` is the consuming component's config; in library mode
with `--config` every component receives the same table, but when the mapping lives only in the defining
crate's `uniffi.toml` the generator should look up that component's config (as
`namespace_for_module_path` already does for the namespace), otherwise the two files disagree on the C#
type. And if the forwarding route is kept instead, `add_type_alias` must also run in the consuming file
and the defining file's converter must be a class rather than an alias for the qualified reference to
resolve.

## Reproduction project

`upstream-repro/issue1/` next to this file: `crate_a` defines `Blob`, `crate_b` uses it and is the cdylib,
`csharp/` compiles the unmodified bindgen output (see its README.md for prerequisites and details).

```
cd upstream-repro/issue1 && ./build.sh
...
csharp/Generated/crate_b.cs(1505,52): error CS0246: The type or namespace name 'Blob' could not be found (are you missing a using directive or an assembly reference?)
   (6 occurrences)
```

`BINDGEN_CONFIG=uniffi.toml ./build.sh` shows the configured variant: the same CS0246 errors, no CS0234
facet.
