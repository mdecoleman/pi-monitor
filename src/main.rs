use chrono::{DateTime, Utc};
use core::time;
use dotenv::dotenv;
use influxdb::{Client, InfluxDbWriteable};
use std::{fs, process::Command, str, thread};

const CPU_TEMP_PATH: &str = "/sys/class/thermal/thermal_zone0/temp";
const VCGETEMPCMD: &str = "vcgencmd";

#[derive(InfluxDbWriteable)]
struct Measurment {
    #[influxdb(tag)]
    sensor_id: String,
    cpu_temp: f32,
    gpu_temp: f32,
    time: DateTime<Utc>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let influxdb_bucket = std::env::var("INFLUXDB_BUCKET").expect("INFLUXDB_BUCKET must be set");
    let influxdb_server = std::env::var("INFLUXDB_SERVER").expect("INFLUXDB_SERVER must be set");
    let influxdb_token = std::env::var("INFLUXDB_TOKEN").expect("INFLUXDB_TOKEN must be set");
    let metrics_interval = std::env::var("METRICS_INTERVAL")
        .expect("METRICS_INTERVAL must be set")
        .parse::<u64>()
        .expect("Unable to parse METRICS_INTERVAL");
    let sensor_id = std::env::var("SENSOR_ID").expect("SENSOR_ID must be set");

    let client = Client::new(influxdb_server, influxdb_bucket).with_token(influxdb_token);

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

        let gpu_temp = gpu_temp_parsed[1]
            .parse::<f32>()
            .expect("Could not parse gpu temperature");

        let cpu_temp_raw = fs::read_to_string(CPU_TEMP_PATH).expect("Failed to read CPU temp");

        let cpu_temp = cpu_temp_raw
            .trim_end()
            .parse::<f32>()
            .expect("Could not conver CPU temp to f32")
            / 1000.0;

        println!("gpu_temp: {}, cpu_temp: {}", gpu_temp, cpu_temp);

        let measurment = Measurment {
            sensor_id: sensor_id.clone(),
            cpu_temp,
            gpu_temp,
            time: Utc::now(),
        };

        client
            .query(&measurment.into_query("measurment"))
            .await
            .expect("There was an error wrtting the measurments to influxdb");

        thread::sleep(time::Duration::from_secs(metrics_interval));
    }
}
