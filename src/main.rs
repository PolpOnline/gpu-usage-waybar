use serde::Serialize;
use std::ops::Add;

fn main() {
    let gpu_status = NvidiaSmiOutput::new();

    let output = OutputFormat {
        text: get_text(&gpu_status),
        tooltip: get_tooltip(&gpu_status),
    };

    println!("{}", serde_json::to_string(&output).unwrap());
}

fn get_text(gpu_status: &NvidiaSmiOutput) -> String {
    format!(
        "{}|{}%",
        gpu_status.gpu_util.clone(),
        gpu_status.compute_mem_usage()
    )
}

fn get_tooltip(gpu_status: &NvidiaSmiOutput) -> String {
    format!(
        "GPU: {}\n\
        MEM USED: {}/{} ({}%)\n\
        MEM R/W: {}\n\
        ENC: {}\n\
        DEC: {}\n\
        TEMP: {}\n\
        POWER: {}\n\
        PSTATE: {}\n\
        FAN SPEED: {}",
        gpu_status.gpu_util,
        gpu_status.mem_used,
        gpu_status.mem_total,
        gpu_status.compute_mem_usage(),
        gpu_status.mem_util,
        gpu_status.enc_util,
        gpu_status.dec_util,
        gpu_status.temp,
        gpu_status.power,
        gpu_status.pstate,
        gpu_status.fan_speed
    )
}

#[derive(Default)]
struct NvidiaSmiOutput {
    gpu_util: String,
    mem_util: String,
    enc_util: String,
    dec_util: String,
    temp: String,
    power: String,
    pstate: String,
    mem_used: String,
    mem_total: String,
    fan_speed: String,
}

impl NvidiaSmiOutput {
    fn new() -> Self {
        let out = &std::process::Command::new("nvidia-smi")
            .arg("--format=csv,noheader")
            .arg("--query-gpu=utilization.gpu,utilization.memory,utilization.encoder,utilization.decoder,temperature.gpu,power.draw,pstate,memory.used,memory.total,fan.speed")
            .output()
            .unwrap()
            .stdout;
        let out = String::from_utf8_lossy(out);
        let out = out.replace(' ', "").replace(',', " ");

        let split = out.split_whitespace();

        let mut gpu_status = NvidiaSmiOutput::default();

        for (i, val) in split.enumerate() {
            match i {
                0 => gpu_status.gpu_util = val.to_owned(),
                1 => gpu_status.mem_util = val.to_owned(),
                2 => gpu_status.enc_util = val.to_owned(),
                3 => gpu_status.dec_util = val.to_owned(),
                4 => gpu_status.temp = val.to_owned().add("°C"),
                5 => gpu_status.power = val.to_owned(),
                6 => gpu_status.pstate = val.to_owned(),
                7 => gpu_status.mem_used = val.to_owned().replace("MiB", ""),
                8 => gpu_status.mem_total = val.to_owned(),
                9 => gpu_status.fan_speed = val.to_owned(),
                _ => (),
            }
        }

        gpu_status
    }

    fn compute_mem_usage(&self) -> u8 {
        let mem_used_percent = (self.mem_used.parse::<f32>().unwrap()
            / self.mem_total.replace("MiB", "").parse::<f32>().unwrap())
            * 100f32;
        mem_used_percent.round() as u8
    }
}

#[derive(Default, Serialize)]
struct OutputFormat {
    text: String,
    tooltip: String,
}
