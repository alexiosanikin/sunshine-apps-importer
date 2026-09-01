//! SteamGridDB API client for fetching game artwork.

use reqwest;
use serde::Deserialize;

/// SteamGridDB API client for fetching game artwork.
pub struct MetadataClient {
    api_key: String,
    base_url: String,
    http_client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
pub struct SteamGridResponse {
    success: bool,
    data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[expect(dead_code)]
pub struct SearchResult {
    id: i64,
    name: String,
    #[serde(rename = "types")]
    types: Vec<String>,
    verified: bool,
}

#[derive(Debug, Deserialize)]
#[expect(dead_code)]
pub struct Grid {
    id: i64,
    style: String,
    url: String,
    thumb: String,
    width: i32,
    height: i32,
}

#[derive(Debug, Deserialize)]
#[expect(dead_code)]
pub struct GridLatestData {
    id: i64,
    url: String,
}

impl MetadataClient {
    /// Create a new SteamGridDB API client.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://www.steamgriddb.com/api/v2".to_string(),
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to build HTTP client"),
        }
    }

    /// Search for games by name.
    pub async fn search_games(&self, query: &str) -> Result<Vec<SearchResult>, anyhow::Error> {
        let endpoint = format!(
            "{}/search/autocomplete/{}",
            self.base_url,
            url_escape(query)
        );

        let resp = self
            .http_client
            .get(&endpoint)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Accept", "application/json")
            .send()
            .await?;

        let status = resp.status();
        if status != 200 {
            let body = resp.text().await?;
            anyhow::bail!("search failed with status {}: {}", status, body);
        }

        let grid_resp: SteamGridResponse = resp.json().await?;
        if !grid_resp.success {
            anyhow::bail!("steamgriddb API returned success=false");
        }

        let results: Vec<SearchResult> = serde_json::from_value(grid_resp.data)?;
        Ok(results)
    }

    /// Fetch grids for a game by its Steam App ID.
    pub async fn get_grids_by_steam_id(
        &self,
        steam_id: i32,
        styles: Option<Vec<String>>,
    ) -> Result<Vec<Grid>, anyhow::Error> {
        let endpoint = match styles {
            Some(ref s) if !s.is_empty() => format!(
                "{}/grids/steam/{}?styles={}",
                self.base_url,
                steam_id,
                s.join(",")
            ),
            _ => format!("{}/grids/steam/{}", self.base_url, steam_id),
        };

        let resp = self
            .http_client
            .get(&endpoint)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Accept", "application/json")
            .send()
            .await?;

        if resp.status() != 200 {
            anyhow::bail!("get grids failed with status {}", resp.status());
        }

        let grid_resp: SteamGridResponse = resp.json().await?;
        if !grid_resp.success {
            anyhow::bail!("steamgriddb API returned success=false");
        }

        let grids: Vec<Grid> = serde_json::from_value(grid_resp.data)?;
        Ok(grids)
    }

    /// Get the URL of the best available grid for a Steam game.
    pub async fn get_best_grid_url(&self, steam_id: i32) -> Result<Option<String>, anyhow::Error> {
        // First try vertical style (1446x638) — Sunshine hero posters
        let grids = self
            .get_grids_by_steam_id(steam_id, Some(vec!["vertical".to_string()]))
            .await;

        let grids = match grids {
            Ok(g) if !g.is_empty() => g,
            _ => self.get_grids_by_steam_id(steam_id, None).await?,
        };

        if grids.is_empty() {
            return Ok(None);
        }

        // Prefer 1446x638 (vertical hero) grids
        let preferred = grids
            .iter()
            .find(|g| g.width >= 1446 && g.height >= 638)
            .or_else(|| grids.first());

        if let Some(grid) = preferred {
            // Try to get the latest URL
            let latest_url = self.get_latest_grid_url(grid.id).await;
            Ok(Some(latest_url.unwrap_or_else(|_| grid.url.clone())))
        } else {
            Ok(None)
        }
    }

    /// Download a grid image and save it to the specified path.
    pub async fn download_grid(
        &self,
        grid_url: &str,
        save_path: &str,
    ) -> Result<(), anyhow::Error> {
        let resp = self
            .http_client
            .get(grid_url)
            .header("Accept", "image/*")
            .send()
            .await?;

        if resp.status() != 200 {
            anyhow::bail!("download failed with status {}", resp.status());
        }

        let dir = std::path::Path::new(save_path)
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        std::fs::create_dir_all(dir)?;

        let data = resp.bytes().await?;
        std::fs::write(save_path, data)?;
        Ok(())
    }

    async fn get_latest_grid_url(&self, grid_id: i64) -> Result<String, anyhow::Error> {
        let endpoint = format!("{}/grid/{}/latest", self.base_url, grid_id);

        let resp = self
            .http_client
            .get(&endpoint)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Accept", "application/json")
            .send()
            .await?;

        if resp.status() != 200 {
            anyhow::bail!("get latest grid failed with status {}", resp.status());
        }

        let latest_resp: SteamGridResponse = resp.json().await?;
        if !latest_resp.success {
            anyhow::bail!("steamgriddb API returned success=false for latest grid");
        }

        let latest_data: GridLatestData = serde_json::from_value(latest_resp.data)?;
        Ok(latest_data.url)
    }
}

/// Re-export for backwards compatibility.
pub type Client = MetadataClient;

fn url_escape(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                result.push(c);
            }
            _ => {
                for b in c.to_string().as_bytes() {
                    result.push_str(&format!("%{:02X}", b));
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_escape() {
        assert_eq!(url_escape("hello"), "hello");
        assert_eq!(url_escape("Counter-Strike 2"), "Counter-Strike%202");
    }
}
