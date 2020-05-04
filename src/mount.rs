use crate::errors::*;

use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

pub struct MountEntry {
    pub mnt_fsname: String,
    pub mnt_dir: String,
    pub mnt_type: String,
    pub mnt_opts: String,
    pub mnt_freq: i32,
    pub mnt_passno: i32,
    pub capacity: String,
    pub free: String,
    pub used_percentage: f32,
    pub used: String,
    pub statfs: Option<nix::sys::statfs::Statfs>,
}

impl MountEntry {
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
            capacity: "".to_string(),
            free: "".to_string(),
            used: "".to_string(),
            used_percentage: 0.0,
            statfs: Option::None,
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
