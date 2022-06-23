#[derive(Debug, Clone)]
pub enum RequestKind {
    Query,
    Mutation,
}

#[derive(Debug, Clone)]
pub struct Request {
    pub name: String,
    pub kind: RequestKind,
}
