use sysinfo::{System, Networks};
use crate::info::widget::Widget;
use std::collections::VecDeque;

const HISTORY: usize = 10;

pub struct NetworkWidget {
    networks: Networks,
    last_rx: u64,
    last_tx: u64,
    rx_hist: VecDeque<f64>,
    tx_hist: VecDeque<f64>,
}

impl NetworkWidget {

    pub fn new() -> Self {
        Self {
            networks: Networks::new_with_refreshed_list(),
            last_rx: 0,
            last_tx: 0,
            rx_hist: VecDeque::with_capacity(HISTORY),
            tx_hist: VecDeque::with_capacity(HISTORY),
        }
    }

    fn to_mbps(bytes: u64) -> f64 {
        (bytes as f64 * 8.0) / 1_000_000.0
    }

    fn push(hist: &mut VecDeque<f64>, val: f64) {
        if hist.len() == HISTORY {
            hist.pop_front();
        }
        hist.push_back(val);
    }

    fn braille_graph(hist: &VecDeque<f64>) -> String {

        let mut graph = String::new();

        let max = hist.iter().cloned().fold(0.0, f64::max).max(1.0);

        for v in hist {

            let level = ((*v / max) * 8.0) as u8;

            let ch = match level {
                0 => '⣀',
                1 => '⣄',
                2 => '⣆',
                3 => '⣇',
                4 => '⣧',
                5 => '⣷',
                6 => '⣾',
                _ => '⣿',
            };

            graph.push(ch);
        }

        graph
    }
}

impl Widget for NetworkWidget {

    fn update(&mut self, _sys: &mut System) {

        self.networks.refresh(true);

        let mut total_rx = 0;
        let mut total_tx = 0;

        for (_, data) in &self.networks {
            total_rx += data.received();
            total_tx += data.transmitted();
        }

        let rx = total_rx.saturating_sub(self.last_rx);
        let tx = total_tx.saturating_sub(self.last_tx);

        self.last_rx = total_rx;
        self.last_tx = total_tx;

        let rx_mbps = Self::to_mbps(rx);
        let tx_mbps = Self::to_mbps(tx);

        Self::push(&mut self.rx_hist, rx_mbps);
        Self::push(&mut self.tx_hist, tx_mbps);
    }

    fn render(&self) -> String {

        let rx_now = *self.rx_hist.back().unwrap_or(&0.0);
        let tx_now = *self.tx_hist.back().unwrap_or(&0.0);

        format!(
            "↓ {} {:>6.2} Mbps\n↑ {} {:>6.2} Mbps\n",
            Self::braille_graph(&self.rx_hist),
            rx_now,
            Self::braille_graph(&self.tx_hist),
            tx_now
        )
    }
}