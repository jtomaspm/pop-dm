pub struct SessionCommand {
    pub program: String,
    pub args: Vec<String>,
}

pub struct SessionExit {
    pub code: Option<i32>,
}