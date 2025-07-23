use crate::{ConfigType, RunArgs};
use dynamic::runtime_export::{AssemblyManagerTrait, VMTrait};
use global::configs::runtime::VMConfig;
use libloading::Library;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::sync::{Arc, OnceLock};

pub struct VM {
    inner: Arc<dyn VMTrait>,
}

static RUNTIME_CORE: OnceLock<Library> = OnceLock::new();

fn init<P: AsRef<Path>>(core_path: P) -> global::Result<&'static Library> {
    unsafe { Ok(RUNTIME_CORE.get_or_try_init(|| Library::new(core_path.as_ref()))?) }
}

impl VM {
    pub fn new<P: AsRef<Path>>(p: P) -> global::Result<Self> {
        let lib = init(p)?;
        unsafe {
            let new_fn =
                lib.get::<extern "Rust" fn() -> global::Result<Arc<dyn VMTrait>>>(b"NewVM\0")?;
            let inner = new_fn()?;
            Ok(Self { inner })
        }
    }
    pub fn with_config<P: AsRef<Path>>(p: P, config: VMConfig) -> global::Result<Self> {
        let lib = init(p)?;
        unsafe {
            let new_fn = lib
                .get::<extern "Rust" fn(VMConfig) -> global::Result<Arc<dyn VMTrait>>>(
                    b"NewVMWithConfig\0",
                )?;
            let inner = new_fn(config)?;
            Ok(Self { inner })
        }
    }
    #[allow(unused)]
    pub fn with_config_assembly_manager<P: AsRef<Path>>(
        p: P,
        config: VMConfig,
        assembly_manager: Arc<dyn AssemblyManagerTrait>,
    ) -> global::Result<Self> {
        let lib = init(p)?;
        unsafe {
            let new_fn = lib.get::<extern "Rust" fn(
                VMConfig,
                Arc<dyn AssemblyManagerTrait>,
            ) -> global::Result<Arc<dyn VMTrait>>>(
                b"NewVMWithConfigAssemblyManager\0"
            )?;
            let inner = new_fn(config, assembly_manager)?;
            Ok(Self { inner })
        }
    }
}

impl Deref for VM {
    type Target = Arc<dyn VMTrait>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for VM {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[allow(unused)]
pub struct AssemblyManager {
    inner: Arc<dyn AssemblyManagerTrait>,
}

impl AssemblyManager {
    #[allow(unused)]
    pub fn new<P: AsRef<Path>>(p: P) -> global::Result<Self> {
        let lib = init(p)?;
        unsafe {
            let new_fn = lib
                .get::<extern "Rust" fn() -> global::Result<Arc<dyn AssemblyManagerTrait>>>(
                    b"NewAssemblyManager\0",
                )?;
            let inner = new_fn()?;
            Ok(Self { inner })
        }
    }
}

impl Deref for AssemblyManager {
    type Target = Arc<dyn AssemblyManagerTrait>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for AssemblyManager {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub fn handle(args: RunArgs) -> global::Result<u64> {
    let vm = match &args.config_path {
        None => VM::new(&args.core)?,
        Some(config_path) => {
            let config = match args.config_type {
                ConfigType::Json => serde_json::from_slice(&std::fs::read(config_path)?)?,
            };
            VM::with_config(&args.core, config)?
        }
    };
    let mut assemblies = Vec::new();
    for assem_path in &args.assemblies {
        assemblies.push(dynamic::runtime_export::binary::Assembly::from_file(
            assem_path,
        )?);
    }
    vm.assembly_manager()
        .load_from_binary_assemblies(&assemblies)?;
    vm.clone().load_statics()?;
    vm.clone().new_cpu().1.run(
        args.main_assembly_name.into(),
        args.main_class_name.into(),
        args.arguments,
    )
}
