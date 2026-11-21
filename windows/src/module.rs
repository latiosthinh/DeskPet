//! Runs a pet module (`<id>.deskpet`, WebAssembly) in a sandbox: the wasmi interpreter, with no
//! imports (no files, network or system), a memory cap, and a fuel budget per call so a pet stuck
//! in a loop can't hang the app. Talks to it only through petkit's boundary types.

use petkit::engine::{About, Command, Input, Render};
use petkit::wire::{decode, encode, Wire};
use wasmi::{
    Config, Engine, Instance, Linker, Memory, Module, Store, StoreLimits, StoreLimitsBuilder,
    TypedFunc,
};

/// The pet API version this app speaks; a module built against a newer major version is refused.
const API_MAJOR: u32 = petkit::API_VERSION.0;
const MAX_MEMORY: usize = 64 << 20;
/// Instructions a module may run in one call: far more than a frame needs, small enough to stop
/// a runaway loop within a fraction of a second.
const FUEL: u64 = 200_000_000;

pub struct PetModule {
    store: Store<StoreLimits>,
    memory: Memory,
    inbox: TypedFunc<u32, u32>,
    about: TypedFunc<(), u64>,
    tick: TypedFunc<u32, ()>,
    command: TypedFunc<u32, u32>,
    render: TypedFunc<(), u64>,
}

fn err(what: &str) -> impl Fn(wasmi::Error) -> String + '_ {
    move |e| format!("{what}: {e}")
}

impl PetModule {
    /// Loads a module and starts its pet. Fails on anything that isn't a DeskPet module this app
    /// can run.
    pub fn load(bytes: &[u8], now: f64, seed: u64) -> Result<PetModule, String> {
        if !bytes.starts_with(b"\0asm") {
            return Err("not a DeskPet pet file".into());
        }
        let mut config = Config::default();
        config.consume_fuel(true);
        let engine = Engine::new(&config);
        let module = Module::new(&engine, bytes).map_err(err("damaged pet file"))?;
        if module.imports().next().is_some() {
            return Err("this pet file asks for things pets aren't allowed".into());
        }
        let mut store = Store::new(
            &engine,
            StoreLimitsBuilder::new()
                .memory_size(MAX_MEMORY)
                .instances(1)
                .build(),
        );
        store.limiter(|limits| limits);
        store.set_fuel(FUEL).map_err(err("fuel"))?;
        let linker = <Linker<StoreLimits>>::new(&engine);
        let instance: Instance = linker
            .instantiate_and_start(&mut store, &module)
            .map_err(err("couldn't start the pet"))?;

        let get = |name: &str| format!("not a DeskPet pet file (no {name})");
        let api = instance
            .get_typed_func::<(), u32>(&store, "deskpet_api")
            .map_err(|_| get("deskpet_api"))?;
        let version = api.call(&mut store, ()).map_err(err("pet"))?;
        if version >> 16 > API_MAJOR {
            return Err("this pet needs a newer DeskPet; please update the app".into());
        }
        let new = instance
            .get_typed_func::<(f64, u64), ()>(&store, "deskpet_new")
            .map_err(|_| get("deskpet_new"))?;
        let mut m = PetModule {
            memory: instance
                .get_memory(&store, "memory")
                .ok_or_else(|| get("memory"))?,
            inbox: instance
                .get_typed_func(&store, "deskpet_inbox")
                .map_err(|_| get("deskpet_inbox"))?,
            about: instance
                .get_typed_func(&store, "deskpet_about")
                .map_err(|_| get("deskpet_about"))?,
            tick: instance
                .get_typed_func(&store, "deskpet_tick")
                .map_err(|_| get("deskpet_tick"))?,
            command: instance
                .get_typed_func(&store, "deskpet_command")
                .map_err(|_| get("deskpet_command"))?,
            render: instance
                .get_typed_func(&store, "deskpet_render")
                .map_err(|_| get("deskpet_render"))?,
            store,
        };
        m.refuel()?;
        new.call(&mut m.store, (now, seed)).map_err(err("pet"))?;
        Ok(m)
    }

    fn refuel(&mut self) -> Result<(), String> {
        self.store.set_fuel(FUEL).map_err(err("fuel"))
    }

    /// Copies a request into the module's inbox; returns its length.
    fn send<T: Wire>(&mut self, v: &T) -> Result<u32, String> {
        let bytes = encode(v);
        self.refuel()?;
        let at = self
            .inbox
            .call(&mut self.store, bytes.len() as u32)
            .map_err(err("pet"))? as usize;
        let mem = self.memory.data_mut(&mut self.store);
        mem.get_mut(at..at + bytes.len())
            .ok_or("pet gave a bad address")?
            .copy_from_slice(&bytes);
        Ok(bytes.len() as u32)
    }

    /// Reads a reply the module left at `(address << 32) | length`.
    fn receive<T: Wire>(&self, at: u64) -> Result<T, String> {
        let (start, len) = ((at >> 32) as usize, (at & 0xffff_ffff) as usize);
        let mem = self.memory.data(&self.store);
        let bytes = mem
            .get(start..start + len)
            .ok_or("pet gave a bad address")?;
        decode(bytes).ok_or_else(|| "pet sent something this app can't read".into())
    }

    pub fn about(&mut self) -> Result<About, String> {
        self.refuel()?;
        let at = self.about.call(&mut self.store, ()).map_err(err("pet"))?;
        self.receive(at)
    }

    pub fn tick(&mut self, input: Input) -> Result<(), String> {
        let len = self.send(&input)?;
        self.tick.call(&mut self.store, len).map_err(err("pet"))
    }

    pub fn command(&mut self, c: Command) -> Result<bool, String> {
        let len = self.send(&c)?;
        Ok(self
            .command
            .call(&mut self.store, len)
            .map_err(err("pet"))?
            != 0)
    }

    pub fn render(&mut self) -> Result<Render, String> {
        self.refuel()?;
        let at = self.render.call(&mut self.store, ()).map_err(err("pet"))?;
        self.receive(at)
    }
}
