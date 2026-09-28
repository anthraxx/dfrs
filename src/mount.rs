//! Filesystem mount records and platform-aware discovery.
//!
//! A [`Mount`] is the normalized record used by the rest of the CLI. The
//! provider boundary in this module deliberately separates discovery from
//! rendering: Linux reads its kernel mount listing, macOS uses `getmntinfo(3)`
//! so APFS records are available, and an explicit `--mounts FILE` remains a
//! deterministic parser override for fixtures and troubleshooting.

use crate::args::{DisplayFilter, NumberFormat};
use crate::errors::*;
use crate::theme::Theme;
use crate::util::{format_count, lvm_alias};

use colored::Color;
#[cfg(target_os = "macos")]
use std::ffi::CStr;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// A mounted filesystem and the usage values collected for its mount point.
///
/// Discovery fills in the six `mnt_*` fields. [`Mount::refresh_stats`] then
/// queries the operating system for capacity, free space, and inode values.
/// The renderer only consumes this normalized record, which keeps platform
/// details out of CLI formatting and path matching.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Mount {
    /// Device, pseudo-device, or remote source name reported by the OS.
    pub mnt_fsname: String,
    /// Directory at which the filesystem is mounted.
    pub mnt_dir: String,
    /// Kernel filesystem type, such as `apfs`, `ext4`, or `tmpfs`.
    pub mnt_type: String,
    /// Comma-separated mount options when the source format provides them.
    pub mnt_opts: String,
    /// Traditional `fstab` frequency field; zero for native macOS records.
    pub mnt_freq: i32,
    /// Traditional `fstab` pass number; zero for native macOS records.
    pub mnt_passno: i32,
    /// Total bytes or inodes selected by the CLI mode.
    pub capacity: u64,
    /// Available bytes or inodes selected by the CLI mode.
    pub free: u64,
    /// Used bytes or inodes, derived from capacity minus free.
    pub used: u64,
    /// Raw statfs result used to distinguish discovered capacity from a
    /// record that could not be inspected.
    pub statfs: Option<nix::sys::statfs::Statfs>,
}

impl Mount {
    /// Return the source name exactly as discovered.
    pub fn fsname(&self) -> String {
        self.mnt_fsname.clone()
    }

    /// Return a shorter LVM source name when the source uses `/dev/mapper`.
    pub fn fsname_aliased(&self) -> String {
        let lvm = lvm_alias(&self.mnt_fsname);
        lvm.unwrap_or_else(|| self.mnt_fsname.clone())
    }

    /// Return the percentage of capacity in use, if capacity is known.
    pub fn used_percentage(&self) -> Option<f32> {
        if self.capacity == 0 {
            return None;
        }

        Some((self.used.min(self.capacity) as f32 * 100.0 / self.capacity as f32).min(100.0))
    }

    /// Return the percentage of capacity available, if capacity is known.
    pub fn free_percentage(&self) -> Option<f32> {
        if self.capacity == 0 {
            return None;
        }

        Some(self.free.min(self.capacity) as f32 * 100.0 / self.capacity as f32)
    }

    /// Format total capacity using the requested decimal or binary unit base.
    pub fn capacity_formatted(&self, delimiter: &NumberFormat) -> String {
        format_count(self.capacity as f64, delimiter.get_powers_of())
    }

    /// Format available capacity using the requested unit base.
    pub fn free_formatted(&self, delimiter: &NumberFormat) -> String {
        format_count(self.free as f64, delimiter.get_powers_of())
    }

    /// Format used capacity using the requested unit base.
    pub fn used_formatted(&self, delimiter: &NumberFormat) -> String {
        format_count(self.used as f64, delimiter.get_powers_of())
    }

    /// Select the table color associated with this mount's usage percentage.
    pub fn usage_color(&self, theme: &Theme) -> Color {
        match self.used_percentage() {
            Some(p) if p >= theme.threshold_usage_high => theme.color_usage_high,
            Some(p) if p >= theme.threshold_usage_medium => theme.color_usage_medium,
            Some(_) => theme.color_usage_low,
            _ => theme.color_usage_void,
        }
        .unwrap_or(Color::White)
    }

    /// Return whether this filesystem is not in the known remote-type list.
    #[inline]
    pub fn is_local(&self) -> bool {
        !self.is_remote()
    }

    /// Return whether the filesystem type identifies a remote filesystem.
    pub fn is_remote(&self) -> bool {
        self.mnt_type.starts_with("fuse.")
            || [
                "afs", "cifs", "coda", "ftpfs", "mfs", "ncpfs", "nfs", "nfs4", "smbfs", "sshfs",
            ]
            .contains(&self.mnt_type.as_str())
    }

