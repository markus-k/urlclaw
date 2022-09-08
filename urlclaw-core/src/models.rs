use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShortUrl {
    pub id: Uuid,

    pub short: String,
    pub target: String,
}

impl ShortUrl {
    pub fn new(short: String, target: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            short,
            target,
        }
    }
}
