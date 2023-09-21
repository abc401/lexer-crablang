use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Write,
    process::Command,
    ptr::NonNull,
};

use crate::{
    parser::{Identifier, IntLiteral, LExp, Program, RExp, Stmt, Term},
    CompileError,
};

#[derive(Debug)]
pub struct Symbol {
    pub decorated_lexeme: String,
    pub size_bytes: usize,
    pub rbp_offset: usize,
    pub initialized: bool,
}

struct SymbolBuilder {
    decorated_lexeme: Option<String>,
    size_bytes: Option<usize>,
    rbp_offset: Option<usize>,
    initialized: Option<bool>,
}

impl SymbolBuilder {
    pub fn new() -> Self {
        return Self {
            decorated_lexeme: None,
            size_bytes: None,
            rbp_offset: None,
            initialized: None,
        };
    }
    pub fn decorated_lexeme(&mut self, decorated_lexeme: String) -> &mut Self {
        self.decorated_lexeme = Some(decorated_lexeme);
        return self;
    }
    pub fn size_bytes(&mut self, bytes: usize) -> &mut Self {
        self.size_bytes = Some(bytes);
        return self;
    }
    pub fn rbp_offset(&mut self, offset: usize) -> &mut Self {
        self.rbp_offset = Some(offset);
        return self;
    }
    pub fn initialized(&mut self, initialized: bool) -> &mut Self {
        self.initialized = Some(initialized);
        return self;
    }
    pub fn build(&self) -> Symbol {
        let self_decorated_lexeme = unsafe {
            let ptr = &self.decorated_lexeme as *const Option<String> as *mut Option<String>;
            ptr.as_mut().unwrap()
        };
        let decorated_lexeme = std::mem::take(self_decorated_lexeme);
        return Symbol {
            decorated_lexeme: decorated_lexeme.unwrap(),
            size_bytes: self.size_bytes.unwrap(),
            rbp_offset: self.rbp_offset.unwrap(),
            initialized: self.initialized.unwrap(),
        };
    }
}

pub type SymTable = HashMap<String, Symbol>;

pub struct Env {
    prev: Option<NonNull<Env>>,
    symtable: SymTable,
    shadow_counts: HashMap<String, u32>,
    current_rbp_offset: usize,
}

impl Env {
    pub fn new() -> Self {
        Self {
            prev: None,
            symtable: HashMap::new(),
            shadow_counts: HashMap::new(),
            current_rbp_offset: 0,
        }
    }

    fn with_tail(tail: &Env) -> Self {
        Self {
            prev: Some(NonNull::from(tail)),
            symtable: HashMap::new(),
            shadow_counts: HashMap::new(),
            current_rbp_offset: tail.current_rbp_offset,
        }
    }

    fn get_shadow_count_in_head_mut(&mut self, lexeme: &str) -> &mut u32 {
        let count = self.shadow_counts.get_mut(lexeme);
        if count.is_some() {
            return unsafe { NonNull::from(count.unwrap()).as_mut() };
        } else {
            self.shadow_counts.insert(String::from(lexeme), 0);
            return self.shadow_counts.get_mut(lexeme).unwrap();
        }
    }
    fn get_shadow_count_in_head(&self, lexeme: &str) -> u32 {
        match self.shadow_counts.get(lexeme) {
            Some(count) => *count,
            None => 0,
        }
    }

    fn get_symbol(&self, lexeme: &str) -> Option<&Symbol> {
        let shadow_count = self.get_shadow_count_in_head(lexeme);
        let decorated_lexeme = format!("{}_{}", lexeme, shadow_count);
        match self.symtable.get(&decorated_lexeme) {
            Some(sym) => return Some(sym),
            None => {
                if self.prev.is_none() {
                    return None;
                }
                let env = unsafe { self.prev.unwrap().as_ref() };
                return env.get_symbol(lexeme);
            }
        }
    }

    fn register_symbol(&mut self, lexeme: &str, symbol_builder: &mut SymbolBuilder) {
        let shadow_count = self.get_shadow_count_in_head_mut(lexeme);
        *shadow_count += 1;
        let decorated_lexeme = format!("{}_{}", lexeme, shadow_count);
        self.current_rbp_offset += 8;
        self.symtable.insert(
            decorated_lexeme.clone(),
            symbol_builder
                .rbp_offset(self.current_rbp_offset)
                .decorated_lexeme(decorated_lexeme)
                .build(),
        );
    }

    fn declare(&mut self, ident: &Identifier) {
        self.register_symbol(
            &ident.lexeme,
            SymbolBuilder::new().size_bytes(8).initialized(false),
        );
    }
    fn initialize(&mut self, ident: &Identifier) {
        self.register_symbol(
            &ident.lexeme,
            SymbolBuilder::new().size_bytes(8).initialized(true),
        );
    }
}

#[derive(Debug)]
pub struct Asm {
    link_files: HashSet<String>,
    externals: Vec<String>,
    text: String,
}

