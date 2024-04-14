// use std::{collections::HashMap, net::{SocketAddr, SocketAddrV4}, time::{SystemTime, UNIX_EPOCH}};

// use anyhow::{Result, anyhow};
// use config::Node;
// use fnv::FnvHashMap;
// use network::{plaintcp::{TcpReceiver, TcpReliableSender, CancelHandler}, Acknowledgement};
// use tokio::sync::{oneshot, mpsc::{unbounded_channel, UnboundedReceiver}};
// // use tokio_util::time::DelayQueue;
// use types::{{WrapperMsg, Replica, ProtMsg}, SyncMsg, SyncState};

// use super::{Handler, SyncHandler};

// pub struct Context {
//     /// Networking context
//     pub net_send: TcpReliableSender<Replica,WrapperMsg,Acknowledgement>,
//     pub net_recv: UnboundedReceiver<WrapperMsg>,
//     pub sync_send:TcpReliableSender<Replica,SyncMsg,Acknowledgement>,
//     pub sync_recv: UnboundedReceiver<SyncMsg>,

//     /// Data context
//     pub num_nodes: usize,
//     pub myid: usize,
//     pub num_faults: usize,
//     pub inp_message:u64,
//     pub values: Vec<u64>,
//     pub quorum: usize,
//     pub echo: bool,
//     pub echo_quorum: usize,
//     pub vec_counts: HashMap<Vec<u64>, u64>,

//     /// Secret Key map
//     pub sec_key_map:HashMap<Replica, Vec<u8>>,


//     /// Cancel Handlers
//     pub cancel_handlers: HashMap<u64,Vec<CancelHandler<Acknowledgement>>>,
//     exit_rx: oneshot::Receiver<()>,

//     // Add your custom fields here
//     pub is_leader: bool,
// }

// impl Context {
//     pub fn spawn(
//         config:Node,
//         message: u64
//     )->anyhow::Result<oneshot::Sender<()>>{
//         let mut consensus_addrs :FnvHashMap<Replica,SocketAddr>= FnvHashMap::default();

//         // populate consensus_addr with network ip mappings (node_id, ip)
//         for (replica, address) in config.net_map.iter(){
//             let address:SocketAddr = address.parse().expect("Unable to parse address");
//             consensus_addrs.insert(*replica, SocketAddr::from(address.clone()));
//         }

//         let mut is_leader = false;

//         // set leader
//         if config.id == 0 {
//             is_leader = true;
//         }
        
//         // get own address
//         let my_port = consensus_addrs.get(&config.id).unwrap();
//         let my_address = to_socket_address("0.0.0.0", my_port.port());
        
//         // why is the client hardcoded to key 0?
//         let mut syncer_map:FnvHashMap<Replica,SocketAddr> = FnvHashMap::default();
//         syncer_map.insert(0, config.client_addr);
        
//         // Setup networking
//         let (tx_net_to_consensus, rx_net_to_consensus) = unbounded_channel();
        
//         TcpReceiver::<Acknowledgement, WrapperMsg, _>::spawn(
//             my_address,
//             Handler::new(tx_net_to_consensus),
//         );
        
//         // The server must listen to the client's messages on some port that is not being used to listen to other servers
//         let syncer_listen_port = config.client_port;
//         let syncer_l_address = to_socket_address("0.0.0.0", syncer_listen_port);
        
//         let (tx_net_to_client,rx_net_from_client) = unbounded_channel();
        
//         TcpReceiver::<Acknowledgement,SyncMsg,_>::spawn(
//             syncer_l_address, 
//             SyncHandler::new(tx_net_to_client)
//         );
        
//         let consensus_net = TcpReliableSender::<Replica,WrapperMsg,Acknowledgement>::with_peers(
//             consensus_addrs.clone()
//         );
        
//         let sync_net = TcpReliableSender::<Replica,SyncMsg,Acknowledgement>::with_peers(syncer_map);
//         let (exit_tx, exit_rx) = oneshot::channel();

//         let values = Vec::new();
        
//         tokio::spawn(async move {
//             let mut c = Context {
//                 net_send:consensus_net,
//                 net_recv:rx_net_to_consensus,
//                 sync_send: sync_net,
//                 sync_recv: rx_net_from_client,
//                 num_nodes: config.num_nodes,
//                 sec_key_map: HashMap::default(),
//                 myid: config.id,
//                 num_faults: config.num_faults,
//                 cancel_handlers:HashMap::default(),
//                 exit_rx: exit_rx,
//                 is_leader: is_leader,
//                 quorum: 0,
//                 inp_message:message,
//                 values: values,
//                 echo: true,
//                 echo_quorum: 0,
//                 vec_counts: HashMap::default(),
//             };
            
