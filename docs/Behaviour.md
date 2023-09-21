# Introduction

In this document I'll be explaining the different behaviours of toylang that may or may not be apparent just by looking at the [Grammar](./Grammar.md) and the [Tokens](./Tokens.md) of this language.

## Name Shadowing

Name shadowing is a concept where you redeclare a variable that you've already declared and even give it a different type than it previously had (just ignore the fact that toylang doesn't have any types except 64-bit signed integers for now). An example of this is as follows:

```rust
let a = 235
{
    let a = 3   // This is fine
    exit a  // ExitCode = 3
}

exit a  // ExitCode = 235

let a = 1   // This is also fine
exit a  // ExitCode = 1
```
