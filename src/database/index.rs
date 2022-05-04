use anyhow::Result;
use git2::{FileMode, Oid};

use crate::{Database, ShadowPath};

impl Database {
    fn add_to_index_unchecked(
        &self,
        mode: FileMode,
        tree: Oid,
        path: &str,
        add_trailing_slash: bool,
    ) -> Result<()> {
        let trailing_slash = if add_trailing_slash { "/" } else { "" };
        self.invoke_git(&[
            "update-index".to_string(),
            "--add".to_string(),
            "--cacheinfo".to_string(),
            format!(
                "{:06o},{},{}{}",
                u32::from(mode),
                tree,
                path,
                trailing_slash
            ),
        ])
    }

    pub fn add_to_index(
        &self,
        mode: FileMode,
        tree: Oid,
        relative_path: &ShadowPath,
    ) -> Result<()> {
        let empty_blob_oid = self.empty_blob_oid()?;
        let mut ancestor = ShadowPath::new();
        for component in relative_path.components() {
            self.add_to_index_unchecked(
                FileMode::Blob,
                empty_blob_oid,
                &ancestor.encode_marker(),
                false,
            )?;
            ancestor.push(component.clone());
        }
        self.add_to_index_unchecked(mode, tree, &relative_path.encode(), true)
    }
}
