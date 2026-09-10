// Exercises the crate_b API. Compiles and runs once the issue is fixed (see ../README.md).
using System;
using uniffi.crate_a;
using uniffi.crate_b;

var widget = new Widget(7);
Console.WriteLine($"widget id: {CrateBMethods.WidgetId(widget)}");
Console.WriteLine($"no widget: {CrateBMethods.WidgetId(null)}");
