const BAR_WIDTH: u64 = 10;

pub struct GpuWidget {
    vendor: String,
    model: String,
    vram_total_mb: u64,
    vram_used_mb: u64,
    load_pct: u32,
    temp_c: u32,
}

impl GpuWidget {
    pub fn new() -> Self {
        Self {
            vendor: String::new(),
            model: String::new(),
            vram_total_mb: 0,
            vram_used_mb: 0,
            load_pct: 0,
            temp_c: 0,
        }
    }

    pub fn update(&mut self) {
        if let Ok(gpu) = gfxinfo::active_gpu() {
            self.vendor = gpu.vendor().to_string();
            self.model = gpu.model().to_string();
            let info = gpu.info();
            self.vram_total_mb = info.total_vram() / (1024 * 1024);
            self.vram_used_mb = info.used_vram() / (1024 * 1024);
            self.load_pct = info.load_pct();
            self.temp_c = info.temperature() / 1000;
        }
    }

    pub fn render(&self) -> String {
        let mut out = String::new();

        out.push_str(&format!(" {} {}\n", self.vendor, self.model));
        out.push_str(&format!(" Temp  {}°C\n\n", self.temp_c));

        // Load bar
        out.push_str(" Load ");
        out.push_str(&build_bar(self.load_pct as u64));
        out.push_str(&format!(" {}%\n\n", self.load_pct));

        // VRAM bar
        let vram_pct = if self.vram_total_mb > 0 {
            (self.vram_used_mb * 100) / self.vram_total_mb
        } else {
            0
        };
        out.push_str(" VRAM ");
        out.push_str(&build_bar(vram_pct));
        out.push_str(&format!(" {} / {} MB\n", self.vram_used_mb, self.vram_total_mb));

        out
    }
}

fn build_bar(percent: u64) -> String {
    let filled = percent / (100 / BAR_WIDTH);
    let mut bar = String::new();
    for i in 0..BAR_WIDTH {
        if i < filled {
            bar.push_str("██");
        } else {
            bar.push_str("░░");
        }
    }
    bar
}
