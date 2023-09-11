$$
\begin{align}
    Program &\to [Stmt]^* \\
    Stmt &\to \begin{cases}
        \text{let } [Ident] \\
        \text{let } [Ident] = [RExp] \\
        [LExp] = [RExp]
    \end{cases} \\

    RExp &\to \begin{cases}
        [LExpr] \\
        [IntLit] \\
        [BinExp] \\
    \end{cases} \\

    BinExp &\to \begin{cases}
        [Term] \\
        [BinExp] + [Term] \\
    \end{cases} \\

    Term &\to \begin{cases}
        [IntLit] \\
        [Ident] \\
    \end{cases} \\

    LExp &\to \begin{cases}
        [Ident]
    \end{cases} \\
\end{align}
$$
