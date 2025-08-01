#[cfg(feature = "firestore")]
mod firestore_store;
mod in_memory_store;
#[cfg(feature = "sqlite")]
mod sqlite_store;

#[allow(unused_imports)]
#[cfg(feature = "firestore")]
pub use self::firestore_store::FirestoreStore;
#[allow(unused_imports)]
pub use self::in_memory_store::InMemoryStore;
#[allow(unused_imports)]
#[cfg(feature = "sqlite")]
pub use self::sqlite_store::SqliteStore;

pub trait Store: crate::port::ThreadReader + crate::port::ThreadRepository {}
