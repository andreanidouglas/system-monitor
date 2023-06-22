use std::thread;

//use crosscurses::{endwin, initscr};
use get_sys_info::{System, Platform, Duration};
use memory_monitor::SysMonitor;

fn main() -> anyhow::Result<()>{
    /*let window = initscr();

    window.mvprintw(0, 0, format!("Hello, World"));
    window.refresh();
    window.getch();

    endwin();*/

    let system = System::new();
    let mut sysmonitor = SysMonitor::new(system);

    loop {
        sysmonitor.update()?;

        let metrics = sysmonitor.metrics();

        for m in metrics.unwrap() {
            match m {
                memory_monitor::Metric::CpuAverage(a) => println!("CPU Average: {}", a), 
                memory_monitor::Metric::CpuTemp(_) => anyhow::bail!("cpu temp not available"),
                memory_monitor::Metric::MemTotal(mt) => println!("Total Memory: {}", mt),
                memory_monitor::Metric::MemUsed(mu) => println!("Usage Memory: {}", mu),
                memory_monitor::Metric::Uptime(u) => println!("Uptime: {}", u.as_secs()),
                memory_monitor::Metric::LoadAverage1(a1) => println!("Load 1: {}", a1),
                memory_monitor::Metric::LoadAverage5(a5) => println!("Load 5: {}", a5),
                memory_monitor::Metric::LoadAverage15(a15) => println!("Load 15: {}", a15),
            }
        }






        thread::sleep(Duration::from_millis(300));
    }

    

}
