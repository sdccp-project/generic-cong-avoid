extern crate slog;

use serde_derive;
use reqwest::Error;

use ::{RemoteGenericCongAvoidAlg, NetworkStatus};
use GenericCongAvoidFlow;
use GenericCongAvoidMeasurements;

#[derive(Default)]
pub struct Reno {
    mss: u32,
    init_cwnd: f64,
    cwnd: f64,
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

    fn update_network_status(&mut self) -> NetworkStatus {
        let request_url = format!("http://127.0.0.1:8080/get_link_utilization");
        let mut response = reqwest::get(&request_url).unwrap();

        let network_status: Vec<NetworkStatus> = response.json().unwrap();
        println!("{:?}", network_status);
        NetworkStatus {
            link_utilization: 0.0,
            queue_length: 0,
        }
    }
}
