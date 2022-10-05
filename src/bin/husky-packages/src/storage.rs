use std::path::PathBuf;

pub struct Storage {
    packages_dir: PathBuf,
}

impl TryFrom<PathBuf> for Storage {
    type Error = std::io::Error;

    fn try_from(packages_dir: PathBuf) -> Result<Self, Self::Error> {
        std::fs::create_dir_all(packages_dir.join("packages"))?;
        std::fs::create_dir_all(packages_dir.join("tmp"))?;
        Ok(Self { packages_dir })
    }
}

impl Storage {
    pub fn write_to_tmp(&self, data: &[u8]) -> std::io::Result<usize> {
        Ok(0)
    }
}
