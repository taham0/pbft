use types::{Msg, ProtMsg};

use super::Context;

impl Context {
    // A function's input parameter needs to be borrowed as mutable only when
    // we intend to modify the variable in the function. Otherwise, it need not be borrowed as mutable.
    // In this example, the mut can (and must) be removed because we are not modifying the Context inside
    // the function. 
    // pub async fn start_ping(self: &mut Context){
    //     // Draft a message
    //     let data;

    //     if self.is_leader {
    //         data = "I am leader".to_string();
    //     } else {
    //         data = "I am not leader".to_string();
    //     }

    //     let msg = Msg{
    //         content: data.as_bytes().to_vec(),
    //         origin: self.myid
    //     };
        
    //     // Wrap the message in a type
    //     // Use different types of messages like INIT, ECHO, .... for the Bracha's RBC implementation
    //     // let protocol_msg = ProtMsg::Init(msg, self.myid);
    //     let protocol_msg = ProtMsg::Ping(msg, 1);

    //     // Broadcast the message to everyone
    //     self.broadcast(protocol_msg).await;
    // }

    pub async fn start_init(self: &mut Context) {
        let protocol_msg = ProtMsg::Init(
            self.myid as u64
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

        if self.echo_quorum == self.num_faults + 1 {
            // broadcast echo msg
            self.broadcast(ProtMsg::Echo(Msg {
                content: (values.clone()), 
                origin: (self.myid) 
            })).await;

            self.echo = false;
        }

        else if self.echo_quorum == self.num_nodes - self.num_faults {
            // deliver values
            log::info!("Delivering value vector {:?}", values);
        }
    }
}