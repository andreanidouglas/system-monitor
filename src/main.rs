use std::{thread, sync::{Arc, Mutex}};

use crosscurses::{endwin, initscr, Window};
use get_sys_info::{Duration, Platform, System};
use memory_monitor::SysMonitor;

fn printw(window: &Window, y: i32, x: i32, value: &str) {
    window.mvprintw(y, x, value);
}

fn main() -> anyhow::Result<()> {
    let window = initscr();
    window.nodelay(true);

    let system = Arc::new(Mutex::new(System::new()));

    let mut sysmonitor = SysMonitor::new(system);
    'outer: loop {
        window.refresh();

        if let Err(e) = sysmonitor.update() {
            printw(&window, 10, 0, &format!("Could not update metrics: {}", e));
        }

        let metrics = sysmonitor.metrics();

        for m in metrics.unwrap() {
            match m {
                memory_monitor::Metric::CpuAverage(a) => {
                    printw(&window, 0, 0, &format!("CPU Average: {}", a))
                }
                memory_monitor::Metric::MemTotal(mt) => {
                    printw(&window, 1, 0, &format!("Total Memory: {}", mt))
                }
                memory_monitor::Metric::MemUsed(mu) => {
                    printw(&window, 2, 0, &format!("Usage Memory: {}", mu))
                }
                memory_monitor::Metric::Uptime(u) => {
                    printw(&window, 3, 0, &format!("Uptime: {}", u.as_secs()))
                }
                memory_monitor::Metric::LoadAverage1(a1) => {
                    printw(&window, 4, 0, &format!("Load 1: {}", a1))
                }
                memory_monitor::Metric::LoadAverage5(a5) => {
                    printw(&window, 5, 0, &format!("Load 5: {}", a5))
                }
                memory_monitor::Metric::LoadAverage15(a15) => {
                    printw(&window, 6, 0, &format!("Load 15: {}", a15))
                }
                memory_monitor::Metric::CpuTemp(v) => {
                    printw(&window, 7, 0, &format!("CPU Temp: {} C", v))
                }
            }
        }
        match window.getch() {
            Some(crosscurses::Input::Character(x)) if x == 'q' => break 'outer,
            None => (),
            _ => (),
        }

        thread::sleep(Duration::from_millis(33));
    }

    endwin();
    Ok(())
}
