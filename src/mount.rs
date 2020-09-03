use crate::errors::*;

use crate::theme::Theme;
use colored::Color;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

#[derive(Clone)]
pub struct MountEntry {
    pub mnt_fsname: String,
    pub mnt_dir: String,
    pub mnt_type: String,
    pub mnt_opts: String,
    pub mnt_freq: i32,
    pub mnt_passno: i32,
    pub capacity: u64,
    pub free: u64,
    pub used: u64,
    pub capacity_formatted: String,
    pub free_formatted: String,
    pub used_formatted: String,
    pub statfs: Option<nix::sys::statfs::Statfs>,
}

impl MountEntry {
    pub fn used_percentage(&self) -> Option<f32> {
        match &self.capacity {
            0 => None,
            _ => Some(100.0 - *&self.free as f32 * 100.0 / *&self.capacity as f32),
        }
    }

    pub fn usage_color(&self, theme: &Theme) -> Color {
        match &self.used_percentage() {
            Some(p) if p >= &theme.threshold_usage_high => &theme.color_usage_high,
            Some(p) if p >= &theme.threshold_usage_medium => &theme.color_usage_medium,
            Some(_) => &theme.color_usage_low,
            _ => &theme.color_usage_void,
        }
        .unwrap_or(Color::White)
    }

    pub fn named(name: String) -> MountEntry {
        MountEntry::new(name, "-".to_string(), "-".to_string(), "".to_string(), 0, 0)
    }

    fn new(
        mnt_fsname: String,
        mnt_dir: String,
        mnt_type: String,
        mnt_opts: String,
        mnt_freq: i32,
        mnt_passno: i32,
    ) -> MountEntry {
        MountEntry {
            mnt_fsname,
            mnt_dir,
            mnt_type,
            mnt_opts,
            mnt_freq,
            mnt_passno,
            capacity: 0,
            free: 0,
            used: 0,
            capacity_formatted: "".to_string(),
            free_formatted: "".to_string(),
            used_formatted: "".to_string(),
            statfs: None,
        }
    }
}

fn parse_mount_line(line: &str) -> Result<MountEntry> {
    let mut mnt_a = line.split_whitespace();
    Ok(MountEntry::new(
        mnt_a
            .next()
            .ok_or_else(|| anyhow!("Missing value fsname"))?
            .into(),
        mnt_a
            .next()
            .ok_or_else(|| anyhow!("Missing value dir"))?
            .into(),
        mnt_a
            .next()
            .ok_or_else(|| anyhow!("Missing value type"))?
            .into(),
        mnt_a
            .next()
            .ok_or_else(|| anyhow!("Missing value opts"))?
            .into(),
        mnt_a
            .next()
            .ok_or_else(|| anyhow!("Missing value freq"))?
            .parse::<i32>()?,
        mnt_a
            .next()
            .ok_or_else(|| anyhow!("Missing value passno"))?
            .parse::<i32>()?,
    ))
}

pub fn get_mounts(f: File) -> Result<Vec<MountEntry>> {
    BufReader::new(f)
        .lines()
        .map(|line| {
            parse_mount_line(&line?)
                .context("Failed to parse mount line")
                .map_err(Error::from)
        })
        .collect::<Result<Vec<_>>>()
}
