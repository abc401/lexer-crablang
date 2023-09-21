# Toylang Tokens

Following are the different tokens supported by the language as of yet:

## Literals

Toylang, currently, only supports positive, decimal, int literals. Following are all examples of correct int literals:

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
- `123abc`
- `1nvalid_identifier`

## Keywords

Following are the different types of keywords that are supported by toylang:

### Let

The `let` keyword can be used to initialize or declare a variable.

### Exit

The `exit` keyword can be used to exit at any part of the program with the desired exit code.

## Operators

Following is a brief description of the different operator tokens that toylang currently supports and what those tokens are meant to do:

### Assignment: `=`

This is the assignment operator and it can be used to assign values to variables.

### Arithmatic Operators

Toylang now supports arithmatic operations such as addition, subtraction, multiplication, division. As you would expect, these operations are accomplished by using the `+`, `-`, `*` and the `/` characters respectively.

### Comparison Operators

All the different types of comparisons are supported by toylang, namely they are:

- Less than: `<`.
- Less than or equal to: `<=`.
- Greater than: `>`.
- Greater than or equal to: `>=`.
- Equal to: `==`.
- Not equal to: `!=`.
