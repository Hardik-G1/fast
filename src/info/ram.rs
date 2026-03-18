
use sysinfo::System;
use crate::info::widget::Widget;
const BAR_WIDTH: u8 = 5;
pub struct RamWidget {
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
}

pub enum MemoryKind {
    Ram,
    Swap,
}
impl RamWidget {
    pub fn new() -> Self {
        Self {
            total_memory: 0,
            used_memory: 0,
            total_swap: 0,
            used_swap: 0,
        }
    }

    fn percentage(&self, kind: MemoryKind) -> u64 {
        match kind {
            MemoryKind::Ram => {
                if self.total_memory == 0 { 0 }
                else { (self.used_memory * 100) / self.total_memory }
            }
            MemoryKind::Swap => {
                if self.total_swap == 0 { 0 }
                else { (self.used_swap * 100) / self.total_swap }
            }
        }
    }

    fn values_mb(&self, kind: MemoryKind) -> (u64, u64) {
        match kind {
            MemoryKind::Ram => (
                self.used_memory / (1024*1024),
                self.total_memory / (1024*1024),
            ),
            MemoryKind::Swap => (
                self.used_swap / (1024*1024),
                self.total_swap / (1024*1024),
            ),
        }
    }
}


impl Widget for RamWidget {

    fn update(&mut self, sys: &mut System) {

        sys.refresh_memory();

        self.total_memory = sys.total_memory();
        self.used_memory = sys.used_memory();

        self.total_swap = sys.total_swap();
        self.used_swap = sys.used_swap();
    }

    fn render(&self) -> String {

        let mut output = String::new();
        let ram_percent  = self.percentage(MemoryKind::Ram);
        let swap_percent = self.percentage(MemoryKind::Swap);

        let (ram_used, ram_total) = self.values_mb(MemoryKind::Ram);
        let (swap_used, swap_total) = self.values_mb(MemoryKind::Swap);
        output.push_str("\n RAM  ");
        output.push_str(&build_bar(ram_percent));
        output.push_str(&format!(" {} MB/{} MB", ram_used, ram_total));
        output.push_str("\n\n SWAP ");
        output.push_str(&build_bar(swap_percent));
        output.push_str(&format!(" {} MB/{} MB", swap_used, swap_total));
        output.push('\n');
        output
 
    }
}

fn build_bar(percent: u64) -> String {
    let filled = percent / (100 / BAR_WIDTH as u64);
    let mut bar = String::new();

    for i in 0..BAR_WIDTH {
        if (i as u64) < filled {
            bar.push_str("██");
        } else {
            bar.push_str("░░");
        }
    }

    bar
}







