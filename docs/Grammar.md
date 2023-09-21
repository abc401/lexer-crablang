$Program \to [Stmt]^\text{*}$

$Stmt \to \begin{cases}
\text{let } [Ident] [NewLine] \\
\text{let } [Ident] = [RExp] [NewLine] \\
[RExp] [NewLine] \\
[LExp] = [RExp] [NewLine] \\
\text{exit } [RExp] [NewLine] \\
\end{cases}$

$RExp \to \begin{cases}
    [Term] \\
    [RExp] \text{ + }  [RExp] \\
    [RExp] \text{ - }  [RExp] \\
    [RExp] \text{ == } [RExp] \\
    [RExp] \text{ != } [RExp] \\
    [RExp] \text{ < }  [RExp] \\
    [RExp] \text{ <= } [RExp] \\
    [RExp] \text{ > }  [RExp] \\
    [RExp] \text{ >= } [RExp] \\
\end{cases}$

$Term \to \begin{cases}
    [IntLit] \\
    [LExp] \\
    -[Term] \\
    ([RExp]) \\
\end{cases} \\$

$LExp \to [Ident]$
