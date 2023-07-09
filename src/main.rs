use serde::Serialize;
use std::ops::Add;

fn main() {
    let output = OutputFormat {
        text: get_text(),
        tooltip: get_tooltip(),
    };

    println!("{}", serde_json::to_string(&output).unwrap());
}

fn get_memory_used_percent() -> u32 {
    let out = &std::process::Command::new("nvidia-smi")
        .arg("--format=csv,noheader")
        .arg("--query-gpu=memory.used,memory.total")
        .output()
        .unwrap()
        .stdout;
    let out = String::from_utf8_lossy(out);

    let arr: Vec<f32> = out
        .replace(',', "")
        .replace("MiB", "")
        .split_whitespace()
        .map(|x| x.replace("MiB", "").parse::<f32>().unwrap())
        .collect();

    let mem_used_percent = (arr[0] / arr[1]) * 100.0;

    mem_used_percent.round() as u32
}

fn get_text() -> String {
    let out = &std::process::Command::new("nvidia-smi")
        .arg("--format=csv,noheader,nounits")
        .arg("--query-gpu=utilization.gpu")
        .output()
        .unwrap()
        .stdout;
    let out = String::from_utf8_lossy(out).trim().parse::<u32>().unwrap();

    let output = TextOutput {
        gpu_util: out.to_string(),
        mem_usage: get_memory_used_percent().to_string(),
    };

    format!("{}%|{}%", output.gpu_util, output.mem_usage)
}

fn get_tooltip() -> String {
    let out = &std::process::Command::new("nvidia-smi")
        .arg("--format=csv,noheader")
        .arg("--query-gpu=utilization.gpu,utilization.memory,utilization.encoder,utilization.decoder,temperature.gpu,power.draw,pstate,memory.used,memory.total,fan.speed")
        .output()
        .unwrap()
        .stdout;
    let out = String::from_utf8_lossy(out);

    let out = out.replace(' ', "").replace(',', " ");

    let arr = out.split_whitespace();

    let mut gpu_status = TooltipOutput::default();

    for (i, val) in arr.enumerate() {
        match i {
            0 => gpu_status.gpu_util = val.to_string(),
            1 => gpu_status.mem_util = val.to_string(),
            2 => gpu_status.enc_util = val.to_string(),
            3 => gpu_status.dec_util = val.to_string(),
            4 => gpu_status.temp = val.to_string().add("°C"),
            5 => gpu_status.power = val.to_string(),
            6 => gpu_status.pstate = val.to_string(),
            7 => gpu_status.mem_used = val.to_string().replace("MiB", ""),
            8 => gpu_status.mem_total = val.to_string(),
            9 => gpu_status.fan_speed = val.to_string(),
            _ => (),
        }
    }

    let memory_utilization_percent = get_memory_used_percent();
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
        memory_utilization_percent,
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
struct TooltipOutput {
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

#[derive(Default, Serialize)]
struct OutputFormat {
    text: String,
    tooltip: String,
}

#[derive(Default)]
struct TextOutput {
    gpu_util: String,
    mem_usage: String,
}
