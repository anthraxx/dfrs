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
        lvm.unwrap_or_else(|| self.mnt_fsname.clone())
    }

    pub fn used_percentage(&self) -> Option<f32> {
        match self.capacity {
            0 => None,
            _ => Some(100.0 - self.free as f32 * 100.0 / self.capacity as f32),
        }
    }

    pub fn free_percentage(&self) -> Option<f32> {
        match self.free {
            0 => None,
            _ => Some(self.free as f32 * 100.0 / self.capacity as f32),
        }
    }

    pub fn capacity_formatted(&self, delimiter: &NumberFormat) -> String {
        format_count(self.capacity as f64, delimiter.get_powers_of())
    }

    pub fn free_formatted(&self, delimiter: &NumberFormat) -> String {
        format_count(self.free as f64, delimiter.get_powers_of())
    }

    pub fn used_formatted(&self, delimiter: &NumberFormat) -> String {
        format_count(self.used as f64, delimiter.get_powers_of())
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

    #[inline]
    pub fn is_local(&self) -> bool {
        !self.is_remote()
    }

    pub fn is_remote(&self) -> bool {
        [
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
        .contains(&self.mnt_type.as_str())
    }

    pub fn named(name: String) -> Self {
        Self::new(name, "-".to_string(), "-".to_string(), "".to_string(), 0, 0)
    }

    const fn new(
        mnt_fsname: String,
        mnt_dir: String,
        mnt_type: String,
        mnt_opts: String,
        mnt_freq: i32,
        mnt_passno: i32,
    ) -> Self {
        Self {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_mounts() {
        let file = r#"sysfs /sys sysfs rw,nosuid,nodev,noexec,relatime 0 0
proc /proc proc rw,nosuid,nodev,noexec,relatime,hidepid=2 0 0
udev /dev devtmpfs rw,nosuid,relatime,size=2009144k,nr_inodes=502286,mode=755 0 0
devpts /dev/pts devpts rw,nosuid,noexec,relatime,gid=5,mode=620,ptmxmode=000 0 0
tmpfs /run tmpfs rw,nosuid,noexec,relatime,size=402800k,mode=755 0 0
/dev/mapper/vg0-root / ext4 rw,relatime,errors=remount-ro 0 0
tmpfs /run/lock tmpfs rw,nosuid,nodev,noexec,relatime,size=5120k 0 0
pstore /sys/fs/pstore pstore rw,relatime 0 0
configfs /sys/kernel/config configfs rw,relatime 0 0
tmpfs /run/shm tmpfs rw,nosuid,nodev,noexec,relatime,size=805580k 0 0
/dev/mapper/vg0-boot /boot ext4 rw,relatime 0 0
/dev/mapper/vg0-tmp /tmp ext4 rw,relatime 0 0
none /cgroup2 cgroup2 rw,relatime 0 0
"#;
        let mounts = file
            .lines()
            .map(|line| {
                parse_mount_line(line)
                    .context("Failed to parse mount line")
                    .map_err(Error::from)
            })
            .collect::<Result<Vec<_>>>()
            .unwrap();
        assert_eq!(mounts.len(), 13);

        // nix::sys::statfs::Statfs doesn't have PartialEq
        let mnt = &mounts[0];
        assert_eq!(mnt.mnt_fsname.as_str(), "sysfs");
        assert_eq!(mnt.mnt_dir.as_str(), "/sys");
        assert_eq!(mnt.mnt_type.as_str(), "sysfs");
        assert_eq!(mnt.mnt_opts.as_str(), "rw,nosuid,nodev,noexec,relatime");
        assert_eq!(mnt.mnt_freq, 0);
        assert_eq!(mnt.mnt_passno, 0);
        assert_eq!(mnt.capacity, 0);
        assert_eq!(mnt.free, 0);
        assert_eq!(mnt.used, 0);
        assert!(mnt.statfs.is_none());
    }

    #[test]
    fn is_remote() {
        let mut mnt = Mount::named("foo".into());
        mnt.mnt_type = String::from("nfs");
        assert!(mnt.is_remote());
    }

    #[test]
    fn is_local() {
        let mut mnt = Mount::named("foo".into());
        mnt.mnt_type = String::from("btrfs");
        assert!(mnt.is_local());
    }
}
