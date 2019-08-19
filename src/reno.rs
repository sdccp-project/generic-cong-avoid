extern crate slog;

use serde_derive;
use reqwest::Error;

use std::fs::File;
use std::io::{Write, BufReader, BufRead};
use std::time::{SystemTime, Instant};

pub const LOG_OUTPUT_FILE: &str = "log_output";
pub const TXT: &str = ".txt";
pub const REST_ADDR: &str = "http://127.0.0.1:8080/get_user_link_utilization";

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
    sock_id: u32,
    server_url: String,
}

impl RemoteGenericCongAvoidAlg for Reno {
    type Flow = Self;

    fn name() -> &'static str {
        "reno"
    }

    fn with_args(_: clap::ArgMatches) -> Self {
        Default::default()
    }

    fn new_flow(&self, _logger: Option<slog::Logger>, init_cwnd: u32, mss: u32,
                sock_id: u32) -> Self::Flow {
        let mut log_file = LOG_OUTPUT_FILE.to_string();
        log_file.push_str(&sock_id.to_string());
        log_file.push_str(TXT);
        Reno {
            mss,
            init_cwnd: f64::from(init_cwnd),
            cwnd: f64::from(init_cwnd),

            last_utilization: 0.0,
            log_file: File::create(&log_file).ok(),
            sock_id,
            server_url: format!("{}/{}", REST_ADDR, sock_id),
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
        let queue_length = network_status.queue_length;
        let mut network_utilization = network_status.link_utilization;
        if network_utilization < 0.0 {
            network_utilization = 0.0;
        }

        {
            // The following codes are used to test how high can cwnd be
            let fix_cwnd :Option<u32> = None;
            if fix_cwnd.is_some() {
                self.cwnd = fix_cwnd.unwrap() as f64 * self.mss as f64;
                let mut queue_packets :i32 = -1;
                if queue_length > 0 {
                    queue_packets = queue_length / self.mss as i32;
                }
                write!(self.log_file.as_mut().unwrap(),
                       "time: {:?}\tlink_utilization: {:.2}\tqueue: {}\tcwnd: {}\trtt: {}\n",
                       SystemTime::now(),
                       network_status.link_utilization,
                       queue_packets,
                       self.cwnd as u32 / self.mss,
                       m.rtt as f64 / 1000.0);
                return;
            }
        }

        if network_utilization == self.last_utilization {
            self.cwnd += f64::from(self.mss) * (f64::from(m.acked) / self.cwnd) * 1.0;

            // log
            let mut queue_packets :i32 = 0;
            if network_status.queue_length >= 0 {
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
        }

        let is_aggressive = network_utilization < 0.8;
        if is_aggressive {
            if network_status.link_utilization == self.last_utilization {
            } else {
                self.cwnd *= 3.0 / (2.0 * network_utilization as f64 + 1.0);
            }
        } else if network_utilization < 1.0 {
            self.cwnd += f64::from(self.mss) * (f64::from(m.acked) / self.cwnd);
        }

        if network_utilization > 1.0 {
            self.cwnd /= network_utilization as f64;
        } else if (queue_length > 0) & (network_utilization > 0.9) {
            {
                println!("Link get full utilized. Decrease cwnd");
                self.cwnd -= f64::from(queue_length) / 3.0;
                if self.cwnd <= 0.0 {
                    self.cwnd = self.mss as f64;
                }
            }
        }
        self.last_utilization = network_utilization;

        // log
        let mut queue_packets :i32 = 0;
        if network_status.queue_length >= 0 {
            queue_packets = network_status.queue_length / self.mss as i32;
        }
        write!(self.log_file.as_mut().unwrap(),
               "time: {:?}\tlink_utilization: {:.2}\tqueue: {}\tcwnd: {}\trtt: {}\n",
               SystemTime::now(),
               network_status.link_utilization,
               queue_packets,
               self.cwnd as u32 / self.mss,
               m.rtt as f64 / 1000.0);
    }

    fn update_network_status(&mut self) -> NetworkStatus {
        let request_url = &self.server_url;
        let mut response = reqwest::get(request_url)
            .expect(&format!("Failed to get response from url ({:?})", request_url));

        response.json()
            .expect(&format!("Failed to parse the response {:?}", response))
    }
}
