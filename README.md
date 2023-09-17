# unmarshalled

Listens to the accessibility bus for AT-SPI2 signals and reports those messages
which have 'unmarshalled body signatures'.

The type signature is made up of zero or more single complete types, each made up of one or more type codes.
The signature is a list of single complete types.

A 'type code' is an ASCII character representing the type of a value.
A 'single complete type' is a sequence of type codes that fully describes one type: either a basic type, or a single fully-described container type.

A data block can be interpreted (unmarshalled) into D-Bus type system types by considering length alignment and padding of each single complete type as specified in the marshalled signature.
The marshalled signature reflects the data block and omits outer struct (or tuple) parentheses.

## Reference

[DBus Specifications](https://dbus.freedesktop.org/doc/dbus-specification.html)

## Example output

```text
============================================================
      D-Bus message with unmarshalled body signature:
============================================================
 Signature: "((so)(so)(so)iiassusau)",
 Sender: ":1.100", Path: "/org/a11y/atspi/cache"
  Toolkit name: Gecko
  Application name: "Firefox"
  Object role: "application"

```
