#[derive(Clone)]
pub struct Message {
    pub content: String,
    pub created_at: String,
    pub number: u16,
    pub thread_id: String,
}
