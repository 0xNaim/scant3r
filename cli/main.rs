#[macro_use] extern crate log;
extern crate scant3r_utils;
extern crate simplelog;
extern crate scanners;
use simplelog::*;
use std::collections::HashMap;
use futures::{stream, StreamExt};
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use indicatif::{ProgressStyle,ProgressBar};
use scant3r_utils::{
    requests,
};
use scanners::scan;
mod args;

#[tokio::main]
async fn main() {
     CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Info, Config::default(), File::create("my_rust_binary.log").unwrap()),
        ]
    ).unwrap();
    let arg = args::args();
    match arg.subcommand_name() {
        Some("scan") => {
            let sub = arg.subcommand_matches("scan").unwrap();
            let urls = {
                let read_file = File::open(sub.value_of("urls").unwrap()).unwrap();
                BufReader::new(read_file).lines().map(|x| x.unwrap()).collect::<Vec<String>>()
            };

            let bar = ProgressBar::new(urls.len() as u64);
            let mut scan_settings = scan::Scanner::new(vec!["xss"],true,false);
            scan_settings.load_payloads();
            bar.set_style(ProgressStyle::default_bar()
                .template("{msg} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} ({eta})")
                .progress_chars("#>-"));
            stream::iter(&urls)
                .for_each_concurrent(sub.value_of("concurrency").unwrap().parse::<usize>().unwrap(), |url| {
                    let bar = &bar;
                    let scan_settings = &scan_settings;
                    async move {
                        let _msg = requests::Msg::new(
                            &sub.value_of("method").unwrap_or("GET"),
                            &url,
                            HashMap::new(),
                            Some(sub.value_of("data").unwrap_or("").to_string()),
                            Some(1_u32),
                            Some(10_u64),
                            Some("http://localhost:8080".parse().unwrap())
                        );
                        let mut live_check = _msg.clone();
                        live_check.send().await;
                        if live_check.clone().error.unwrap_or(String::from("")) != "" {
                            error!("{}", live_check.clone().error.unwrap());
                        } else {
//                            bar.inc(1);
                            scan_settings.scan(_msg,&bar).await;
                        }
                }
                }).await;
            bar.finish();
        }
        Some("passive") => {
            let sub = arg.subcommand_matches("passive").unwrap();
            for _ in sub.value_of("modules").unwrap().split(",") {
                let _msg = requests::Msg::new(
                    "GET",
                    sub.value_of("url").unwrap(),
                    HashMap::new(),
                    None,
                    Some(sub.value_of("redirect").unwrap_or("0").parse::<u32>().unwrap()),
                    Some(sub.value_of("timeout").unwrap_or("10").parse::<u64>().unwrap()),
                    None,
                );
            }
        },
        _ => println!("No subcommand was used"),
    }
}