//             for (id, sk_data) in config.sk_map.clone() {
//                 c.sec_key_map.insert(id, sk_data.clone());
//             }
            
//             if let Err(e) = c.run().await {
//                 log::error!("Consensus error: {}", e);
//             }
//         });

//         Ok(exit_tx)
//     }

//     pub async fn broadcast(&mut self, protmsg:ProtMsg){
//         let sec_key_map = self.sec_key_map.clone();
//         for (replica,sec_key) in sec_key_map.into_iter() {
//             // if replica != self.myid{
//             //     let wrapper_msg = WrapperMsg::new(protmsg.clone(), self.myid, &sec_key.as_slice());
//             //     let cancel_handler:CancelHandler<Acknowledgement> = self.net_send.send(replica, wrapper_msg).await;
//             //     self.add_cancel_handler(cancel_handler);
//             // }

//             if self.is_leader || replica != self.myid{
//                 let wrapper_msg = WrapperMsg::new(protmsg.clone(), self.myid, &sec_key.as_slice());
//                 let cancel_handler:CancelHandler<Acknowledgement> = self.net_send.send(replica, wrapper_msg).await;
//                 self.add_cancel_handler(cancel_handler);
//             }
//         }
//     }

//     pub fn add_cancel_handler(&mut self, canc: CancelHandler<Acknowledgement>){
//         self.cancel_handlers
//             .entry(0)
//             .or_default()
//             .push(canc);
//     }

//     pub async fn send(&mut self,replica:Replica, wrapper_msg:WrapperMsg){
//         let cancel_handler:CancelHandler<Acknowledgement> = self.net_send.send(replica, wrapper_msg).await;
//         self.add_cancel_handler(cancel_handler);
//     }

//     pub async fn run(&mut self)-> Result<()>{
//         // The process starts listening to messages in this process. 
//         // First, the node sends an alive message 
//         let cancel_handler = self.sync_send.send(0,
//             SyncMsg { sender: self.myid, state: SyncState::ALIVE,value:"".to_string()}
//         ).await;

//         self.add_cancel_handler(cancel_handler);
        
//         loop {
//             tokio::select! {
//                 // Receive exit handlers
//                 exit_val = &mut self.exit_rx => {
//                     exit_val.map_err(anyhow::Error::new)?;
//                     log::info!("Termination signal received by the server. Exiting.");
//                     break
//                 },

//                 msg = self.net_recv.recv() => {
//                     // Received messages are processed here
//                     // net_recv is the internal unbounded channel
//                     log::debug!("Got a consensus message from the network: {:?}", msg);
//                     let msg = msg.ok_or_else(||
//                         anyhow!("Networking layer has closed")
//                     )?;
//                     self.process_msg(msg).await;
//                 },

//                 sync_msg = self.sync_recv.recv() =>{
//                     let sync_msg = sync_msg.ok_or_else(||
//                         anyhow!("Networking layer has closed")
//                     )?;

//                     match sync_msg.state {
//                         SyncState::START => {
//                             log::info!("Consensus Start time: {:?}", SystemTime::now()
//                                 .duration_since(UNIX_EPOCH)
//                                 .unwrap()
//                                 .as_millis());
//                             // Start your protocol from here
//                             // Write a function to broadcast a message. We demonstrate an example with a PING function

//                             self.start_init().await;

//                             let cancel_handler = self.sync_send.send(0, SyncMsg { sender: self.myid, state: SyncState::STARTED, value:"".to_string()}).await;
//                             self.add_cancel_handler(cancel_handler);
//                         },

//                         SyncState::STOP => {
//                             // Code used for internal purposes
//                             log::info!("Consensus Stop time: {:?}", SystemTime::now()
//                                 .duration_since(UNIX_EPOCH)
//                                 .unwrap()
//                                 .as_millis());
//                             log::info!("Termination signal received by the server. Exiting.");
//                             break
//                         },

//                         _=>{}
//                     }
//                 },
//             };
//         }

//         Ok(())
//     }

//     pub async fn terminate(&mut self, res: String) {
//         log::info!("terminating..");
//         let cancel_handler = self.sync_send.send(0,
//             SyncMsg { sender: self.myid, state: SyncState::COMPLETED, value:res}
//         ).await;

