use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use failure::{format_err, ResultExt};
use reqwest::Url;
use serde::Deserialize;
use tempfile::NamedTempFile;
type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Debug, Deserialize)]
pub struct Overlay {
    pub url: String,
    pub path: PathBuf,

    #[serde(skip)]
    hash: u64,

    #[serde(skip)]
    cached: PathBuf,

    #[serde(skip)]
    pub filename: String,
}

struct ProgressWriter<W, F> where
    W: Write,
    F: FnOnce(u64) + Copy
{
    writer: W,
    update: F,
}

impl<W, F> Write for ProgressWriter<W, F> where
    W: Write,
    F: FnOnce(u64) + Copy
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        (self.update)(buf.len() as u64);
        self.writer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

impl Overlay {
    pub fn compute_for_cache(&mut self, cache: &Path) -> Result<()> {
        let mut h = DefaultHasher::new();
        self.url.hash(&mut h);
        self.hash = h.finish();

        let mut cache = cache.to_path_buf();
        cache.push(format!("{:016x}", self.hash));

        std::fs::create_dir_all(&cache)
            .context("creating cache directory")?;

        let filename = Url::parse(&self.url)
            .context("parsing url for overlay")?
            .path_segments()
            .ok_or_else(|| format_err!("couldn't parse url: {}", self.url))?
            .last()
            .ok_or_else(|| format_err!("url has no filename: {}", self.url))?
            .to_string();

        cache.push(&filename);
        self.cached = cache;
        self.filename = filename;
        Ok(())
    }

    pub fn is_cached(&self) -> bool {
        self.cached.is_file()
    }

    pub fn download<F, G>(&self, length: F, update: G) -> Result<()> where
        F: FnOnce(u64), G: FnOnce(u64) + Copy {
        let mut resp = reqwest::get(&self.url)?
            .error_for_status()?;

        length(resp.content_length().unwrap_or(0));

        let mut temp = NamedTempFile::new()?;
        let mut pw: ProgressWriter<_, G> = ProgressWriter{
            writer: &mut temp,
            update: update,
        };

        if let Err(e) = resp.copy_to(&mut pw) {
            temp.close()?;
            Err(e)?
        } else {
            temp.persist(&self.cached)?;
            Ok(())
        }
    }

    pub fn extract(&self, root: &Path) -> Result<()> {
        let mut target = root.to_path_buf();
        target.push(&self.path);

        std::fs::create_dir_all(&target)
            .context("creating target directory")?;

        Command::new("tar")
            .arg("xf")
            .arg(&self.cached)
            .arg("-C")
            .arg(target)
            .spawn()
            .context("Untarring overlay")?
            .wait()
            .context("Waiting for tar child process")?;
        Ok(())
    }
}
