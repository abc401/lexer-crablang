use std::collections::HashMap;

use crate::{
    parser::{Identifier, LExp, Program, RExp, Statement as Stmt, Term},
    CompileError,
};

#[derive(Debug)]
pub struct Symbol {
    pub ident: Identifier,
    pub rbp_offset: usize,
    pub size_bytes: usize,
    pub initialized: bool,
}

pub type SymTable = HashMap<String, Symbol>;

pub fn analyze(program: &Program) -> Result<SymTable, CompileError> {
    let mut symtable: SymTable = HashMap::new();
    let mut current_rbp_offset = 0;
    for stmt in program.iter() {
        match stmt {
            Stmt::Declare(ident) => {
                if symtable.contains_key(&ident.lexeme) {
                    let ident = &symtable.get(&ident.lexeme).unwrap().ident;
                    return Err(CompileError::RedeclareIdent(ident.clone()));
                }
                current_rbp_offset += 8;
                symtable.insert(
                    ident.lexeme.clone(),
                    Symbol {
                        ident: ident.clone(),
                        size_bytes: 8,
                        rbp_offset: current_rbp_offset,
                        initialized: false,
                    },
                );
            }
            Stmt::Initialize(l_ident, rexp) => {
                if symtable.contains_key(&l_ident.lexeme) {
                    let ident = &symtable.get(&l_ident.lexeme).unwrap().ident;
                    return Err(CompileError::RedeclareIdent(ident.clone()));
                }
                analyze_rexp(rexp, &mut symtable)?;
                current_rbp_offset += 8;
                symtable.insert(
                    l_ident.lexeme.clone(),
                    Symbol {
                        ident: l_ident.clone(),
                        size_bytes: 8,
                        rbp_offset: current_rbp_offset,
                        initialized: true,
                    },
                );
            }
            Stmt::Assign(lexp, rexp) => {
                analyze_rexp(rexp, &mut symtable)?;
                let LExp::Ident(l_ident) = lexp;
                let l_sym = symtable.get_mut(&l_ident.lexeme);
                if l_sym.is_none() {
                    return Err(CompileError::UseOfUndeclaredIdent(l_ident.clone()));
                }
                l_sym.unwrap().initialized = true;
            }
            Stmt::RExp(rexp) => analyze_rexp(rexp, &mut symtable)?,
            _ => panic!("[Semantic Analysis] Not implemented: {}", stmt),
        }
    }

    return Ok(symtable);
}

fn analyze_term(term: &Term, symtable: &SymTable) -> Result<(), CompileError> {
    match term {
        Term::IntLit(_) => Ok(()),
        Term::LExp(LExp::Ident(ident)) => {
            let sym = symtable.get(&ident.lexeme);
            if sym.is_none() {
                return Err(CompileError::UseOfUndeclaredIdent(ident.clone()));
            }
            if !sym.unwrap().initialized {
                return Err(CompileError::UseOfUninitializedIdent(ident.clone()));
            }
            return Ok(());
        }
        _ => panic!("[Semantic Analysis] Not implemented: {}", term),
    }
}

fn analyze_rexp(rexp: &RExp, symtable: &SymTable) -> Result<(), CompileError> {
    match rexp {
        RExp::Term(term) => analyze_term(term, symtable)?,
        RExp::Add(lhs, rhs)
        | RExp::Sub(lhs, rhs)
        | RExp::Mul(lhs, rhs)
        | RExp::Div(lhs, rhs)
        | RExp::Equal(lhs, rhs)
        | RExp::NotEqual(lhs, rhs)
        | RExp::Less(lhs, rhs)
        | RExp::LessEqual(lhs, rhs)
        | RExp::Greater(lhs, rhs)
        | RExp::GreaterEqual(lhs, rhs) => {
            analyze_rexp(lhs, symtable)?;
            analyze_rexp(rhs, symtable)?;
        }

        _ => panic!("[Semantic Analysis] Not implemented: {}", rexp),
    }
    return Ok(());
}
