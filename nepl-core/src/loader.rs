use crate::ast::{self, Directive, Module, Stmt};
use crate::diagnostic::Severity;
use crate::error::CoreError;
use crate::lexer;
use crate::parser;
use crate::span::FileId;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use alloc::string::ToString;
use core::result::Result;
use std::fs;
use std::path::PathBuf;
extern crate std;

/// Holds all loaded sources and their assigned FileId.
#[derive(Debug, Clone)]
pub struct SourceMap {
    files: Vec<(PathBuf, String)>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    pub fn add(&mut self, path: PathBuf, src: String) -> FileId {
        let id = FileId(self.files.len() as u32);
        self.files.push((path, src));
        id
    }

    pub fn get(&self, id: FileId) -> Option<&str> {
        self.files.get(id.0 as usize).map(|(_, s)| s.as_str())
    }
}

/// Loader that builds a single merged module from an entry file,
/// preserving FileId/Span for diagnostics. #include inlines AST;
/// #import loads the module once and inlines its items (simple model).
#[derive(Debug)]
pub struct Loader {
    stdlib_root: PathBuf,
}

impl Loader {
    pub fn new(stdlib_root: PathBuf) -> Self {
        Self { stdlib_root }
    }

    /// Load an already-provided source string as a pseudo file (for stdin use).
    pub fn load_inline(
        &self,
        path: PathBuf,
        src: String,
    ) -> Result<(Module, SourceMap), CoreError> {
        let mut sm = SourceMap::new();
        let mut cache: BTreeMap<PathBuf, Module> = BTreeMap::new();
        let mut processing: BTreeSet<PathBuf> = BTreeSet::new();
        let mut imported: BTreeSet<PathBuf> = BTreeSet::new();
        let file_id = sm.add(path.clone(), src.clone());
        let module = self.parse_module(
            path,
            file_id,
            src,
            &mut sm,
            &mut cache,
            &mut processing,
            &mut imported,
        )?;
        Ok((module, sm))
    }

    pub fn load(&self, entry: &PathBuf) -> Result<(Module, SourceMap), CoreError> {
        let mut sm = SourceMap::new();
        let mut cache: BTreeMap<PathBuf, Module> = BTreeMap::new();
        let mut processing: BTreeSet<PathBuf> = BTreeSet::new();
        let mut imported: BTreeSet<PathBuf> = BTreeSet::new();
        let module = self.load_file(entry, &mut sm, &mut cache, &mut processing, &mut imported)?;
        Ok((module, sm))
    }

    fn load_file(
        &self,
        path: &PathBuf,
        sm: &mut SourceMap,
        cache: &mut BTreeMap<PathBuf, Module>,
        processing: &mut BTreeSet<PathBuf>,
        imported_once: &mut BTreeSet<PathBuf>,
    ) -> Result<Module, CoreError> {
        let canon = path
            .canonicalize()
            .map_err(|e| CoreError::Io(e.to_string()))?;
        if let Some(m) = cache.get(&canon) {
            return Ok(m.clone());
        }
        if !processing.insert(canon.clone()) {
            return Err(CoreError::Io(format!(
                "circular import/include detected at {:?}",
                canon
            )));
        }
        let src = fs::read_to_string(&canon).map_err(|e| CoreError::Io(e.to_string()))?;
        let file_id = sm.add(canon.clone(), src.clone());
        let mut module = self.parse_module(
            canon.clone(),
            file_id,
            src,
            sm,
            cache,
            processing,
            imported_once,
        )?;
        // process includes/imports
        let mut directives = module.directives.clone();
        let mut items = Vec::new();
        for stmt in module.root.items.clone() {
            match &stmt {
                Stmt::Directive(Directive::Import { path, .. }) => {
                    let target = self.resolve_path(&canon, path);
                    if imported_once.insert(target.clone()) {
                        let imp_mod =
                            self.load_file(&target, sm, cache, processing, imported_once)?;
                        directives.extend(imp_mod.directives.clone());
                        items.extend(imp_mod.root.items.clone());
                    }
                }
                Stmt::Directive(Directive::Include { path, .. }) => {
                    let target = self.resolve_path(&canon, path);
                    let inc_mod = self.load_file(&target, sm, cache, processing, imported_once)?;
                    directives.extend(inc_mod.directives.clone());
                    items.extend(inc_mod.root.items.clone());
                }
                _ => items.push(stmt),
            }
        }
        module.directives = directives;
        module.root.items = items;
        processing.remove(&canon);
        cache.insert(canon.clone(), module.clone());
        Ok(module)
    }

    fn parse_module(
        &self,
        _path: PathBuf,
        file_id: FileId,
        src: String,
        _sm: &mut SourceMap,
        _cache: &mut BTreeMap<PathBuf, Module>,
        _processing: &mut BTreeSet<PathBuf>,
        _imported_once: &mut BTreeSet<PathBuf>,
    ) -> Result<Module, CoreError> {
        let lex = lexer::lex(file_id, &src);
        if lex
            .diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
        {
            return Err(CoreError::from_diagnostics(lex.diagnostics));
        }
        let parse = parser::parse_tokens(file_id, lex);
        if parse.module.is_none() {
            return Err(CoreError::from_diagnostics(parse.diagnostics));
        }
        Ok(parse.module.unwrap())
    }

    fn resolve_path(&self, base: &PathBuf, spec: &str) -> PathBuf {
        let mut p = if spec.starts_with("std/") {
            self.stdlib_root.join(spec)
        } else {
            base.parent()
                .map(|p| p.join(spec))
                .unwrap_or_else(|| PathBuf::from(spec))
        };
        if p.extension().is_none() {
            p = p.with_extension("nepl");
        }
        p
    }
}
