$$
\begin{equation}
    Program \to Statement^*
\end{equation}
\\
\begin{equation}
    Statement \to \begin{cases}
        \text{let } Ident \\
        \text{let } Ident = RExpression \\
        LExpression = RExpression
    \end{cases}
\end{equation}
\\

\begin{equation}
    LExpression \to \begin{cases}
        Ident
    \end{cases}
\end{equation}
\\
\begin{equation}
    RExpression \to \begin{cases}
        Ident \\
        IntLiteral \\
    \end{cases}
\end{equation}
\\
\begin{equation}
    Ident \to [\_ a-zA-Z][\_ a-zA-Z0-9]^*
\end{equation}
\\
\begin{equation}
    IntLiteral \to [0-9]^+
\end{equation}
$$
