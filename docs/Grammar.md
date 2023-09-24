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

$RExp \to Compare$

$Compare \to \begin{cases}
    Add == Add \\
    Add~\text{!=}~Add \\
    Add <  Add \\
    Add <= Add \\
    Add >  Add \\
    Add >= Add \\
\end{cases}$

$Add \to \begin{cases}
    Mult~+~Mult \\
    Mult~-~Mult \\
\end{cases}$

$Mult \to \begin{cases}
    Term~*~Term \\
    Term~/~Term \\
    Term \\
\end{cases}$

$Term \to \begin{cases}
    IntLit \\
    LExp \\
    -Term \\
    [LBrace]RExp[RBrace] \\
\end{cases} \\$

$LExp \to Ident$
