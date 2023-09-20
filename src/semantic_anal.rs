use std::{collections::HashMap, ptr::NonNull};

use crate::{
    parser::{Identifier, LExp, Program, RExp, Stmt, Term},
    CompileError,
};

fn analyze_aux(stmts: Vec<Stmt>) {}

pub fn analyze(program: &mut Program) -> Result<Env, CompileError> {
    let mut env = Env::new();
    for stmt in program.iter_mut() {
        match stmt {
            Stmt::Declare(ident) => {
                env.declare(ident);
            }
            Stmt::Initialize(l_ident, rexp) => {
                analyze_rexp(rexp, &mut env)?;
                env.init(l_ident);
            }
            Stmt::Assign(lexp, rexp) => {
                analyze_rexp(rexp, &mut env)?;
                let LExp::Ident(l_ident) = lexp;
                env.decorate_ident(l_ident);
                let l_sym = env.get_mut(&l_ident.lexeme);
                if l_sym.is_none() {
                    return Err(CompileError::UseOfUndeclaredIdent(l_ident.clone()));
                }
                l_sym.unwrap().initialized = true;
            }
            Stmt::RExp(rexp) => analyze_rexp(rexp, &mut env)?,
            _ => panic!("[Semantic Analysis] Not implemented: {}", stmt),
        }
    }

    return Ok(env);
}

fn analyze_term(term: &mut Term, env: &Env) -> Result<(), CompileError> {
    match term {
        Term::IntLit(_) => Ok(()),
        Term::LExp(LExp::Ident(ident)) => {
            env.decorate_ident(ident);
            let sym = env.get(&ident.lexeme);
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

fn analyze_rexp(rexp: &mut RExp, env: &Env) -> Result<(), CompileError> {
    match rexp {
        RExp::Term(term) => analyze_term(term, env)?,
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
            analyze_rexp(lhs, env)?;
            analyze_rexp(rhs, env)?;
        }

        _ => panic!("[Semantic Analysis] Not implemented: {}", rexp),
    }
    return Ok(());
}
