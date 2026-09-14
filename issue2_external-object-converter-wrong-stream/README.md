# Issue 2: external object converter is passed the wrong crate's BigEndianStream

Minimal, self-contained reproduction against
[uniffi-bindgen-cs](https://github.com/NordSecurity/uniffi-bindgen-cs) `v0.11.0+v0.31.0`, uniffi `0.31`.
The generated code is used as-is.

| crate     | role                                                                          |
|-----------|-------------------------------------------------------------------------------|
| `crate_a` | defines `Widget`, a `uniffi::Object` with a constructor and one method        |
| `crate_b` | exports `widget_id(Option<Arc<Widget>>)`; the cdylib that bindgen reads       |

## Run

```
./build.sh
```

Builds `target/release/libcrate_b.dylib` (`.so` on Linux, `crate_b.dll` on Windows), runs `uniffi-bindgen-cs --library` into `csharp/Generated/`
(one unmodified `.cs` file per crate), copies the library to `csharp/native/` so `dotnet run` can load it
later, and runs `dotnet build csharp`. Expected:

```
csharp/Generated/crate_b.cs(1464,53): error CS1503: Argument 1: cannot convert from 'uniffi.crate_b.BigEndianStream' to 'uniffi.crate_a.BigEndianStream'
csharp/Generated/crate_b.cs(1480,66): error CS1503: Argument 2: cannot convert from 'uniffi.crate_b.BigEndianStream' to 'uniffi.crate_a.BigEndianStream'
```

## What bindgen generates

For an object of another crate, `ExternalObjectTypeTemplate.cs` only aliases that crate's converter:

```csharp
using uniffi.crate_a;
namespace uniffi.crate_b;
using FfiConverterTypeWidget = uniffi.crate_a.FfiConverterTypeWidget;
```

Every generated file has its own `BigEndianStream` class. The optional converter in `crate_b.cs`
(from `OptionalTemplate.cs`) hands its own stream to crate_a's converter:

```csharp
class FfiConverterOptionalTypeWidget: FfiConverterRustBuffer<Widget?> {
    public override Widget? Read(BigEndianStream stream) {              // uniffi.crate_b.BigEndianStream
        if (stream.ReadByte() == 0) {
            return null;
        }
        return FfiConverterTypeWidget.INSTANCE.Read(stream);            // wants uniffi.crate_a.BigEndianStream
    }

    public override void Write(Widget? value, BigEndianStream stream) {
        ...
        FfiConverterTypeWidget.INSTANCE.Write((Widget)value, stream);   // same
    }
}
```

Only positions that go through a `RustBuffer` are affected (optional, sequence and map converters, record
fields, callback arguments). A plain `Arc<Widget>` parameter uses `Lower`/`Lift` with a `ulong` handle and
compiles, which is why a test that only passes external objects directly does not catch this.

## Verified fix

By hand: replace the alias in `crate_b.cs` with a local forwarding converter that re-wraps the stream, the
way `ExternalTypeTemplate.cs` already does for records and enums:

```csharp
class FfiConverterTypeWidget : FfiConverter<uniffi.crate_a.Widget, ulong> {
    public static FfiConverterTypeWidget INSTANCE = new FfiConverterTypeWidget();
    static uniffi.crate_a.FfiConverterTypeWidget Remote => uniffi.crate_a.FfiConverterTypeWidget.INSTANCE;

    public override uniffi.crate_a.Widget Lift(ulong value) => Remote.Lift(value);
    public override ulong Lower(uniffi.crate_a.Widget value) => Remote.Lower(value);
    public override int AllocationSize(uniffi.crate_a.Widget value) => Remote.AllocationSize(value);
    public override uniffi.crate_a.Widget Read(BigEndianStream stream) =>
        Remote.Read(new uniffi.crate_a.BigEndianStream(stream.InnerStream));
    public override void Write(uniffi.crate_a.Widget value, BigEndianStream stream) =>
        Remote.Write(value, new uniffi.crate_a.BigEndianStream(stream.InnerStream));
}
```

`dotnet build csharp` then reports zero errors and `dotnet run --project csharp` prints `widget id: 7`
and `no widget: 0`.

## Layout

```
Cargo.toml          workspace: crate_a, crate_b
crate_a/            defines Widget
crate_b/            cdylib, uses Option<Arc<Widget>>
csharp/             Issue2.csproj, Program.cs, Generated/ (bindgen output), native/ (shared library)
build.sh
```

`csharp/Generated/`, `csharp/native/`, `csharp/bin/`, `csharp/obj/` and `target/` are ignored by git;
`./build.sh` recreates them.