//         self.add_cancel_handler(cancel_handler);
//     }
// }

// pub fn to_socket_address(
//     ip_str: &str,
//     port: u16,
// ) -> SocketAddr {
//     let addr = SocketAddrV4::new(ip_str.parse().unwrap(), port);
//     addr.into()
// }

use std::{collections::HashMap, net::{SocketAddr, SocketAddrV4}, time::{SystemTime, UNIX_EPOCH}};

use anyhow::{Result, anyhow};
use config::Node;
use fnv::FnvHashMap;
use network::{plaintcp::{TcpReceiver, TcpReliableSender, CancelHandler}, Acknowledgement};
use tokio::sync::{oneshot, mpsc::{unbounded_channel, UnboundedReceiver}};
// use tokio_util::time::DelayQueue;
use types::{{WrapperMsg, Replica, ProtMsg}, SyncMsg, SyncState};

use super::{Handler, SyncHandler};

pub struct Context {
    /// Networking context
    pub net_send: TcpReliableSender<Replica,WrapperMsg,Acknowledgement>,
    pub net_recv: UnboundedReceiver<WrapperMsg>,
    pub sync_send:TcpReliableSender<Replica,SyncMsg,Acknowledgement>,
    pub sync_recv: UnboundedReceiver<SyncMsg>,

    /// Data context
    pub num_nodes: usize,
    pub myid: usize,
    pub num_faults: usize,
    pub inp_message:u64,
    pub values: Vec<u64>,
    pub quorum: usize,
    pub echo: bool,
    pub echo_quorum: usize,
    pub vec_counts: HashMap<Vec<u64>, u64>,
    pub byz: bool,

    /// Secret Key map
    pub sec_key_map:HashMap<Replica, Vec<u8>>,


    /// Cancel Handlers
    pub cancel_handlers: HashMap<u64,Vec<CancelHandler<Acknowledgement>>>,
    exit_rx: oneshot::Receiver<()>,

    // Add your custom fields here
    pub is_leader: bool,
}

impl Context {
    pub fn spawn(
        config:Node,
        message: u64,
        byz: bool
    )->anyhow::Result<oneshot::Sender<()>>{
        let mut consensus_addrs :FnvHashMap<Replica,SocketAddr>= FnvHashMap::default();
        for (replica,address) in config.net_map.iter(){
            let address:SocketAddr = address.parse().expect("Unable to parse address");
            consensus_addrs.insert(*replica, SocketAddr::from(address.clone()));
        }

        let mut is_leader = false;

        // set leader
        if config.id == 0 {
            is_leader = true;
        }

        let my_port = consensus_addrs.get(&config.id).unwrap();
        let my_address = to_socket_address("0.0.0.0", my_port.port());
        let mut syncer_map:FnvHashMap<Replica,SocketAddr> = FnvHashMap::default();
        syncer_map.insert(0, config.client_addr);
        
        // Setup networking
        let (tx_net_to_consensus, rx_net_to_consensus) = unbounded_channel();
        TcpReceiver::<Acknowledgement, WrapperMsg, _>::spawn(
            my_address,
            Handler::new(tx_net_to_consensus),
        );
        let syncer_listen_port = config.client_port;
        let syncer_l_address = to_socket_address("0.0.0.0", syncer_listen_port);
        // The server must listen to the client's messages on some port that is not being used to listen to other servers
        let (tx_net_to_client,rx_net_from_client) = unbounded_channel();
        TcpReceiver::<Acknowledgement,SyncMsg,_>::spawn(
            syncer_l_address, 
            SyncHandler::new(tx_net_to_client)
        );
        let consensus_net = TcpReliableSender::<Replica,WrapperMsg,Acknowledgement>::with_peers(
            consensus_addrs.clone()
        );

        let values = Vec::new();
        
        let sync_net = TcpReliableSender::<Replica,SyncMsg,Acknowledgement>::with_peers(syncer_map);
        let (exit_tx, exit_rx) = oneshot::channel();
        tokio::spawn(async move {
            let mut c = Context {
                net_send:consensus_net,
                net_recv:rx_net_to_consensus,
                sync_send: sync_net,
                sync_recv: rx_net_from_client,
                num_nodes: config.num_nodes,
                sec_key_map: HashMap::default(),
                myid: config.id,
                num_faults: config.num_faults,
                cancel_handlers:HashMap::default(),
                exit_rx: exit_rx,
                is_leader: is_leader,
                quorum: 0,
                inp_message:message,
                values: values,
                echo: true,
                echo_quorum: 0,
                vec_counts: HashMap::default(),
                byz: byz,
            };
            for (id, sk_data) in config.sk_map.clone() {
                c.sec_key_map.insert(id, sk_data.clone());
            }
            //c.invoke_coin.insert(100, Duration::from_millis(sleep_time.try_into().unwrap()));
            if let Err(e) = c.run().await {
                log::error!("Consensus error: {}", e);
            }
        });
        Ok(exit_tx)
    }

