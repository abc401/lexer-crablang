use std::collections::HashMap;

use crate::parser::{Identifier, LExp, Program, RExp, Statement as Stmt, Term};

#[derive(Debug)]
pub struct Symbol {
    pub ident: Identifier,
    pub rbp_offset: usize,
    pub size_bytes: usize,
    pub initialized: bool,
}

#[derive(Debug)]
pub enum SematicError {
    RedeclareIdent(Identifier),
    UseOfUndeclaredIdent(Identifier),
    UseOfUninitializedIdent(Identifier),
}
use SematicError as SErr;

pub type SymTable = HashMap<String, Symbol>;

pub fn analyze(program: &Program) -> Result<SymTable, SematicError> {
    let mut symtable: SymTable = HashMap::new();
    let mut current_rbp_offset = 0;
    for stmt in program.iter() {
        match stmt {
            Stmt::Declare(ident) => {
                if symtable.contains_key(&ident.lexeme) {
                    let ident = &symtable.get(&ident.lexeme).unwrap().ident;
                    return Err(SErr::RedeclareIdent(ident.clone()));
                }
                current_rbp_offset += 4;
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
                    return Err(SErr::RedeclareIdent(ident.clone()));
                }
                analyze_rexp(rexp, &mut symtable)?;
                current_rbp_offset += 4;
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
                    return Err(SErr::UseOfUndeclaredIdent(l_ident.clone()));
                }
                l_sym.unwrap().initialized = true;
            }
            Stmt::RExp(rexp) => analyze_rexp(rexp, &mut symtable)?,
        }
    }

    return Ok(symtable);
}

fn analyze_term(term: &Term, symtable: &SymTable) -> Result<(), SematicError> {
    match term {
        Term::IntLit(_) => Ok(()),
        Term::LExp(LExp::Ident(ident)) => {
            let sym = symtable.get(&ident.lexeme);
            if sym.is_none() {
                return Err(SErr::UseOfUndeclaredIdent(ident.clone()));
            }
            if !sym.unwrap().initialized {
                return Err(SErr::UseOfUninitializedIdent(ident.clone()));
            }
            return Ok(());
        }
    }
}

fn analyze_rexp(rexp: &RExp, symtable: &SymTable) -> Result<(), SematicError> {
    match rexp {
        RExp::Term(term) => analyze_term(term, symtable)?,
        RExp::Add(lhs, rhs) | RExp::Sub(lhs, rhs) | RExp::Mul(lhs, rhs) | RExp::Div(lhs, rhs) => {
            analyze_rexp(lhs, symtable)?;
            analyze_rexp(rhs, symtable)?;
        }
    }
    return Ok(());
}
