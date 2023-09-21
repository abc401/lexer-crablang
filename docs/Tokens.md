# Toylang Tokens

Following are the different tokens supported by the language as of yet:

## Literals

Toylang, currently only supports decimal int literals. Following are all examples of correct int literals:

- `123`
- `0000132`
- `1900`

Following are examples of invalid int literals:

- `135ab32`
- `0x23af`
- `13_523_562`
- `3525.534`

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

### Assignment:

The `=` character is used as the assignment operator and it can be used to assign values to variables.

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

### Unary Operators

Toylang supports the unary negation operator `-`.

## Delimiters

### Brackets i.e. `()`

Toylang supports adding a pair of brackets around an expression to make it have a heigher precedence than its surroundings. Following is an example of this:

- `1 - 2 + 3` == `2`
- `1 - (2 + 3)` == `-4`

Another reason you might want to consider putting brackets around your expressions is if you want to split it on different lines. You can do this in the following way:

```rust
let a = 1 +
    2 + 3   // Syntax Error

let a = 1 + (
    2 + 3
    + 4
)   // This is fine
```

### Curly Braces i.e. `{}`

The curly braces can be used to start a new scope. In this new scope all the variables of the parent scope are accessible but the variables of the variables of the child scope are not accessible by the parent scope
