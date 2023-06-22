use anyhow::Context;
use get_sys_info::{data::readable_byte, saturating_sub_bytes, Duration, Platform, System};
use std::sync::{Arc, Mutex};


#[derive(Copy, Clone)]
pub enum Metric {
    CpuAverage(f32),
    CpuTemp(f32),
    MemTotal(readable_byte),
    MemUsed(readable_byte),
    Uptime(Duration),
    LoadAverage1(f32),
    LoadAverage5(f32),
    LoadAverage15(f32),
}

pub struct SysMonitor {
    sys: Arc<Mutex<System>>,
    metrics: Arc<Mutex<Vec<Metric>>>,
    reading_cpu: Option<std::thread::JoinHandle<()>>,
}

impl SysMonitor {
    pub fn new(sys: System) -> Self {
        Self {
            sys: Arc::new(Mutex::new(sys)),
            metrics: Arc::new(Mutex::new(Vec::new())),
            reading_cpu: None,
        }
    }

    pub fn metrics(&self) -> anyhow::Result<Vec<Metric>> {
        let mut mv = Vec::new();
        let metrics = self.metrics.clone();
        for m in metrics.lock().expect("").iter() {
            mv.push(m.clone());
        }

        Ok(mv)
    }

    pub fn update(&mut self) -> anyhow::Result<()> {
        let sys = self.sys.clone();
        let metrics = self.metrics.clone();

        if let Some(rcpu) = self.reading_cpu.take() {
            match rcpu.is_finished() {
                true => rcpu.join().expect("could not join thread"),
                false => self.reading_cpu = Some(rcpu),
            }
        } else {
            self.reading_cpu = Some(std::thread::spawn(move || {
                let binding = sys.clone();
                let sys = binding.lock().expect("could not lock sys on cpu thread");
                let cpu_load = sys.cpu_load_aggregate().expect("could not read CPU avg");
                drop(sys);

                // by doc you should wait one second before reading the cpu info
                std::thread::sleep(Duration::from_secs(1));

                metrics
                    .lock()
                    .expect("could not lock metrics on cpu thread")
                    .push(Metric::CpuAverage(
                        cpu_load.done().expect("could not get user load").user,
                    ));
            }));
        }

        let binding = self.metrics.clone();
        let mut metrics = binding
            .lock()
            .expect("could not lock metrics on main thread");
        let binding = self.sys.clone();
        let sys = binding.lock().expect("could not lock sys on main thread");

        //let cpu_temp = sys.cpu_temp().context("could not load cpu temp")?;
        //metrics.push(Metric::CpuTemp(cpu_temp));

        let memory = sys.memory().context("could not load memory metrics")?;
        let mem_free = memory.free;
        let mem_total = memory.total;
        let mem_used = saturating_sub_bytes(mem_total, mem_free);

        metrics.push(Metric::MemTotal(mem_total));
        metrics.push(Metric::MemUsed(mem_used));

        let uptime = sys.uptime().context("could not read uptime")?;
        metrics.push(Metric::Uptime(uptime));

        let load_average = sys
            .load_average()
            .context("could not read load_average metrics")?;
        metrics.push(Metric::LoadAverage1(load_average.one));
        metrics.push(Metric::LoadAverage5(load_average.five));
        metrics.push(Metric::LoadAverage15(load_average.fifteen));

        Ok(())
    }
}
