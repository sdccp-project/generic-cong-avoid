extern crate slog;

use serde_derive;
use reqwest::Error;

use std::fs::File;
use std::io::{Write, BufReader, BufRead};
use std::time::SystemTime;

pub const LOG_OUTPUT_FILE: &str = "log_output.txt";
pub const REST_ADDR: &str = "http://127.0.0.1:8080/get_link_utilization";

use ::{RemoteGenericCongAvoidAlg, NetworkStatus};
use GenericCongAvoidFlow;
use GenericCongAvoidMeasurements;

#[derive(Default)]
pub struct Reno {
    mss: u32,
    init_cwnd: f64,
    cwnd: f64,

    last_utilization: f32,
    log_file: Option<File>,
}

impl RemoteGenericCongAvoidAlg for Reno {
    type Flow = Self;

    fn name() -> &'static str {
        "reno"
    }

    fn with_args(_: clap::ArgMatches) -> Self {
        Default::default()
    }

    fn new_flow(&self, _logger: Option<slog::Logger>, init_cwnd: u32, mss: u32) -> Self::Flow {
        Reno {
            mss,
            init_cwnd: f64::from(init_cwnd),
            cwnd: f64::from(init_cwnd),

            last_utilization: 0.0,
            log_file: File::create(LOG_OUTPUT_FILE).ok(),
        }
    }
}

impl GenericCongAvoidFlow for Reno {
    fn curr_cwnd(&self) -> u32 {
        self.cwnd as u32
    }

    fn set_cwnd(&mut self, cwnd: u32) {
        self.cwnd = f64::from(cwnd);
    }

    fn increase(&mut self, m: &GenericCongAvoidMeasurements) {
        // increase cwnd by 1 / cwnd per packet
        self.cwnd += f64::from(self.mss) * (f64::from(m.acked) / self.cwnd);
    }

    fn reduction(&mut self, _m: &GenericCongAvoidMeasurements) {
        self.cwnd /= 2.0;
        if self.cwnd <= self.init_cwnd {
            self.cwnd = self.init_cwnd;
        }
    }

    fn adjust_cwnd(&mut self,
                   network_status: &NetworkStatus,
                   m: &GenericCongAvoidMeasurements)
    {
        self.cwnd = 37.0 * self.mss as f64;
        let mut queue_packets :i32 = -1;
        if network_status.queue_length > 0 {
            queue_packets = network_status.queue_length / self.mss as i32;
        }
        write!(self.log_file.as_mut().unwrap(),
               "time: {:?}\tlink_utilization: {:.2}\tqueue: {}\tcwnd: {}\trtt: {}\n",
               SystemTime::now(),
               network_status.link_utilization,
               queue_packets,
               self.cwnd as u32 / self.mss,
               m.rtt as f64 / 1000.0);
        return;
        if network_status.queue_length > 10 * self.mss as i32 {
            println!("Link get full utilized. Decrease cwnd");
            self.cwnd -= f64::from(network_status.queue_length) / 10.0;
            if self.cwnd < f64::from(self.mss) {
                self.cwnd = f64::from(self.mss);
            }
        } else {
            let is_aggressive = false;
            if is_aggressive {
                if network_status.link_utilization == self.last_utilization {
                    self.cwnd += f64::from(self.mss) * (f64::from(m.acked) / self.cwnd);
                } else {
                    let mut link_uti = 1.0;
                    if network_status.link_utilization < 1.0 {
                        link_uti = network_status.link_utilization
                    }

                    self.cwnd *= 3.0 / (2.0 * link_uti as f64 + 1.0);
                    self.last_utilization = network_status.link_utilization;
                }
            } else {
                self.cwnd += 2.0 * f64::from(self.mss) * (f64::from(m.acked) / self.cwnd);
            }
        }

        let mut queue_packets :i32 = -1;
        if network_status.queue_length > 0 {
            queue_packets = network_status.queue_length / self.mss as i32;
        }
        write!(self.log_file.as_mut().unwrap(),
               "time: {:?}\tlink_utilization: {:.2}\tqueue: {}\tcwnd: {}\n",
               SystemTime::now(),
               network_status.link_utilization,
               queue_packets,
               self.cwnd as u32 / self.mss);
    }

    fn update_network_status(&mut self) -> NetworkStatus {
        let request_url = REST_ADDR;
        let mut response = reqwest::get(request_url).unwrap();

        response.json().unwrap()
    }
}
