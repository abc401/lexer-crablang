
# Lexer in Rust

A lexer, written in the rust programming language, for a made up toy language. It has the following tokens:

## Identifiers

The grammer for the identifiers is as follows:
> `[a-zA-Z_][a-zA-Z_0-9]*`

## Keywords

Following are the different keywords that are present in this language as of this version:
- `let`
- `fun`
- `return`

## Literals

Followin are the supported literal types:
- Integer literals: `[0-9]*`
- Float literals: `[0-9]*\.([0-9]*?)`

## Operators

Following are the different supported operators:

### Arithmatic Operators

- Addition (`+`)
- Subtraction (`-`)
- Multiplication (`*`)
- Division (`/`)
- Modulus (`%`)

### Assignment Operators

- Simple Assignment (`=`)
- Subtract Assignment (`-=`)
- Divide Assignment (`/=`)
- Multiply Assignment (`*=`)
- Add Assignment (`+=`)
- Modulus Assignment (`%=`)

### Relational Operators

- Equal to (`==`)
- Not equal (`!=`)
- Less than (`<`)
- Greater than (`>`)
- Less than or equal to  (`<=`)
- Greater than or equal to (`>=`)

### Logical Operators

- And (`&&`)
- Or (`||`)
- Not (`!`)

### Bitwise Operators

- And (`&`)
- Or (`|`)
- Not (`~`)

### Punctuators

- Scope Delimiters (`{}`)
- Commas (`,`)
- Paranthesis (`()`)