    /// Return whether the type is an OS/runtime pseudo filesystem.
    ///
    /// Pseudo mounts can still report a nominal capacity, but the default
    /// display hides them so `/proc`, `/sys`, and similar runtime trees do not
    /// drown out actual storage. `--more` and `--all` intentionally broaden
    /// this policy.
    pub fn is_pseudo(&self) -> bool {
        matches!(
            self.mnt_type.as_str(),
            "autofs"
                | "bpf"
                | "cgroup"
                | "cgroup2"
                | "configfs"
                | "debugfs"
                | "devfs"
                | "devpts"
                | "devtmpfs"
                | "efivarfs"
                | "fusectl"
                | "hugetlbfs"
                | "kernfs"
                | "mqueue"
                | "proc"
                | "procfs"
                | "pstore"
                | "ramfs"
                | "securityfs"
                | "sysfs"
                | "tmpfs"
                | "tracefs"
        )
    }

    /// Return whether a statfs query supplied usable capacity.
    pub const fn is_capacity_bearing(&self) -> bool {
        self.statfs.is_some() && self.capacity > 0
    }

    /// Decide whether this record belongs in a display policy.
    ///
    /// Native discovery uses metadata and capacity instead of source-name
    /// conventions. Explicit mount-file overrides retain the historical
    /// source filters, which keeps existing fixtures and scripts compatible.
    pub fn matches_display_filter(&self, filter: &DisplayFilter, native: bool) -> bool {
        match filter {
            DisplayFilter::All => true,
            DisplayFilter::More if native => self.statfs.is_some(),
            DisplayFilter::Minimal if native => self.is_capacity_bearing() && !self.is_pseudo(),
            filter => filter
                .get_mnt_fsname_filter()
                .iter()
                .any(|fsname| crate::util::mnt_matches_filter(self, fsname)),
        }
    }

    /// Populate the usage counters from the mount directory.
    ///
    /// A mount can disappear between discovery and inspection, so a failed
    /// statfs query is represented as an empty record rather than aborting the
    /// entire report. Multiplication and subtraction are saturating to keep a
    /// malformed or changing kernel record from wrapping counters.
    pub fn refresh_stats(&mut self, show_inodes: bool) {
        self.statfs = nix::sys::statfs::statfs(Path::new(&self.mnt_dir)).ok();

        let Some(stat) = self.statfs else {
            self.capacity = 0;
            self.free = 0;
            self.used = 0;
            return;
        };

        let (capacity, free) = if show_inodes {
            (stat.files(), stat.files_free())
        } else {
            let block_size = stat.block_size() as u64;
            (
                stat.blocks().saturating_mul(block_size),
                stat.blocks_available().saturating_mul(block_size),
            )
        };

        self.capacity = capacity;
        self.free = free.min(capacity);
        self.used = capacity.saturating_sub(self.free);
    }

