use std::{collections::HashSet, fs::File, io::Write, process::Command};

use crate::{
    parser::{
        Identifier, IntLiteral, LExpression as LExp, Program, RExpression as RExp,
        Statement as Stmt, Term,
    },
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
            Term::Ident(ident) => self.ident(ident, symtable),
            Term::IntLit(intlit) => self.intlit(intlit),
        }
    }

    fn ident(&mut self, ident: &Identifier, symtable: &SymTable) {
        let sym = symtable.get(&ident.lexeme).expect(&format!(
            "[AsmGen.rexp] Identifier `{}` in program is not present in symtable",
            ident.lexeme
        ));

        self.stmt(&format!("mov ebx, dword [rbp-{}]", sym.rbp_offset));
    }

    fn intlit(&mut self, intlit: &IntLiteral) {
        self.stmt(&format!("mov ebx, {}", intlit.lexeme));
    }

    fn rexp(&mut self, rexp: &RExp, symtable: &SymTable) {
        match rexp {
            RExp::AddTerms(term1, term2) => {
                self.term(term1, symtable);
                self.stmt("add eax, ebx");
                self.term(term2, symtable);
                self.stmt("add eax, ebx");
            }
            RExp::Add(rexp, term) => {
                self.rexp(rexp, symtable);
                self.term(term, symtable);
                self.stmt("add eax, ebx");
            }
            RExp::IntLiteral(intlit) => {
                self.intlit(intlit);
                self.stmt("mov eax, ebx");
            }
            RExp::LExp(LExp::Ident(ident)) => {
                self.ident(ident, symtable);
                self.stmt("mov eax, ebx");
            }
        }
    }

    pub fn genasm(&mut self, program: &Program, symtable: SymTable) {
        self.label("_start");
        self.stmt("mov rbp, rsp");

        for stmt in program.iter() {
            match stmt {
                Stmt::Declare(ident) => {
                    self.stmt("");
                    self.comment(&format!("let {}", ident.lexeme));
                    self.stmt("sub rsp, 4");
                }
                Stmt::Initialize(l_ident, rexp) => {
                    let l_sym = symtable.get(&l_ident.lexeme).expect(&format!(
                        "[AsmGen] Identifier `{}` in program is not present in symtable",
                        l_ident.lexeme
                    ));
                    self.stmt("");
                    self.comment(&format!("let {} = {}", l_ident.lexeme, rexp));
                    self.stmt("sub rsp, 4");
                    self.stmt("xor eax, eax");
                    self.rexp(rexp, &symtable);
                    self.stmt(&format!("mov dword [rbp-{}], eax", l_sym.rbp_offset));
                }
                Stmt::Assign(lexp, rexp) => {
                    let LExp::Ident(l_ident) = lexp;
                    let l_sym = symtable.get(&l_ident.lexeme).expect(&format!(
                        "[AsmGen] Identifier `{}` in program is not present in symtable",
                        l_ident.lexeme
                    ));
                    self.stmt("");
                    self.comment(&format!("{} = {}", l_ident.lexeme, rexp));
                    self.rexp(rexp, &symtable);
                    self.stmt(&format!("mov dword [rbp-{}], eax", l_sym.rbp_offset));
                }
                _ => {}
            }
        }

        self.stmt("");
        self.comment("Exit with exit code 0");
        self.stmt("xor rcx, rcx");
        self.stmt("call ExitProcess");
    }
}
