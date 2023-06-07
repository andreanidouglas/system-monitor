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



        thread::sleep(Duration::from_millis(300));
    }

    

}
