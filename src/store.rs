mod in_memory_store;

pub use self::in_memory_store::InMemoryStore;

pub trait Store: crate::port::ThreadReader + crate::port::ThreadRepository {}
