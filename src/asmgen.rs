use std::{collections::HashSet, fs::File, io::Write, process::Command};

use crate::{
    parser::{Identifier, IntLiteral, LExp, Program, RExp, Statement as Stmt, Term},
    semantic_anal::SymTable,
};

#[derive(Debug)]
pub struct AsmCode {
    link_files: HashSet<String>,
    externals: Vec<String>,
    text: String,
}

impl Default for AsmCode {
    fn default() -> Self {
        return Self {
            link_files: HashSet::from(["C:/windows/system32/kernel32.dll".into()]),
            externals: vec!["ExitProcess".into()],
            text: Default::default(),
        };
    }
}

impl AsmCode {
    fn stmt(&mut self, stmt: &str) {
        self.text.push_str("    ");
        self.text.push_str(stmt);
        self.text.push('\n');
    }

    fn label(&mut self, label: &str) {
        self.text.push_str(label);
        self.text.push_str(":\n");
    }

    fn comment(&mut self, comment: &str) {
        self.text.push_str("    ; ");
        self.text.push_str(comment);
        self.text.push('\n');
    }
    fn comment_emp(&mut self, comment: &str) {
        self.text.push_str("    ; ################# ");
        self.text.push_str(comment);
        self.text.push_str(" #################");
        self.text.push('\n');
    }

