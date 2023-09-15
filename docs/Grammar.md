$$
\begin{align}
    Program &\to [Stmt]^* \\
    Stmt &\to \begin{cases}
        \text{let } [Ident]\text{: } [TypeIdent] [NewLine] \\
        \text{let } [Ident]\text{: } [TypeIdent] = [RExp] [NewLine] \\
        [RExp] [NewLine] \\
        [LExp] = [RExp] [NewLine] \\
    \end{cases} \\

    RExp &\to \begin{cases}
        [Term] \\
        [RExp] + [RExp] \\
        [RExp] - [RExp] \\
        [RExp] * [RExp] \\
        [RExp] / [RExp] \\
    \end{cases} \\

    Term &\to \begin{cases}
        [IntLit] \\
        [LExp] \\
    \end{cases} \\

    LExp &\to \begin{cases}
        [Ident]
    \end{cases} \\
\end{align}
$$
