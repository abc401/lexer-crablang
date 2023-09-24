$Program \to (Stmt)^*$

$Stmt \to \begin{cases}
    let~Ident\\
    let~Ident~=~RExp \\
    if~RExp~Block \\
    Block \\
    RExp \\
    LExp~=~RExp \\
    exit~RExp \\
\end{cases}$

$Block \to [LCurly]~Stmt^*~[RCurly]$

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
    [LBrace]RExp[RBrace] \\
\end{cases} \\$

$LExp \to Ident$
