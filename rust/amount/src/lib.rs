pub mod parser;

#[derive(Clone, Default, Debug)]
pub struct Mount {
    pub device: String,
    pub mount_point: String,
    pub filesystem: String,
    pub options: Vec<String>,
}

impl std::fmt::Display for Mount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            //            "{} on {} type {} ({})",
            "{} on {} type {}",
            self.device,
            self.mount_point,
            self.filesystem,
            //            self.options.join(",")
        )
    }
}
