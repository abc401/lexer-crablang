$Program \to (Stmt[Newline])^*$

$Stmt \to \begin{cases}
    let~Ident\\
    let~Ident~=~RExp \\
    if~RExp~Block \\
    Block \\
    RExp \\
    LExp~=~RExp \\
    exit~RExp \\
\end{cases}$

$Block \to \{ (Stmt[Newline])^* \}$

$RExp \to \begin{cases}
    RExp +  RExp \\
    RExp -  RExp \\
    RExp == RExp \\
    RExp~\text{!=}~RExp \\
    RExp <  RExp \\
    RExp <= RExp \\
    RExp >  RExp \\
    RExp >= RExp \\
    Term \\
\end{cases}$

$Term \to \begin{cases}
    IntLit \\
    LExp \\
    -Term \\
    [OpeningBrace]RExp[ClosingBrace] \\
\end{cases} \\$

$LExp \to Ident$
