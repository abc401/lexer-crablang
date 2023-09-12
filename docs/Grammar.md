$$
\begin{align}
    Program &\to [Stmt]^* \\
    Stmt &\to \begin{cases}
        \text{let } [Ident] [NewLine] \\
        \text{let } [Ident] = [RExp] [NewLine] \\
        [RExp] [NewLine] \\
        [LExp] = [RExp] [NewLine] \\
    \end{cases} \\

    RExp &\to \begin{cases}
        [LExp] \\
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
