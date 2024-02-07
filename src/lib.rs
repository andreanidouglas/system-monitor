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
    pub fn new(sys: Arc<Mutex<System>>) -> Self {
        Self {
            sys,
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

        loadaverage(&sys.clone(), &metrics.clone())?;

        if let Some(rcpu) = self.reading_cpu.take() {
            match rcpu.is_finished() {
                true => rcpu.join().expect("could not join thread"),
                false => {
                    self.reading_cpu = Some(rcpu);
                    metrics
                        .lock()
                        .expect("could not lock metrics")
                        .push(Metric::CpuAverage(0.0))
                }
            }
        } else {
            self.reading_cpu = Some(std::thread::spawn(move || {
                let binding = sys.clone();
                let sys = binding.lock().expect("could not lock sys on cpu thread");
                let cpu_load = sys.cpu_load_aggregate().expect("could not read CPU avg");
                drop(sys);
                metrics
                    .lock()
                    .expect("could not lock metrics")
                    .push(Metric::CpuAverage(0.0));

                // by doc you should wait one second before reading the cpu info
                std::thread::sleep(Duration::from_secs(1));

                let cpu_user = cpu_load.done().expect("could not get user load").user;

                metrics
                    .lock()
                    .expect("could not lock metrics on cpu thread")
                    .push(Metric::CpuAverage(cpu_user));
            }));
        }

        let binding = self.metrics.clone();
        let mut metrics = binding
            .lock()
            .expect("could not lock metrics on main thread");
        let binding = self.sys.clone();
        let sys = binding.lock().expect("could not lock sys on main thread");

        if let Ok(v) = sys.cpu_temp() {
            metrics.push(Metric::CpuTemp(v))
        };

        let memory = sys.memory().context("could not load memory metrics")?;
        let mem_free = memory.free;
        let mem_total = memory.total;
        let mem_used = saturating_sub_bytes(mem_total, mem_free);

        metrics.push(Metric::MemTotal(mem_total));
        metrics.push(Metric::MemUsed(mem_used));

        let uptime = sys.uptime().context("could not read uptime")?;
        metrics.push(Metric::Uptime(uptime));

        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
fn loadaverage(sys: &Arc<Mutex<System>>, metrics: &Arc<Mutex<Vec<Metric>>>) -> anyhow::Result<()> {
    let sys = sys.lock().expect("could not lock system information");
    let mut metrics = metrics.lock().expect("could not lock metrics");

    let load_average = sys
        .load_average()
        .context("could not read load_average metrics")?;
    metrics.push(Metric::LoadAverage1(load_average.one));
    metrics.push(Metric::LoadAverage5(load_average.five));
    metrics.push(Metric::LoadAverage15(load_average.fifteen));

    Ok(())
}

// windows does not supports load average
#[cfg(target_os = "windows")]
fn loadaverage(sys: &Arc<Mutex<System>>, metrics: &Arc<Mutex<Vec<Metric>>>) -> anyhow::Result<()> {
    Ok(())
}
