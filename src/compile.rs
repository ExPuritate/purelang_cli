#![allow(improper_ctypes_definitions)]

use crate::CompileArgs;
use dynamic::CompileServiceTrait;
use global::configs::compiler::CompileServiceConfig;
use libloading::Library;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

struct CompileService {
    ptr: Box<dyn CompileServiceTrait>,
}

static COMPILE_SERVICE_CORE: OnceLock<Library> = OnceLock::new();

impl CompileService {
    pub fn new<P: AsRef<Path>>(p: P) -> global::Result<Self> {
        unsafe {
            eprintln!("Loading compile service from {}", p.as_ref().display());
            let lib = COMPILE_SERVICE_CORE.get_or_try_init(|| Library::new(p.as_ref()))?;
            eprintln!("Compile service loaded");
            let new_fn = lib.get::<extern "Rust" fn() -> Box<dyn CompileServiceTrait>>(
                b"NewCompileService\0",
            )?;
            let ptr = new_fn();
            Ok(Self { ptr })
        }
    }
    pub fn with_config<P: AsRef<Path>>(p: P, config: CompileServiceConfig) -> global::Result<Self> {
        unsafe {
            eprintln!("Loading compile service from {}", p.as_ref().display());
            let lib = COMPILE_SERVICE_CORE.get_or_try_init(|| Library::new(p.as_ref()))?;
            eprintln!("Compile service loaded");
            let new_fn =
                lib.get::<extern "Rust" fn(CompileServiceConfig) -> Box<dyn CompileServiceTrait>>(
                    b"NewCompileServiceWithConfig\0",
                )?;
            let ptr = new_fn(config);
            Ok(Self { ptr })
        }
    }
}

impl Deref for CompileService {
    type Target = dyn CompileServiceTrait;
    fn deref(&self) -> &Self::Target {
        &*self.ptr
    }
}

impl DerefMut for CompileService {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.ptr
    }
}

pub fn handle(args: CompileArgs) -> global::Result<()> {
    let mut compile_service = match &args.config_path {
        Some(path) => CompileService::with_config(
            args.core.as_str(),
            serde_json::from_str(std::fs::read_to_string(path)?.as_str())?,
        )?,
        None => CompileService::new(args.core.as_str())?,
    };
    for compiler in &args.compilers {
        compile_service.load_compiler_from_path(compiler)?;
    }
    for source in &args.sources {
        compile_service.add_file(source)?;
    }
    compile_service.compile(|s| {
        dbg!(s);
        let mut p = PathBuf::from(s);
        p.set_extension("plb");
        Ok(p.to_str().unwrap().to_owned())
    })?;
    Ok(())
}