    /// Construct a display-only total row.
    pub fn named(name: String) -> Self {
        Self::new(name, "-".to_string(), "-".to_string(), String::new(), 0, 0)
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

/// Discover mounts from an explicit file or the current operating system.
///
/// Linux's `/proc/self/mounts` is the kernel-backed native listing for the
/// process. macOS uses `getmntinfo(3)` because it exposes the current APFS
/// topology without assuming a Linux path exists. The explicit path is always
/// parsed using the portable mount-file grammar.
pub fn discover_mounts(override_path: Option<&Path>) -> Result<Vec<Mount>> {
    if let Some(path) = override_path {
        let file = File::open(path)
            .with_context(|| format!("failed to open mount-file override {}", path.display()))?;
        return parse_mounts(BufReader::new(file));
    }

    #[cfg(target_os = "linux")]
    {
        let file = File::open("/proc/self/mounts")
            .context("failed to open the Linux native mount listing /proc/self/mounts")?;
        return parse_mounts(BufReader::new(file));
    }

    #[cfg(target_os = "macos")]
    {
        discover_macos_mounts()
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        Err(anyhow!(
            "native mount discovery is currently supported on macOS and Linux"
        ))
    }
}

/// Parse a whitespace-delimited mount listing.
pub fn parse_mounts<R: BufRead>(reader: R) -> Result<Vec<Mount>> {
    let mut mounts = Vec::new();
    for (line_number, line) in reader.lines().enumerate() {
        let line =
            line.with_context(|| format!("failed to read mount line {}", line_number + 1))?;
        if line.trim().is_empty() {
            continue;
        }
        mounts.push(
            parse_mount_line(&line)
                .with_context(|| format!("failed to parse mount line {}", line_number + 1))?,
        );
    }
    Ok(mounts)
}

fn parse_mount_line(line: &str) -> Result<Mount> {
    let fields = line
        .split_whitespace()
        .map(decode_mount_field)
        .collect::<Vec<_>>();
    if fields.len() < 6 {
        return Err(anyhow!("expected six mount fields, found {}", fields.len()));
    }

    Ok(Mount::new(
        fields[0].clone(),
        fields[1].clone(),
        fields[2].clone(),
        fields[3].clone(),
        fields[4].parse::<i32>()?,
        fields[5].parse::<i32>()?,
    ))
}

fn decode_mount_field(field: &str) -> String {
    let bytes = field.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'\\' && index + 3 < bytes.len() {
            let octal = &bytes[index + 1..index + 4];
            if octal.iter().all(|byte| byte.is_ascii_digit())
                && octal.iter().all(|byte| *byte < b'8')
            {
                decoded.push((octal[0] - b'0') * 64 + (octal[1] - b'0') * 8 + octal[2] - b'0');
                index += 4;
                continue;
            }
        }
        decoded.push(bytes[index]);
        index += 1;
    }

    String::from_utf8_lossy(&decoded).into_owned()
}

#[cfg(target_os = "macos")]
fn discover_macos_mounts() -> Result<Vec<Mount>> {
    let mut table = std::ptr::null_mut();
    let count = unsafe { libc::getmntinfo(&mut table, libc::MNT_NOWAIT) };
    if count < 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    if count > 0 && table.is_null() {
        return Err(anyhow!("macOS returned a null mount table"));
    }

    (0..count as usize)
        .map(|index| {
            // `getmntinfo` owns a contiguous table for the duration of this
            // call. We copy every string before returning to avoid retaining
            // pointers into that table.
            let stat = unsafe { &*table.add(index) };
            Ok(Mount::new(
                c_string(&stat.f_mntfromname),
                c_string(&stat.f_mntonname),
                c_string(&stat.f_fstypename),
                String::new(),
                0,
                0,
            ))
        })
        .collect()
}

#[cfg(target_os = "macos")]
fn c_string(value: &[libc::c_char]) -> String {
    // BSD mount names are NUL-terminated fixed-size arrays. Lossy decoding is
    // preferable to dropping a valid mount when a volume name is not UTF-8.
    unsafe { CStr::from_ptr(value.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parse_mounts() {
        let file = concat!(
            "sysfs /sys sysfs rw,nosuid,nodev,noexec,relatime 0 0\n",
            "proc /proc proc rw,nosuid,nodev,noexec,relatime,hidepid=2 0 0\n",
            "udev /dev devtmpfs rw,nosuid,relatime,size=2009144k,nr_inodes=502286,mode=755 0 0\n",
            "devpts /dev/pts devpts rw,nosuid,noexec,relatime,gid=5,mode=620,ptmxmode=000 0 0\n",
            "tmpfs /run tmpfs rw,nosuid,noexec,relatime,size=402800k,mode=755 0 0\n",
            "/dev/mapper/vg0-root / ext4 rw,relatime,errors=remount-ro 0 0\n",
            "tmpfs /run/lock tmpfs rw,nosuid,nodev,noexec,relatime,size=5120k 0 0\n",
            "pstore /sys/fs/pstore pstore rw,relatime 0 0\n",
            "configfs /sys/kernel/config configfs rw,relatime 0 0\n",
            "tmpfs /run/shm tmpfs rw,nosuid,nodev,noexec,relatime,size=805580k 0 0\n",
            "/dev/mapper/vg0-boot /boot ext4 rw,relatime 0 0\n",
            "/dev/mapper/vg0-tmp /tmp ext4 rw,relatime 0 0\n",
            "none /cgroup2 cgroup2 rw,relatime 0 0\n",
        );
        let mounts = super::parse_mounts(Cursor::new(file)).unwrap();
        assert_eq!(mounts.len(), 13);

        let mnt = &mounts[0];
        assert_eq!(mnt.mnt_fsname, "sysfs");
        assert_eq!(mnt.mnt_dir, "/sys");
        assert_eq!(mnt.mnt_type, "sysfs");
        assert_eq!(mnt.mnt_opts, "rw,nosuid,nodev,noexec,relatime");
        assert_eq!(mnt.mnt_freq, 0);
        assert_eq!(mnt.mnt_passno, 0);
        assert_eq!(mnt.capacity, 0);
        assert_eq!(mnt.free, 0);
        assert_eq!(mnt.used, 0);
        assert!(mnt.statfs.is_none());
    }

    #[test]
    fn parse_mounts_decodes_escaped_fields() {
        let mounts = super::parse_mounts(Cursor::new(
            "/dev/disk3s1 /Volumes/My\\040Disk apfs rw 0 0\n",
        ))
        .unwrap();
        assert_eq!(mounts[0].mnt_dir, "/Volumes/My Disk");
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

    #[test]
    fn pseudo_types_are_hidden_from_minimal_native_view() {
        let mut mnt = Mount::named("tmpfs".into());
        mnt.mnt_type = "tmpfs".into();
        mnt.capacity = 100;
        mnt.statfs = None;
        assert!(mnt.is_pseudo());
        assert!(!mnt.matches_display_filter(&DisplayFilter::Minimal, true));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn native_discovery_finds_apfs_or_another_mount() {
        let mounts = discover_mounts(None).unwrap();
        assert!(!mounts.is_empty());
        assert!(mounts.iter().any(|mount| mount.mnt_type == "apfs"));
    }
}
