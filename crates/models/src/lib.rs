use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionBuilder {
    pub main_class: MainClass,
    pub java_version: JavaVersion,
    pub arguments: Arguments,
    pub libraries: Vec<Library>,
    pub mods: Vec<Mod>,
    pub natives: Option<Vec<Native>>,
    pub client: Option<Client>,
    pub assets: Vec<Asset>,
    #[serde(skip)]
    pub url_to_path_map: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MainClass {
    pub main_class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaVersion {
    pub major_version: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arguments {
    pub game: Vec<String>,
    pub jvm: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Native {
    pub name: String,
    pub url: String,
    pub path: String,
    pub sha1: String,
    pub size: u64,
    pub os: String,  // "windows", "linux", or "macos"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub name: String,
    pub url: String,
    pub path: String,
    pub sha1: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub hash: String,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mod {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}


impl VersionBuilder {
    /// Builds the url→path lookup map for O(1) file resolution
    pub fn build_url_map(&mut self) {
        let mut map = HashMap::new();

        // Add client
        if let Some(ref client) = self.client {
            map.insert(client.url.clone(), format!("client/{}", client.path));
        }

        // Add libraries
        for lib in &self.libraries {
            if let (Some(ref url), Some(ref path)) = (&lib.url, &lib.path) {
                map.insert(url.clone(), format!("libraries/{}", path));
            }
        }

        // Add mods
        for mod_item in &self.mods {
            if let (Some(ref url), Some(ref path)) = (&mod_item.url, &mod_item.path) {
                map.insert(url.clone(), format!("mods/{}", path));
            }
        }

        // Add natives
        if let Some(ref natives) = self.natives {
            for native in natives {
                map.insert(native.url.clone(), format!("natives/{}", native.path));
            }
        }

        // Add assets
        for asset in &self.assets {
            if let (Some(ref url), Some(ref path)) = (&asset.url, &asset.path) {
                map.insert(url.clone(), format!("assets/{}", path));
            }
        }

        self.url_to_path_map = map;
    }
}
