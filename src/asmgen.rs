use std::{
    collections::HashSet,
    fs::File,
    io::{Error, Result, Write},
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    parser::{LExpression as LE, Program, RExpression as RE, Statement as Stmt},
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
}

pub fn genasm(program: &Program, symtable: SymTable) -> AsmCode {
    let mut asm = AsmCode::default();
    asm.label("_start");
    asm.stmt("mov rbp, rsp");

    for stmt in program.iter() {
        match stmt {
            Stmt::Declare(ident) => {
                asm.stmt("");
                asm.comment(&format!("let {}", ident));
                asm.stmt("sub rsp, 4");
            }
            Stmt::Initialize(l_ident, rexp) => {
                let l_sym = symtable.get(l_ident).expect(&format!(
                    "[AsmGen] Identifier `{}` in program is not present in symtable",
                    l_ident
                ));
                match rexp {
                    RE::Ident(r_ident) => {
                        let r_sym = symtable.get(r_ident).expect(&format!(
                            "[AsmGen] Identifier `{}` in program is not present in symtable",
                            r_ident
                        ));

                        asm.stmt("");
                        asm.comment(&format!("let {} = {}", l_ident, r_ident));
                        asm.stmt("sub rsp, 4");
                        asm.stmt(&format!("mov rax, qword [rbp-{}]", r_sym.rbp_offset));
                        asm.stmt(&format!("mov qword [rbp-{}], rax", l_sym.rbp_offset));
                        asm.stmt("");
                    }
                    RE::IntLiteral(intlit) => {
                        asm.stmt("");
                        asm.comment(&format!("let {} = {}", l_ident, intlit));
                        asm.stmt("sub rsp, 4");
                        asm.stmt(&format!("mov qword [rbp-{}], {}", l_sym.rbp_offset, intlit));
                    }
                }
            }
            Stmt::Assign(lexp, rexp) => {
                let LE::Ident(l_ident) = lexp;
                let l_sym = symtable.get(l_ident).expect(&format!(
                    "[AsmGen] Identifier `{}` in program is not present in symtable",
                    l_ident
                ));
                match rexp {
                    RE::Ident(r_ident) => {
                        let r_sym = symtable.get(r_ident).expect(&format!(
                            "[AsmGen] Identifier `{}` in program is not present in symtable",
                            r_ident
                        ));

                        asm.stmt("");
                        asm.comment(&format!("{} = {}", l_ident, r_ident));
                        asm.stmt(&format!("mov rax, qword [rbp-{}]", r_sym.rbp_offset));
                        asm.stmt(&format!("mov qword [rbp-{}], rax", l_sym.rbp_offset));
                    }
                    RE::IntLiteral(intlit) => {
                        asm.stmt("");
                        asm.comment(&format!("{} = {}", l_ident, intlit));
                        asm.stmt(&format!("mov qword [rbp-{}], {}", l_sym.rbp_offset, intlit));
                    }
                }
            }
        }
    }

    asm.stmt("");
    asm.comment("Exit with exit code 0");
    asm.stmt("xor rcx, rcx");
    asm.stmt("call ExitProcess");
    return asm;
}
