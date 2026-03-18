use battery::Manager;

const BAR_WIDTH: u64 = 10;

pub struct BatteryWidget {
    charge_pct: f32,
    state: String,
    energy_wh: f32,
    energy_full_wh: f32,
    voltage: f32,
    available: bool,
}

impl BatteryWidget {
    pub fn new() -> Self {
        Self {
            charge_pct: 0.0,
            state: String::new(),
            energy_wh: 0.0,
            energy_full_wh: 0.0,
            voltage: 0.0,
            available: false,
        }
    }

    pub fn update(&mut self) {
        let Ok(manager) = Manager::new() else { return };
        let Ok(batteries) = manager.batteries() else { return };

        for bat in batteries {
            if let Ok(b) = bat {
                self.available = true;
                self.charge_pct = b.state_of_charge().value * 100.0;
                self.state = format!("{:?}", b.state());
                self.energy_wh = b.energy().value;
                self.energy_full_wh = b.energy_full().value;
                self.voltage = b.voltage().value;
                break; // first battery
            }
        }
    }

    pub fn render(&self) -> String {
        if !self.available {
            return " No battery detected\n".to_string();
        }

        let mut out = String::new();

        out.push_str(&format!(" State  {}\n\n", self.state));

        out.push_str(" Charge ");
        out.push_str(&build_bar(self.charge_pct as u64));
        out.push_str(&format!(" {:.1}%\n\n", self.charge_pct));

        out.push_str(&format!(" Energy  {:.1} / {:.1} Wh\n", self.energy_wh, self.energy_full_wh));
        out.push_str(&format!(" Voltage {:.2} V\n", self.voltage));

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
