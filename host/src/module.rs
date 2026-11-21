//! Runs a pet module (`<id>.deskpet`, WebAssembly) in a sandbox: the wasmi interpreter, with no
//! imports (no files, network or system), a memory cap, and a fuel budget per call so a pet stuck
//! in a loop can't hang the app. Talks to it only through petkit's boundary types.

use petkit::engine::{About, Command, Input, Render};
use petkit::wire::{decode, encode, Wire};
use wasmi::{Config, Engine, Instance, Linker, Memory, Module, Store, StoreLimits, StoreLimitsBuilder, TypedFunc};

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
    /// Optional: the message of the panic that stopped the module.
    panic: Option<TypedFunc<(), u64>>,
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
        let mut store = Store::new(&engine, StoreLimitsBuilder::new().memory_size(MAX_MEMORY).instances(1).build());
        store.limiter(|limits| limits);
        store.set_fuel(FUEL).map_err(err("fuel"))?;
        let linker = <Linker<StoreLimits>>::new(&engine);
        let instance: Instance = linker.instantiate_and_start(&mut store, &module).map_err(err("couldn't start the pet"))?;

        let get = |name: &str| format!("not a DeskPet pet file (no {name})");
        let api = instance.get_typed_func::<(), u32>(&store, "deskpet_api").map_err(|_| get("deskpet_api"))?;
        let version = api.call(&mut store, ()).map_err(err("pet"))?;
        if version >> 16 > API_MAJOR {
            return Err("this pet needs a newer DeskPet; please update the app".into());
        }
        let new = instance.get_typed_func::<(f64, u64), ()>(&store, "deskpet_new").map_err(|_| get("deskpet_new"))?;
        let mut m = PetModule {
            memory: instance.get_memory(&store, "memory").ok_or_else(|| get("memory"))?,
            inbox: instance.get_typed_func(&store, "deskpet_inbox").map_err(|_| get("deskpet_inbox"))?,
            about: instance.get_typed_func(&store, "deskpet_about").map_err(|_| get("deskpet_about"))?,
            tick: instance.get_typed_func(&store, "deskpet_tick").map_err(|_| get("deskpet_tick"))?,
            command: instance.get_typed_func(&store, "deskpet_command").map_err(|_| get("deskpet_command"))?,
            render: instance.get_typed_func(&store, "deskpet_render").map_err(|_| get("deskpet_render"))?,
            panic: instance.get_typed_func(&store, "deskpet_panic").ok(),
            store,
        };
        m.refuel()?;
        let started = new.call(&mut m.store, (now, seed));
        started.map_err(|e| m.why(e))?;
        Ok(m)
    }

    fn refuel(&mut self) -> Result<(), String> {
        self.store.set_fuel(FUEL).map_err(err("fuel"))
    }

    /// Says why a call failed: the pet's panic message if it panicked, else what stopped it.
    fn why(&mut self, e: wasmi::Error) -> String {
        let message = self.panic.and_then(|f| {
            self.store.set_fuel(FUEL).ok()?;
            let at = f.call(&mut self.store, ()).ok()?;
            let (start, len) = ((at >> 32) as usize, (at & 0xffff_ffff) as usize);
            let bytes = self.memory.data(&self.store).get(start..start + len.min(2000))?;
            Some(String::from_utf8_lossy(bytes).replace('\n', " "))
        });
        match message {
            Some(m) if !m.is_empty() => format!("it panicked: {m}"),
            _ if e.to_string().contains("fuel") => "it ran too long without finishing (a loop that never ends?)".into(),
            _ => format!("it stopped: {e}"),
        }
    }

    /// Copies a request into the module's inbox; returns its length.
    fn send<T: Wire>(&mut self, v: &T) -> Result<u32, String> {
        let bytes = encode(v);
        self.refuel()?;
        let r = self.inbox.call(&mut self.store, bytes.len() as u32);
        let at = r.map_err(|e| self.why(e))? as usize;
        let mem = self.memory.data_mut(&mut self.store);
        mem.get_mut(at..at + bytes.len()).ok_or("pet gave a bad address")?.copy_from_slice(&bytes);
        Ok(bytes.len() as u32)
    }

    /// Reads a reply the module left at `(address << 32) | length`.
    fn receive<T: Wire>(&self, at: u64) -> Result<T, String> {
        let (start, len) = ((at >> 32) as usize, (at & 0xffff_ffff) as usize);
        let mem = self.memory.data(&self.store);
        let bytes = mem.get(start..start + len).ok_or("pet gave a bad address")?;
        decode(bytes).ok_or_else(|| "pet sent something this app can't read".into())
    }

    pub fn about(&mut self) -> Result<About, String> {
        self.refuel()?;
        let r = self.about.call(&mut self.store, ());
        let at = r.map_err(|e| self.why(e))?;
        self.receive(at)
    }

    pub fn tick(&mut self, input: Input) -> Result<(), String> {
        let len = self.send(&input)?;
        let r = self.tick.call(&mut self.store, len);
        r.map_err(|e| self.why(e))
    }

    pub fn command(&mut self, c: Command) -> Result<bool, String> {
        let len = self.send(&c)?;
        let r = self.command.call(&mut self.store, len);
        Ok(r.map_err(|e| self.why(e))? != 0)
    }

    pub fn render(&mut self) -> Result<Render, String> {
        self.refuel()?;
        let r = self.render.call(&mut self.store, ());
        let at = r.map_err(|e| self.why(e))?;
        self.receive(at)
    }
}
