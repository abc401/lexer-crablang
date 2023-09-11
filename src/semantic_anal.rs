use std::collections::HashMap;

use crate::parser::{LExpression as LE, Program, RExpression as RE, Statement as Stmt};

#[derive(Debug)]
pub struct Symbol {
    pub id: String,
    pub rbp_offset: usize,
    pub initialized: bool,
}

#[derive(Debug)]
pub enum SematicError {
    RedeclareSymbol(String),
    UseOfUndeclaredSymbol(String),
    UseOfUninitializedSymbol(String),
}
use SematicError as SE;

pub type SymTable = HashMap<String, Symbol>;

pub fn analyze(program: &Program) -> Result<SymTable, SematicError> {
    let mut symtable: SymTable = HashMap::new();
    let mut current_rbp_offset = 0;
    for stmt in program.iter() {
        match stmt {
            Stmt::Declare(ident) => {
                if symtable.contains_key(ident) {
                    return Err(SE::RedeclareSymbol(ident.clone()));
                }
                current_rbp_offset += 4;
                symtable.insert(
                    ident.clone(),
                    Symbol {
                        id: ident.clone(),
                        rbp_offset: current_rbp_offset,
                        initialized: false,
                    },
                );
            }
            Stmt::Initialize(l_ident, rexp) => {
                if symtable.contains_key(l_ident) {
                    return Err(SE::RedeclareSymbol(l_ident.clone()));
                }
                match rexp {
                    RE::Ident(r_ident) => {
                        let r_sym = symtable.get(r_ident);
                        if r_sym.is_none() {
                            return Err(SE::UseOfUndeclaredSymbol(r_ident.clone()));
                        }
                        let r_sym = r_sym.unwrap();
                        if !r_sym.initialized {
                            return Err(SE::UseOfUninitializedSymbol(r_ident.clone()));
                        }
                    }
                    _ => (),
                }
                current_rbp_offset += 4;
                symtable.insert(
                    l_ident.clone(),
                    Symbol {
                        id: l_ident.clone(),
                        rbp_offset: current_rbp_offset,
                        initialized: true,
                    },
                );
            }
            Stmt::Assign(lexp, rexp) => {
                match rexp {
                    RE::Ident(r_ident) => {
                        let r_sym = symtable.get(r_ident);
                        if r_sym.is_none() {
                            return Err(SE::UseOfUndeclaredSymbol(r_ident.clone()));
                        } else if !r_sym.unwrap().initialized {
                            return Err(SE::UseOfUninitializedSymbol(r_ident.clone()));
                        }
                    }
                    _ => {}
                }
                let LE::Ident(l_ident) = lexp;
                let l_sym = symtable.get_mut(l_ident);
                if l_sym.is_none() {
                    return Err(SE::UseOfUndeclaredSymbol(l_ident.clone()));
                }
                l_sym.unwrap().initialized = true;
            }
        }
    }

    return Ok(symtable);
}
