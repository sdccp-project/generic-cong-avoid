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

    last_utilization: f32,
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
        if network_status.queue_length > 10 {
            println!("Link get full utilized. Decrease cwnd");
            self.cwnd -= network_status.queue_length as f64 / 10.0;
            return
        }

        if network_status.link_utilization == self.last_utilization {
            println!("Link underutilized. Increase cwnd by 1 / cwnd per packet");
            self.cwnd += f64::from(self.mss) * (f64::from(m.acked) / self.cwnd);
            return
        }

        let mut link_uti = 1.0;
        if network_status.link_utilization < 1.0 {
            link_uti = network_status.link_utilization
        }

        self.cwnd *= 3.0 / (2.0 * link_uti as f64 + 1.0);
        self.last_utilization = network_status.link_utilization;

//        let link_uti = network_status.link_utilization * 10.0;
//        let link_uti = link_uti as u32;
//        let link_uti = link_uti as f64 / 10.0;
//
//        {
//            self.cwnd /= link_uti;
//            return
//        }
        // increase cwnd by 1 / cwnd per packet
//        println!("Link underutilized. Increase cwnd by 1 / cwnd per packet");
//        self.cwnd += f64::from(self.mss) * (f64::from(m.acked) / self.cwnd);
    }

    fn update_network_status(&mut self) -> NetworkStatus {
        let request_url = format!("http://127.0.0.1:8080/get_link_utilization");
        let mut response = reqwest::get(&request_url).unwrap();

        response.json().unwrap()
    }
}