impl Default for Asm {
    fn default() -> Self {
        return Self {
            link_files: HashSet::from(["C:/windows/system32/kernel32.dll".into()]),
            externals: vec!["ExitProcess".into()],
            text: Default::default(),
        };
    }
}

impl Asm {
    pub fn gen(&mut self, program: &Program, env: &mut Env) -> Result<(), CompileError> {
        self.label("_start");
        self.stmt("mov rbp, rsp");

        for stmt in program.iter() {
            match stmt {
                Stmt::Declare(ident) => {
                    env.declare(ident);
                    let sym = env.get_symbol(&ident.lexeme).expect(&format!(
                        "[AsmGen.gen] Identifier {:?} was not declared properly.",
                        ident
                    ));
                    let lexeme = &sym.decorated_lexeme;
                    self.stmt("");
                    self.comment_emp(format!("let {}", lexeme));
                    self.stmt(format!("sub rsp, {}", sym.size_bytes));
                }
                Stmt::Initialize(l_ident, rexp) => {
                    self.stmt("");
                    self.comment_emp(format!("let {} = {}", l_ident, rexp));
                    self.stmt("");

                    self.rexp(rexp, env)?;

                    env.initialize(l_ident);
                    let l_sym = env.get_symbol(&l_ident.lexeme).expect(&format!(
                        "[AsmGen.gen] Identifier {:?} was not initialized properly.",
                        l_ident
                    ));
                    let lexeme = &l_sym.decorated_lexeme;

                    self.stmt("pop rax");

                    self.stmt("");
                    self.comment(&format!("let {} = {}", lexeme, rexp));
                    self.stmt(&format!("sub rsp, {}", l_sym.size_bytes));
                    self.stmt(&format!("mov qword [rbp-{}], rax", l_sym.rbp_offset));
                }
                Stmt::Assign(lexp, rexp) => {
                    let LExp::Ident(l_ident) = lexp;
                    let l_sym = env.get_symbol(&l_ident.lexeme);
                    let l_sym = match l_sym {
                        Some(sym) => sym,
                        None => return Err(CompileError::UseOfUndeclaredIdent(l_ident.clone())),
                    };
                    let lexeme = &l_sym.decorated_lexeme;
                    self.stmt("");
                    self.comment_emp(format!("{} = {}", lexeme, rexp));
                    self.rexp(rexp, env)?;

                    self.stmt("");
                    self.comment(format!("{} = {}", lexeme, rexp));
                    self.stmt("pop rax");
                    self.stmt(&format!("mov qword [rbp-{}], rax", l_sym.rbp_offset));
                }
                Stmt::RExp(rexp) => {
                    self.comment_emp(format!("{}", rexp));
                    self.rexp(rexp, env)?;
                }
                Stmt::Exit(rexp) => {
                    self.comment_emp("");
                    self.rexp(rexp, env)?;
                    self.stmt("");
                    self.comment(format!("exit {}", rexp));
                    self.stmt("mov rcx, rax");
                    self.stmt("call ExitProcess");
                }

                _ => panic!("[Assembly Generation] Not implemented for Stmt: {}", stmt),
            }
        }

        self.stmt("");
        self.comment_emp("Exit with exit code 0");
        self.stmt("xor rcx, rcx");
        self.stmt("call ExitProcess");
        return Ok(());
    }
    fn stmt<'a>(&mut self, stmt: impl AsRef<str>) {
        self.text.push_str("    ");
        self.text.push_str(stmt.as_ref());
        self.text.push('\n');
    }

    fn label(&mut self, label: impl AsRef<str>) {
        self.text.push_str(label.as_ref());
        self.text.push_str(":\n");
    }

    fn comment(&mut self, comment: impl AsRef<str>) {
        self.text.push_str("    ; ");
        self.text.push_str(comment.as_ref());
        self.text.push('\n');
    }
    fn comment_emp(&mut self, comment: impl AsRef<str>) {
        self.text.push_str("    ; ################# ");
        self.text.push_str(comment.as_ref());
        self.text.push_str(" #################");
        self.text.push('\n');
    }

    pub fn write_to_file(&self, filename: impl AsRef<str>) -> std::io::Result<()> {
        let filename = filename.as_ref();
        let mut outfile = File::create(format!("{filename}.asm"))?;
        outfile.write_all("default rel\nglobal _start\n".as_bytes())?;

        outfile.write_all("extern ".as_bytes())?;
        for ext in self.externals.iter() {
            outfile.write_all(ext.as_bytes())?;
            outfile.write_all(", ".as_bytes())?;
        }
        outfile.write_all("\n".as_bytes())?;

        outfile.write_all("section .text\n".as_bytes())?;
        outfile.write_all(self.text.as_bytes())?;

        return Ok(());
    }

    pub fn compile(&self, filename: impl AsRef<str>) -> std::io::Result<()> {
        let filename = filename.as_ref();
        self.write_to_file(filename)?;
        Command::new("nasm")
            .args([
                "-f",
                "win64",
                &format!("{filename}.asm"),
                "-o",
                &format!("{filename}.obj"),
            ])
            .output()?;

        let mut gcc_args = vec![
            "-g".into(),
            "-nostdlib".into(),
            "-o".into(),
            format!("{filename}.exe"),
            format!("{filename}.obj"),
        ];
        gcc_args.extend(self.link_files.iter().map(|l| l.clone()));

        Command::new("gcc").args(gcc_args).output()?;
        return Ok(());
    }

    fn term(&mut self, term: &Term, env: &Env) -> Result<(), CompileError> {
        match term {
            Term::LExp(LExp::Ident(ident)) => self.ident(ident, env),
            Term::IntLit(intlit) => self.intlit(intlit),
            Term::Neg(inner_term) => {
                self.term(inner_term, env)?;
                self.stmt("pop rax");
                self.stmt("");
                self.comment(format!("{}", term));
                self.stmt("neg rax");
                self.stmt("push rax");
                return Ok(());
            }
            Term::Bracketed(rexp) => self.rexp(rexp, env),

            _ => panic!("[Assembly Generation] Not implemented for term: {}", term),
        }
    }

    fn ident(&mut self, ident: &Identifier, env: &Env) -> Result<(), CompileError> {
        let sym = env.get_symbol(&ident.lexeme);
        let sym = match sym {
            Some(sym) => sym,
            None => return Err(CompileError::UseOfUndeclaredIdent(ident.clone())),
        };
        let lexeme = &sym.decorated_lexeme;

        self.stmt("");
        self.comment(lexeme);
        self.stmt(format!("push qword [rbp-{}]", sym.rbp_offset));
        return Ok(());
    }

    fn intlit(&mut self, intlit: &IntLiteral) -> Result<(), CompileError> {
        self.stmt("");
        self.comment(&intlit.lexeme);
        self.stmt(format!("mov rax, {}", intlit.lexeme));
        self.stmt("push rax");
        return Ok(());
    }

    fn binary_operator<F>(
        &mut self,
        bin_exp: &RExp,
        lhs: &RExp,
        rhs: &RExp,
        env: &Env,
        asm_gen: &mut F,
    ) -> Result<(), CompileError>
    where
        F: FnMut(&mut Self) -> (),
    {
        self.rexp(lhs, env)?;
        self.rexp(rhs, env)?;

        self.stmt("");
        self.comment(&format!("{}", bin_exp));

        self.stmt("pop rbx");
        self.stmt("pop rax");

        asm_gen(self);

        self.stmt("push rax");
        return Ok(());
    }

    fn rexp(&mut self, rexp: &RExp, env: &Env) -> Result<(), CompileError> {
        match rexp {
            RExp::Add(lhs, rhs) => self.binary_operator(rexp, lhs, rhs, env, &mut |asm| {
                asm.stmt("add rax, rbx");
            }),
            RExp::Term(term) => self.term(term, env),
            RExp::Sub(lhs, rhs) => self.binary_operator(rexp, lhs, rhs, env, &mut |asm| {
                asm.stmt("sub rax, rbx");
            }),
            RExp::Mul(lhs, rhs) => self.binary_operator(rexp, lhs, rhs, env, &mut |asm| {
                asm.stmt("mul rbx");
            }),
            RExp::Div(lhs, rhs) => self.binary_operator(rexp, lhs, rhs, env, &mut |asm| {
                asm.stmt("xor rdx, rdx");
                asm.stmt("div rbx");
            }),
            RExp::Equal(lhs, rhs) => self.binary_operator(rexp, lhs, rhs, env, &mut |asm| {
                asm.stmt("cmp rax, rbx");
                asm.stmt("sete al");
                asm.stmt("and rax, 255");
            }),
            RExp::NotEqual(lhs, rhs) => self.binary_operator(rexp, lhs, rhs, env, &mut |asm| {
                asm.stmt("cmp rax, rbx");
                asm.stmt("setne al");
                asm.stmt("and rax, 255");
            }),
            RExp::Less(lhs, rhs) => self.binary_operator(rexp, lhs, rhs, env, &mut |asm| {
                asm.stmt("cmp rax, rbx");
                asm.stmt("setl al");
                asm.stmt("and rax, 255");
            }),
            RExp::LessEqual(lhs, rhs) => self.binary_operator(rexp, lhs, rhs, env, &mut |asm| {
                asm.stmt("cmp rax, rbx");
                asm.stmt("setle al");
                asm.stmt("and rax, 255");
            }),
            RExp::Greater(lhs, rhs) => self.binary_operator(rexp, lhs, rhs, env, &mut |asm| {
                asm.stmt("cmp rax, rbx");
                asm.stmt("setg al");
                asm.stmt("and rax, 255");
            }),
            RExp::GreaterEqual(lhs, rhs) => self.binary_operator(rexp, lhs, rhs, env, &mut |asm| {
                asm.stmt("cmp rax, rbx");
                asm.stmt("setge al");
                asm.stmt("and rax, 255");
            }),
            _ => panic!("[Assembly Generation] Not implemented for RExp: {}", rexp),
        }
        // return Ok(());
    }
}
