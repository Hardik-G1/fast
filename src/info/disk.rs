use sysinfo::{System, Disks};
use crate::info::widget::Widget;

const BLOCKS: u64 = 10;

pub struct DiskEntry {
    pub mount: String,
    pub kind: String,
    pub total: u64,
    pub used: u64,
}

pub struct DiskWidget {
    pub disks: Vec<DiskEntry>,
}

impl DiskWidget {
    pub fn new() -> Self {
        Self { disks: Vec::new() }
    }

    fn percent(used: u64, total: u64) -> u64 {
        if total == 0 { 0 } else { (used * 100) / total }
    }

    fn build_blocks(percent: u64) -> String {
        let filled = percent / (100 / BLOCKS);
        let mut s = String::from(" [");

        for i in 0..BLOCKS {
            if i < filled {
                s.push('■');
            } else {
                s.push('□');
            }
        }

        s.push(']');
        s
    }

    fn to_gb(bytes: u64) -> u64 {
        bytes / 1_000_000_000
    }
}

impl Widget for DiskWidget {

    fn update(&mut self, _sys: &mut System) {

        self.disks.clear();

        let disks = Disks::new_with_refreshed_list();

        for d in disks.list() {

            let total = d.total_space();
            let available = d.available_space();

            self.disks.push(DiskEntry {
                mount: d.mount_point().to_string_lossy().to_string(),
                kind: format!("{:?}", d.kind()),
                total,
                used: total - available,
            });
        }
    }

    fn render(&self) -> String {

        let mut out = String::from("\n");

        for d in &self.disks {

            let percent = Self::percent(d.used, d.total);

            let used_gb = Self::to_gb(d.used);
            let total_gb = Self::to_gb(d.total);

            out.push_str(&format!(
                " {} ({})\n",
                d.mount,
                d.kind
            ));

            out.push_str(&Self::build_blocks(percent));

            out.push_str(&format!(
                "\n {}GB / {}GB  ({}%)\n",
                used_gb,
                total_gb,
                percent
            ));
        }

        out
    }
}