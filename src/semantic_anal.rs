use std::{collections::HashMap, ptr::NonNull};

use crate::{
    parser::{Identifier, LExp, Program, RExp, Stmt, Term},
    CompileError,
};

#[derive(Debug)]
pub struct Symbol {
    pub ident: Identifier,
    pub size_bytes: usize,
    pub rbp_offset: usize,
    pub initialized: bool,
}

pub type SymTable = HashMap<String, Symbol>;

#[derive(Debug)]
pub struct Env {
    prev: Option<NonNull<Env>>,
    symtable: SymTable,
    shadow_indices: HashMap<String, i32>,
    rbp: usize,
    current_rbp_offset: usize,
}

impl Env {
    fn new() -> Self {
        return Self {
            prev: None,
            symtable: HashMap::new(),
            shadow_indices: HashMap::new(),
            rbp: 0,
            current_rbp_offset: 0,
        };
    }

    fn get_mut(&mut self, ident: &str) -> Option<&mut Symbol> {
        let mut env_ptr = NonNull::from(self);
        loop {
            let env;
            unsafe {
                env = env_ptr.as_mut();
            }
            let symbol = env.symtable.get_mut(ident);
            if symbol.is_some() {
                return symbol;
            }
            match env.prev {
                None => return None,
                Some(env_prev) => env_ptr = env_prev,
            }
        }
    }

    pub fn get(&self, ident: &str) -> Option<&Symbol> {
        let mut env_ptr = NonNull::from(self);
        loop {
            let env;
            unsafe {
                env = env_ptr.as_ref();
            }
            let symbol = env.symtable.get(ident);
            if symbol.is_some() {
                return symbol;
            }
            match env.prev {
                None => return None,
                Some(env_prev) => env_ptr = env_prev,
            }
        }
    }

    fn decorate_ident(&self, ident: &mut Identifier) -> String {
        let shadow_index = self.get_shadow_index(&ident.lexeme);
        let mut lexeme = format!("{}_{}", ident.lexeme, shadow_index);
        std::mem::swap(&mut ident.lexeme, &mut lexeme);
        println!("Decorated Identifier: {:?}", ident);
        return lexeme;
    }

    fn get_shadow_index(&self, undecorated_lexeme: &str) -> i32 {
        let mut env_ptr = NonNull::from(self);
        loop {
            let env;
            unsafe {
                env = env_ptr.as_ref();
            }
            let shadow_index = env.shadow_indices.get(undecorated_lexeme);
            if shadow_index.is_some() {
                return *shadow_index.unwrap();
            }
            match env.prev {
                None => return -1,
                Some(env_prev) => env_ptr = env_prev,
            }
        }
    }

    fn declare_aux(&mut self, ident: &mut Identifier, initialized: bool) {
        let symbol = self.symtable.get(&ident.lexeme);
        match symbol {
            Some(ident) => {
                panic!("[Env.insert] redeclaring identifier without encorporating name shadowing: {:?}", ident);
            }
            None => {
                let shadow_index = self.shadow_indices.get(&ident.lexeme).unwrap_or(&0);
                self.shadow_indices
                    .insert(ident.lexeme.clone(), shadow_index + 1);
                self.decorate_ident(ident);

                self.current_rbp_offset += 8;

                self.symtable.insert(
                    ident.lexeme.clone(),
                    Symbol {
                        ident: ident.clone(),
                        size_bytes: 8,
                        rbp_offset: self.current_rbp_offset,
                        initialized,
                    },
                );
            }
        }
    }

    fn declare(&mut self, ident: &mut Identifier) {
        self.declare_aux(ident, false)
    }
    fn init(&mut self, ident: &mut Identifier) {
        self.declare_aux(ident, true)
    }
}

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
