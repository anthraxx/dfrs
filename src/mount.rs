use crate::errors::*;

use crate::args::NumberFormat;
use crate::theme::Theme;
use crate::util::{format_count, lvm_alias};

use colored::Color;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

#[derive(Clone)]
pub struct Mount {
    pub mnt_fsname: String,
    pub mnt_dir: String,
    pub mnt_type: String,
    pub mnt_opts: String,
    pub mnt_freq: i32,
    pub mnt_passno: i32,
    pub capacity: u64,
    pub free: u64,
    pub used: u64,
    pub statfs: Option<nix::sys::statfs::Statfs>,
}

impl Mount {
    pub fn fsname(&self) -> String {
        self.mnt_fsname.clone()
    }

    pub fn fsname_aliased(&self) -> String {
        let lvm = lvm_alias(&self.mnt_fsname);
        lvm.unwrap_or(self.mnt_fsname.clone())
    }

    pub fn used_percentage(&self) -> Option<f32> {
        match self.capacity {
            0 => None,
            _ => Some(100.0 - self.free as f32 * 100.0 / self.capacity as f32),
        }
    }

    pub fn capacity_formatted(&self, delimiter: &NumberFormat) -> String {
        return format_count(self.capacity as f64, delimiter.get_powers_of());
    }

    pub fn free_formatted(&self, delimiter: &NumberFormat) -> String {
        return format_count(self.free as f64, delimiter.get_powers_of());
    }

    pub fn used_formatted(&self, delimiter: &NumberFormat) -> String {
        return format_count(self.used as f64, delimiter.get_powers_of());
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

    pub fn is_local(&self) -> bool {
        return !self.is_remote();
    }

    pub fn is_remote(&self) -> bool {
        return vec![
            "afs",
            "cifs",
            "coda",
            "ftpfs",
            "fuse.sshfs",
            "mfs",
            "ncpfs",
            "nfs",
            "nfs4",
            "smbfs",
            "sshfs",
        ]
        .iter()
        .any(|&fstype| self.mnt_type.eq(fstype));
    }

    pub fn named(name: String) -> Mount {
        Mount::new(name, "-".to_string(), "-".to_string(), "".to_string(), 0, 0)
    }

    fn new(
        mnt_fsname: String,
        mnt_dir: String,
        mnt_type: String,
        mnt_opts: String,
        mnt_freq: i32,
        mnt_passno: i32,
    ) -> Mount {
        Mount {
            mnt_fsname,
            mnt_dir,
            mnt_type,
            mnt_opts,
            mnt_freq,
            mnt_passno,
            capacity: 0,
            free: 0,
            used: 0,
            statfs: None,
        }
    }
}

fn parse_mount_line(line: &str) -> Result<Mount> {
    let mut mnt_a = line.split_whitespace();
    Ok(Mount::new(
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

pub fn parse_mounts(f: File) -> Result<Vec<Mount>> {
    BufReader::new(f)
        .lines()
        .map(|line| {
            parse_mount_line(&line?)
                .context("Failed to parse mount line")
                .map_err(Error::from)
        })
        .collect::<Result<Vec<_>>>()
}
