# Toylang Tokens

Following are the different tokens supported by the language as of yet:

## Integer Literals

Toylang only supports positive, decimal, int literals currently. Following are all examples of correct int literals:

- `123`
- `0000132`
- `1900`

Following are examples of invalid int literals:

- `-123`
- `135ab32`
- `0x23af`
- `13_523_562`

## Identifiers

Identifiers in Toylang can consist of all alphanumeric characters along with `_`. The first character however can only be an alphabet or `_`. Following are examples of valid identifiers:

- `abc`
- `abc123`
- `_valid_identifier_`
- `AlsoAValidIdent`

Following are examples of invalid identifiers:

- `abc def`
- `1nvalid_identifier` <- has a one at the start

## Keywords

The only keyword that Toylang supports as of yet is `let`. It is used to initialize or declare a variable.

## Operators

Toylang supports a single operator which is `=`. It can be used to assign a value to a variable.