    pub async fn broadcast(&mut self, protmsg:ProtMsg){
        let sec_key_map = self.sec_key_map.clone();
        for (replica,sec_key) in sec_key_map.into_iter() {
            if self.byz && replica%2 == 0{
                // Simulates a crash fault
                continue;
            }
            // if replica != self.myid{
            //     let wrapper_msg = WrapperMsg::new(protmsg.clone(), self.myid, &sec_key.as_slice());
            //     let cancel_handler:CancelHandler<Acknowledgement> = self.net_send.send(replica, wrapper_msg).await;
            //     self.add_cancel_handler(cancel_handler);
            // }

            if self.is_leader || replica != self.myid{
                let wrapper_msg = WrapperMsg::new(protmsg.clone(), self.myid, &sec_key.as_slice());
                let cancel_handler:CancelHandler<Acknowledgement> = self.net_send.send(replica, wrapper_msg).await;
                self.add_cancel_handler(cancel_handler);
            }
        }
    }

    pub fn add_cancel_handler(&mut self, canc: CancelHandler<Acknowledgement>){
        self.cancel_handlers
            .entry(0)
            .or_default()
            .push(canc);
    }

    pub async fn send(&mut self,replica:Replica, wrapper_msg:WrapperMsg){
        let cancel_handler:CancelHandler<Acknowledgement> = self.net_send.send(replica, wrapper_msg).await;
        self.add_cancel_handler(cancel_handler);
    }

    pub async fn run(&mut self)-> Result<()>{
        // The process starts listening to messages in this process. 
        // First, the node sends an alive message 
        let cancel_handler = self.sync_send.send(0,
            SyncMsg { sender: self.myid, state: SyncState::ALIVE,value:"".to_string()}
        ).await;
        self.add_cancel_handler(cancel_handler);
        loop {
            tokio::select! {
                // Receive exit handlers
                exit_val = &mut self.exit_rx => {
                    exit_val.map_err(anyhow::Error::new)?;
                    log::info!("Termination signal received by the server. Exiting.");
                    break
                },
                msg = self.net_recv.recv() => {
                    // Received messages are processed here
                    log::debug!("Got a consensus message from the network: {:?}", msg);
                    let msg = msg.ok_or_else(||
                        anyhow!("Networking layer has closed")
                    )?;
                    self.process_msg(msg).await;
                },
                sync_msg = self.sync_recv.recv() =>{
                    let sync_msg = sync_msg.ok_or_else(||
                        anyhow!("Networking layer has closed")
                    )?;
                    match sync_msg.state {
                        SyncState::START =>{
                            log::error!("Consensus Start time: {:?}", SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis());
                            // Start your protocol from here
                            // Write a function to broadcast a message. We demonstrate an example with a PING function
                            self.start_init().await;

                            let cancel_handler = self.sync_send.send(0, SyncMsg { sender: self.myid, state: SyncState::STARTED, value:"".to_string()}).await;
                            self.add_cancel_handler(cancel_handler);
                        },
                        SyncState::STOP =>{
                            // Code used for internal purposes
                            log::error!("Consensus Stop time: {:?}", SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis());
                            log::info!("Termination signal received by the server. Exiting.");
                            break
                        },
                        _=>{}
                    }
                },
            };
        }
        Ok(())
    }

    pub async fn terminate(&mut self, res: String) {
        log::info!("terminating..");
        let cancel_handler = self.sync_send.send(0,
            SyncMsg { sender: self.myid, state: SyncState::COMPLETED, value:res}
        ).await;

        self.add_cancel_handler(cancel_handler);
    }
}

pub fn to_socket_address(
    ip_str: &str,
    port: u16,
) -> SocketAddr {
    let addr = SocketAddrV4::new(ip_str.parse().unwrap(), port);
    addr.into()
}