use types::{Msg, ProtMsg};

use super::Context;

impl Context {
    // A function's input parameter needs to be borrowed as mutable only when
    // we intend to modify the variable in the function. Otherwise, it need not be borrowed as mutable.
    // In this example, the mut can (and must) be removed because we are not modifying the Context inside
    // the function. 
    
    pub async fn start_init(self: &mut Context) {
        let protocol_msg = ProtMsg::Init(
            self.inp_message
        );

        // echo propose value
        self.broadcast(protocol_msg).await;
    }

    pub async fn handle_init(&mut self, msg:u64, sender:usize){
        // ignore incoming values if quorum reached
        if self.quorum == (2*self.num_faults) + 1 {
            return;
        }

        // only process init messages at the leader
        if self.is_leader {
            log::info!("Received init message {:?} from node {}", msg, sender);

            self.values.push(msg);
            self.quorum = self.values.len();
            log::info!("Total values received: {:?}, Values: {:?}", self.quorum, self.values);

            // if quorum reached start echo stage
            if self.quorum == (2*self.num_faults) + 1 {
                self.quorum = 0;

                log::info!("quorum reached, beginning echo stage...");
                
                // broadcast echo msg
                self.broadcast(ProtMsg::Prepare(Msg {
                    content: (self.values.clone()), 
                    origin: (self.myid) 
                })).await;
            }
        }
    }

    pub async fn handle_prepare(&mut self, values: Vec<u64>, sender_id: usize) {
        log::info!("received values vector {:?} from node {:?}", values, sender_id);

        if !self.is_leader {
            if self.echo {
                // broadcast echo msg
                self.broadcast(ProtMsg::Echo(Msg {
                    content: (values.clone()), 
                    origin: (self.myid) 
                })).await;

                self.echo = false;
            }
        }
    }

    pub async fn handle_echo(&mut self, values: Vec<u64>, sender_id: usize) {
        log::info!("received values vector {:?} from node {:?}", values, sender_id);
        self.echo_quorum += 1;

        let count = self.vec_counts.entry(values.clone()).or_insert(0);
        *count += 1;

        let f_plus_one_responses = (self.num_faults + 1) as u64;

        let result = self.vec_counts.iter()
            .find(|&(_, &count) | count == f_plus_one_responses)
            .map(|(vec, _)| vec.clone());

        match result {
            Some(vec) => {
                log::info!("Found vector: {:?}", vec);

                if self.echo {
                    // broadcast echo msg
                    self.broadcast(ProtMsg::Echo(Msg {
                        content: (vec.clone()), 
                        origin: (self.myid) 
                    })).await;
    
                    self.echo = false;
                }
            },
            None => (),
        }

        // check if enough responses received for delivery
        let n_minus_f_responses = (self.num_nodes - self.num_faults) as u64;

        let delivery_res = self.vec_counts.iter()
            .find(|&(_, &count) | count == n_minus_f_responses)
            .map(|(vec2, _)| vec2.clone());

        match delivery_res {
            Some(vec2) => {
                log::info!("Delivering vector: {:?}", vec2);

                let mut vec3 = vec2.clone();

                // get the median value
                vec3.sort_unstable();

                let result: String;
    
                let mid = vec3.len() / 2;
                if vec3.len() % 2 == 0 {
                    let mid_val = (vec3[mid - 1] as f64 + vec3[mid] as f64) / 2.0;
                    result = format!("{:?}", mid_val);
                } else {
                    // If odd, return the middle element
                    result = format!("{:?}", vec3[mid]);
                }

                self.terminate(result).await;
            },
            None => (),
        }

        // if self.echo_quorum == self.num_nodes - self.num_faults {
        //     // deliver values
        //     log::info!("Delivering value vector {:?}", values);
        // }
    }
}