    pub fn write_to_file(&self, filename: &str) -> std::io::Result<()> {
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

    pub fn compile(&self, filename: &str) -> std::io::Result<()> {
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

    fn term(&mut self, term: &Term, symtable: &SymTable) {
        match term {
            Term::LExp(LExp::Ident(ident)) => self.ident(ident, symtable),
            Term::IntLit(intlit) => self.intlit(intlit),

            _ => panic!("[Assembly Generation] Not implemented: {}", term),
        }
    }

    fn ident(&mut self, ident: &Identifier, symtable: &SymTable) {
        let sym = symtable.get(&ident.lexeme).expect(&format!(
            "[AsmGen.rexp] Identifier `{}` in program is not present in symtable",
            ident.lexeme
        ));

        self.stmt("");
        self.comment(&ident.lexeme);
        self.stmt(&format!("push qword [rbp-{}]", sym.rbp_offset));
    }

    fn intlit(&mut self, intlit: &IntLiteral) {
        self.stmt("");
        self.comment(&intlit.lexeme);
        self.stmt(&format!("mov rax, {}", intlit.lexeme));
        self.stmt("push rax");
    }

    fn rexp(&mut self, rexp: &RExp, symtable: &SymTable) {
        match rexp {
            RExp::Add(lhs, rhs) => {
                self.rexp(lhs, symtable);
                self.rexp(rhs, symtable);

                self.stmt("");
                self.comment(&format!("{}", rexp));
                self.stmt("pop rbx");
                self.stmt("pop rax");
                self.stmt("add rax, rbx");
                self.stmt("push rax");
            }
            RExp::Term(term) => {
                self.term(term, symtable);
            }
            RExp::Sub(lhs, rhs) => {
                self.rexp(lhs, symtable);
                self.rexp(rhs, symtable);

                self.stmt("");
                self.comment(&format!("{}", rexp));
                self.stmt("pop rbx");
                self.stmt("pop rax");
                self.stmt("sub rax, rbx");
                self.stmt("push rax");
            }
            RExp::Mul(lhs, rhs) => {
                self.rexp(lhs, symtable);
                self.rexp(rhs, symtable);

                self.stmt("");
                self.comment(&format!("{}", rexp));
                self.stmt("pop rbx");
                self.stmt("pop rax");
                self.stmt("mul rbx");
                self.stmt("push rax");
            }
            RExp::Div(lhs, rhs) => {
                self.rexp(lhs, symtable);
                self.rexp(rhs, symtable);

                self.stmt("");
                self.comment(&format!("{}", rexp));
                self.stmt("pop rbx");
                self.stmt("pop rax");
                self.stmt("xor rdx, rdx");
                self.stmt("div rbx");
                self.stmt("push rax");
            }
            RExp::Equal(lhs, rhs) => {
                self.rexp(lhs, symtable);
                self.rexp(rhs, symtable);

                self.stmt("");
                self.comment(&format!("{}", rexp));
                self.stmt("pop rbx");
                self.stmt("pop rax");
                self.stmt("cmp rax, rbx");
                self.stmt("sete al");
                self.stmt("and rax, 255");
                self.stmt("push rax");
            }
            RExp::NotEqual(lhs, rhs) => {
                self.rexp(lhs, symtable);
                self.rexp(rhs, symtable);

                self.stmt("");
                self.comment(&format!("{}", rexp));
                self.stmt("pop rbx");
                self.stmt("pop rax");
                self.stmt("cmp rax, rbx");
                self.stmt("setne al");
                self.stmt("and rax, 255");
                self.stmt("push rax");
            }
            RExp::Less(lhs, rhs) => {
                self.rexp(lhs, symtable);
                self.rexp(rhs, symtable);

                self.stmt("");
                self.comment(&format!("{}", rexp));
                self.stmt("pop rbx");
                self.stmt("pop rax");
                self.stmt("cmp rax, rbx");
                self.stmt("setl al");
                self.stmt("and rax, 255");
                self.stmt("push rax");
            }
            RExp::LessEqual(lhs, rhs) => {
                self.rexp(lhs, symtable);
                self.rexp(rhs, symtable);

                self.stmt("");
                self.comment(&format!("{}", rexp));
                self.stmt("pop rbx");
                self.stmt("pop rax");
                self.stmt("cmp rax, rbx");
                self.stmt("setle al");
                self.stmt("and rax, 255");
                self.stmt("push rax");
            }
            RExp::Greater(lhs, rhs) => {
                self.rexp(lhs, symtable);
                self.rexp(rhs, symtable);

                self.stmt("");
                self.comment(&format!("{}", rexp));
                self.stmt("pop rbx");
                self.stmt("pop rax");
                self.stmt("cmp rax, rbx");
                self.stmt("setg al");
                self.stmt("and rax, 255");
                self.stmt("push rax");
            }
            RExp::GreaterEqual(lhs, rhs) => {
                self.rexp(lhs, symtable);
                self.rexp(rhs, symtable);

                self.stmt("");
                self.comment(&format!("{}", rexp));
                self.stmt("pop rbx");
                self.stmt("pop rax");
                self.stmt("cmp rax, rbx");
                self.stmt("setge al");
                self.stmt("and rax, 255");
                self.stmt("push rax");
            }
            _ => panic!("[Assembly Generation] Not implemented: {}", rexp),
        }
    }

    pub fn genasm(&mut self, program: &Program, symtable: SymTable) {
        self.label("_start");
        self.stmt("mov rbp, rsp");

        for stmt in program.iter() {
            match stmt {
                Stmt::Declare(ident) => {
                    let sym = symtable.get(&ident.lexeme).expect(&format!(
                        "[AsmGen] Identifier `{}` in program is not present in symtable",
                        ident.lexeme
                    ));

                    self.stmt("");
                    self.comment_emp(&format!("let {}", ident.lexeme));
                    self.stmt(&format!("sub rsp, {}", sym.size_bytes));
                }
                Stmt::Initialize(l_ident, rexp) => {
                    let l_sym = symtable.get(&l_ident.lexeme).expect(&format!(
                        "[AsmGen] Identifier `{}` in program is not present in symtable",
                        l_ident.lexeme
                    ));
                    self.stmt("");
                    self.comment_emp(&format!("let {} = {}", l_ident.lexeme, rexp));

                    self.stmt("");
                    self.comment(&format!("let {}", l_ident.lexeme));
                    self.stmt(&format!("sub rsp, {}", l_sym.size_bytes));

                    self.rexp(rexp, &symtable);
                    self.stmt("pop rax");

                    self.stmt(&format!("mov qword [rbp-{}], rax", l_sym.rbp_offset));
                }
                Stmt::Assign(lexp, rexp) => {
                    let LExp::Ident(l_ident) = lexp;
                    let l_sym = symtable.get(&l_ident.lexeme).expect(&format!(
                        "[AsmGen] Identifier `{}` in program is not present in symtable",
                        l_ident.lexeme
                    ));
                    self.stmt("");
                    self.comment_emp(&format!("{} = {}", l_ident.lexeme, rexp));
                    self.rexp(rexp, &symtable);
                    self.stmt("pop rax");
                    self.stmt(&format!("mov qword [rbp-{}], rax", l_sym.rbp_offset));
                }
                Stmt::RExp(_) => {}

                _ => panic!("[Assembly Generation] Not implemented: {}", stmt),
            }
        }

        self.stmt("");
        self.comment_emp("Exit with exit code 0");
        self.stmt("xor rcx, rcx");
        self.stmt("call ExitProcess");
    }
}
