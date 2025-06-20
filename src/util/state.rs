use sled::Db;
use anyhow::{anyhow, Result};
use tracing::{trace, warn};

use crate::{paths::posts::Post, util::build::build};

#[derive(Clone, Debug)]
pub struct SiteState {
    db: Db
}

impl SiteState {
    pub fn new(path: &str, allow_building: bool) -> Result<Self> {
        let db: sled::Db = sled::open(path)?;

        let trees = db.tree_names();
        trace!("Found {} trees [{}]", trees.len(), trees.iter().map(|t| String::from_utf8_lossy(t).to_string()).collect::<Vec<String>>().join(", "));
        if db.tree_names().len() < 2 && allow_building {
            warn!("No database found! Building");
            build(&db)?
        } else if db.tree_names().len() < 2 {
            panic!("No database found! Exiting");
        }

        Ok(SiteState {
            db
        })
    }

    pub fn get_post(&self, slug: &str) -> Result<Post> {
        let posts_tree: sled::Tree = self.db.open_tree(b"posts")?;

        if let Some(value) = posts_tree.get(slug.as_bytes())? {
            let post: Post = bincode::deserialize(&value)?;
            Ok(post)
        } else {
            Err(anyhow!("Post with slug '{}' not found", slug))
        }
    }

    pub fn get_cached_html_element(&self, key: &str) -> String {
        trace!("Getting cached HTML element: {}", key);
        let cached_html_tree = match self.db.open_tree(b"html") {
            Ok(tree) => tree,
            Err(_) => return "???".to_string(),
        };

        let value = match cached_html_tree.get(key) {
            Ok(value) => value,
            Err(_) => return "???".to_string(),
        };

        if let Some(value) = value {
            String::from_utf8_lossy(&value).to_string()
        } else {
            "???".to_string()
        }
    }

    pub fn get_image(&self, path: &str) -> Option<Vec<u8>> {
        trace!("Getting image: {}", path);
        let images_tree = match self.db.open_tree(b"images") {
            Ok(tree) => tree,
            Err(_) => return None,
        };
        
        match images_tree.get(path.as_bytes()) {
            Ok(Some(value)) => Some(value.to_vec()), // TODO: dont do copies
            Ok(None) => None,
            Err(_) => None,
        }
    }

    pub fn map_internal_image(&self, path: &str) -> String {
        trace!("Mapping internal image: {}", path);
        let image_map_tree = match self.db.open_tree(b"image_map") {
            Ok(tree) => tree,
            Err(_) => return "???".to_string(),
        };
        
        match image_map_tree.get(path.as_bytes()) {
            Ok(Some(value)) => {
                match String::from_utf8(value.to_vec()) {
                    Ok(html) => html,
                    Err(_) => "???".to_string(),
                }
            },
            Ok(None) => "???".to_string(),
            Err(_) => "???".to_string(),
        }
    }
}
