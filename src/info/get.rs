use sysinfo::{
    System
};
use ratatui::layout::{Constraint, Direction, Layout};
 use crate::info::disk::DiskWidget;
 use crate::info::network::NetworkWidget;
use std::{
  io::{stdout, Result},
  time::Duration,
};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    widgets::{Paragraph, Block, Borders},
};
use ratatui::crossterm::{
  terminal::{EnterAlternateScreen, LeaveAlternateScreen,
            enable_raw_mode,
        disable_raw_mode,},
  ExecutableCommand,
};
use crate::info::system_info::SystemInfoWidget;
use crate::info::gpu::GpuWidget;
use crate::info::battery_info::BatteryWidget;
use crate::info::widget::Widget;

use std::{
    time::{Instant},
};

use crate::info::cpu::get_cpu_info;
use crate::info::cpu::draw_cpu;
use crate::info::ram::RamWidget;
fn initialize()->System{
      return System::new_all();
}


pub fn ftop()->Result<()>{
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    let mut sys = initialize();
    let mut cpu_data = Vec::new();
    let mut ram = RamWidget::new();
    let mut disk = DiskWidget::new();
    let mut network=NetworkWidget::new();
    let mut basic_info=SystemInfoWidget::new();
    let mut gpu = GpuWidget::new();
    let mut battery = BatteryWidget::new();


    let tick_rate = Duration::from_millis(50);
    let slow_rate = Duration::from_secs(2);
    let mut last_tick = Instant::now();
    let mut last_slow = Instant::now();

    // initial slow updates
    disk.update(&mut sys);
    basic_info.update(&mut sys);
    gpu.update();
    battery.update();

    loop {
        // fast: every tick
        get_cpu_info(&mut sys, &mut cpu_data);
        ram.update(&mut sys);
        network.update(&mut sys);

        // slow: every 2s
        if last_slow.elapsed() >= slow_rate {
            disk.update(&mut sys);
            basic_info.update(&mut sys);
            gpu.update();
            battery.update();
            last_slow = Instant::now();
        }

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {

                match key.code {

                    KeyCode::Char('q') => break,

                    KeyCode::Char('c')
                        if key.modifiers.contains(KeyModifiers::CONTROL) => break,

                    KeyCode::Esc => break,

                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
        terminal.draw(|frame| {

            let area = frame.area();

            // BIG BOX (whole terminal)
            let outer = Block::default()
                .title("Monitor")
                .borders(Borders::ALL);

            frame.render_widget(outer.clone(), area);

            let inner = outer.inner(area);

            // layout inside big box
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(16), // CPU widget height
                    Constraint::Min(0),     // Network widget height
                ])
                .split(inner);

            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(34),
                ])
                .split(rows[0]);
            let basic_widget = Paragraph::new(basic_info.render())
                .block(Block::default().title("System information").borders(Borders::ALL));

            frame.render_widget(basic_widget, cols[0]);
            let cpu_text = draw_cpu(&cpu_data);

            let cpu_widget = Paragraph::new(cpu_text)
                .block(Block::default().title("Cores").borders(Borders::ALL));

            frame.render_widget(cpu_widget, cols[1]);
            
            let ram_widget = Paragraph::new(ram.render())
                .block(Block::default().title("Memory").borders(Borders::ALL));

            frame.render_widget(ram_widget, cols[2]);


            let bottom_cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                ])
                .split(rows[1]);
            let disk_widget = Paragraph::new(disk.render())
                .block(Block::default().title("Disk").borders(Borders::ALL));
            frame.render_widget(disk_widget, bottom_cols[0]);

            let network_widget = Paragraph::new(network.render())
                .block(Block::default().title("Network").borders(Borders::ALL));
            frame.render_widget(network_widget, bottom_cols[1]);

            let gpu_widget = Paragraph::new(gpu.render())
                .block(Block::default().title("GPU").borders(Borders::ALL));
            frame.render_widget(gpu_widget, bottom_cols[2]);

            let battery_widget = Paragraph::new(battery.render())
                .block(Block::default().title("Battery").borders(Borders::ALL));
            frame.render_widget(battery_widget, bottom_cols[3]);

        })?;
        last_tick = Instant::now();
    }
    }
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
// use sysinfo::System;

// let mut sys = System::new();

// loop {
//     sys.refresh_cpu_usage(); // Refreshing CPU usage.
//     for cpu in sys.cpus() {
//         print!("{}% ", cpu.cpu_usage());
//     }
//     // Sleeping to let time for the system to run for long
//     // enough to have useful information.
//     std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
// }


// Most of the time, you don't want all information provided by sysinfo but just a subset of it. In this case, it's recommended to use refresh_specifics(...) methods with only what you need to have much better performance.


// pub fn _get_all_info(){
//     let mut sys = System::new_all();
//     sys.refresh_all();

//     // We display all disks' information:
//     println!("=> disks:");


//     // Network interfaces name, total data received and total data transmitted:
//     let networks = Networks::new_with_refreshed_list();
//     println!("=> networks:");
//     for (interface_name, data) in &networks {
//         println!(
//             "{interface_name}: {} B (down) / {} B (up)",
//             data.total_received(),
//             data.total_transmitted(),
//         );
//         // If you want the amount of data received/transmitted since last call
//         // to `Networks::refresh`, use `received`/`transmitted`.
//     }

//     // Components temperature:

// }