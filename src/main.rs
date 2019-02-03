extern crate libc;
extern crate nix;

use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::mem;

use nix::sys::statfs;

pub struct MountEntry {
    pub mnt_fsname: String,
    pub mnt_dir: String,
    pub mnt_type: String,
    pub mnt_opts: String,
    pub mnt_freq: i32,
    pub mnt_passno: i32,
    pub statfs: Option<libc::statfs>
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
            statfs: Option::None
        }
    }
}

fn parse_mount_line(line: &str) -> MountEntry {
    let mut mnt_a = line.split_whitespace();
    MountEntry::new(
        mnt_a.next().unwrap().to_string(),
        mnt_a.next().unwrap().to_string(),
        mnt_a.next().unwrap().to_string(),
        mnt_a.next().unwrap().to_string(),
        mnt_a.next().unwrap().parse::<i32>().unwrap(),
        mnt_a.next().unwrap().parse::<i32>().unwrap(),
    )
}

fn get_mounts(f: File) -> Vec<MountEntry> {
    let mut mnts = Vec::new();

    let file = BufReader::new(f);

    for line in file.lines() {
        mnts.push(parse_mount_line(&line.unwrap()));
    }

    mnts
}

fn bar(width: usize, percentage: u8) -> String {
    let filled = (percentage as f32 / 100.0 * (width - 2) as f32).ceil() as usize;
    let fill = String::from_utf8(vec![b'#'; filled]).unwrap();
    let empty = String::from_utf8(vec![b'.'; width - 2 - filled]).unwrap();
    format!("[{}{}]", fill, empty)
}

fn main() {
    let f = File::open("/proc/self/mounts").unwrap();
    let _accept_minimal = vec!["/dev*"];
    let _accept_more = vec!["dev", "run", "tmpfs", "/dev*"];

    let bar_width = 22;

    let mut mnts = get_mounts(f);
    let mut mnts: Vec<&mut MountEntry> = mnts
        .iter_mut()
        .filter(|m| {
            _accept_more.iter().any(|&x| {
                if x.ends_with("*") {
                    m.mnt_fsname.starts_with(&x[..x.len() - 1])
                } else {
                    m.mnt_fsname == x
                }
            })
        })
        .collect();

    for mnt in mnts.iter_mut() {
        let mut stat = unsafe { mem::uninitialized() };
        let stat_opt = match statfs::statfs(&mnt.mnt_dir[..], &mut stat) {
            Ok(_) => Option::Some(stat),
            Err(_) => Option::None
        };
        mnt.statfs = stat_opt;
    }

    let label_fsname = "FILESYSTEM";
    let label_type = "TYPE";
    let label_bar = "";
    let label_used = "%USED";
    let label_available = "AVAILABLE";
    let label_total = "TOTAL";
    let label_mounted = "MOUNTED ON";

    let fsname_width = mnts
        .iter()
        .map(|m| m.mnt_fsname.len())
        .chain(std::iter::once(label_fsname.len()))
        .max()
        .unwrap_or(label_fsname.len());
    let type_width = mnts
        .iter()
        .map(|m| m.mnt_type.len())
        .chain(std::iter::once(label_type.len()))
        .max()
        .unwrap_or(label_type.len());

    println!(
        "{:<fsname_width$} {:<type_width$} {:<bar_width$} {:>6} {:>10} {:>9} {}",
        label_fsname,
        label_type,
        label_bar,
        label_used,
        label_available,
        label_total,
        label_mounted,
        fsname_width = fsname_width,
        type_width = type_width,
        bar_width = bar_width
    );
    for mnt in mnts {
        let (total, available) = match mnt.statfs {
            Some(stat) => (
                stat.f_blocks * (stat.f_frsize as u64),
                stat.f_bavail * (stat.f_frsize as u64),
            ),
            None => (0, 0)
        };

        let used_percentage = 100.0 - available as f32 * 100.0 / total as f32;
        println!(
            "{:<fsname_width$} {:<type_width$} {} {:>5.1}% {:>10} {:>9} {}",
            mnt.mnt_fsname,
            mnt.mnt_type,
            bar(bar_width, used_percentage.ceil() as u8),
            (used_percentage * 10.0).round() / 10.0,
            available / 1024 / 1,
            total / 1024 / 1,
            mnt.mnt_dir,
            fsname_width = fsname_width,
            type_width = type_width,
        );
    }
}
