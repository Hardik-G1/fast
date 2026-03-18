use sysinfo::System;

pub trait Widget {
    fn update(&mut self, sys: &mut System);
    fn render(&self) -> String;
}