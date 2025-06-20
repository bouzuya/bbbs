mod in_memory_store;
#[cfg(feature = "sqlite")]
mod sqlite_store;

pub use self::in_memory_store::InMemoryStore;
#[cfg(feature = "sqlite")]
pub use self::sqlite_store::SqliteStore;

pub trait Store: crate::port::ThreadReader + crate::port::ThreadRepository {}
