use sysinfo::System;
use crate::info::widget::Widget;

pub struct SystemInfoWidget {
    os_name: String,
    os_version: String,
    kernel: String,
    hostname: String,
    cpu_brand: String,
    cores: usize,
    uptime: u64,
    boot_time: u64,
}

impl SystemInfoWidget {

    pub fn new() -> Self {
        Self {
            os_name: String::new(),
            os_version: String::new(),
            kernel: String::new(),
            hostname: String::new(),
            cpu_brand: String::new(),
            cores: 0,
            uptime: 0,
            boot_time: 0,
        }
    }

    fn format_uptime(secs: u64) -> String {

        let hours = secs / 3600;
        let minutes = (secs % 3600) / 60;
        let seconds = secs % 60;

        format!("{:02}h {:02}m {:02}s", hours, minutes, seconds)
    }
}

impl Widget for SystemInfoWidget {

    fn update(&mut self, sys: &mut System) {

        self.os_name = System::name().unwrap_or_default();
        self.os_version = System::os_version().unwrap_or_default();
        self.kernel = System::kernel_version().unwrap_or_default();
        self.hostname = System::host_name().unwrap_or_default();

        self.uptime = System::uptime();
        self.boot_time = System::boot_time();

        if let Some(cpu) = sys.cpus().first() {
            self.cpu_brand = cpu.brand().to_string();
        }

        self.cores = sys.cpus().len();
    }

    fn render(&self) -> String {

        let mut out = String::new();

        out.push_str(&format!(" Host     {}\n", self.hostname));
        out.push_str(&format!(" OS       {} {}\n", self.os_name, self.os_version));
        out.push_str(&format!(" Kernel   {}\n", self.kernel));
        out.push_str(&format!(" CPU      {}\n", self.cpu_brand));
        out.push_str(&format!(" Cores    {}\n", self.cores));
        out.push_str(&format!(" Uptime   {}\n", Self::format_uptime(self.uptime)));
        out.push_str(&format!(" Boot     {}\n", self.boot_time));

        out
    }
}