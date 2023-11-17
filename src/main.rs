use core::time;
use std::{fs, process::Command, str, thread};

const VCGETEMPCMD: &str = "vcgencmd";
const CPU_TEMP_PATH: &str = "/sys/class/thermal/thermal_zone0/temp";

fn main() {
    loop {
        let gpu_temp_output = Command::new(VCGETEMPCMD)
            .arg("measure_temp")
            .output()
            .expect("Failed to execute vcgencmd command");

        let gpu_temp_parsed: Vec<&str> = str::from_utf8(&gpu_temp_output.stdout)
            .ok()
            .expect("Failed to convert from byte string")
            .split(['=', '\''].as_ref())
            .collect();

        let gpu_temp = gpu_temp_parsed[1];

        let cpu_temp_raw = fs::read_to_string(CPU_TEMP_PATH).expect("Failed to read CPU temp");

        let cpu_temp = cpu_temp_raw
            .trim_end()
            .parse::<f32>()
            .expect("Could not conver CPU temp to f32")
            / 1000.0;

        println!("CPU temperature: {cpu_temp}°C, GPU temperature: {gpu_temp}°C",);

        thread::sleep(time::Duration::from_secs(10));
    }
}
