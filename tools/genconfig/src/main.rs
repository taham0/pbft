// A tool that builds config files for all the nodes and the clients for the
// protocol.

use crypto::{secp256k1::{SecretKey}};
use config::{Node, Client};
use clap::{load_yaml, App};
use rand::Rng;
use types::Replica;
use crypto::Algorithm;
use std::{error::Error, io::{BufWriter, Write}, fs::File};
use util::io::*;
use fnv::FnvHashMap as HashMap;

// fn new_root_cert() -> Result<(X509, PKey<Private>), ErrorStack> {
//     let rsa = Rsa::generate(2048)?;
//     let privkey = PKey::from_rsa(rsa)?;

//     let mut x509_name = X509NameBuilder::new()?;
//     x509_name.append_entry_by_text("C", "US")?;
//     x509_name.append_entry_by_text("ST", "IN")?;
//     x509_name.append_entry_by_text("O", "Libchatter Test")?;
//     x509_name.append_entry_by_text("CN", "Root")?;
//     let x509_name = x509_name.build();

//     let mut cert_builder = X509::builder()?;
//     cert_builder.set_version(2)?;
//     let serial_number = {
//         let mut serial = BigNum::new()?;
//         serial.rand(159, MsbOption::MAYBE_ZERO, false)?;
//         serial.to_asn1_integer()?
//     };
//     cert_builder.set_serial_number(&serial_number)?;
//     cert_builder.set_subject_name(&x509_name)?;
//     cert_builder.set_issuer_name(&x509_name)?;
//     cert_builder.set_pubkey(&privkey)?;
//     let not_before = Asn1Time::days_from_now(0)?;
//     cert_builder.set_not_before(&not_before)?;
//     let not_after = Asn1Time::days_from_now(365)?;
//     cert_builder.set_not_after(&not_after)?;

//     cert_builder.append_extension(BasicConstraints::new().critical().ca().build()?)?;
//     cert_builder.append_extension(
//         KeyUsage::new()
//             .critical()
//             .key_cert_sign()
//             .crl_sign()
//             .build()?,
//     )?;

//     let subject_key_identifier =
//         SubjectKeyIdentifier::new().build(&cert_builder.x509v3_context(None, None))?;
//     cert_builder.append_extension(subject_key_identifier)?;

//     cert_builder.sign(&privkey, MessageDigest::sha256())?;
//     let cert = cert_builder.build();

//     Ok((cert, privkey))
// }

// /// Make a X509 request with the given private key
// fn mk_request(privkey: &PKey<Private>) -> Result<X509Req, ErrorStack> {
//     let mut req_builder = X509ReqBuilder::new()?;
//     req_builder.set_pubkey(&privkey)?;

//     let mut x509_name = X509NameBuilder::new()?;
//     x509_name.append_entry_by_text("C", "US")?;
//     x509_name.append_entry_by_text("ST", "IN")?;
//     x509_name.append_entry_by_text("O", "Nodes")?;
//     x509_name.append_entry_by_text("CN", "nodes.com")?;
//     let x509_name = x509_name.build();
//     req_builder.set_subject_name(&x509_name)?;

//     req_builder.sign(&privkey, MessageDigest::sha256())?;
//     let req = req_builder.build();
//     Ok(req)
// }

// /// Make a certificate and private key signed by the given CA cert and private key
// fn get_signed_cert(
//     ca_cert: &X509Ref,
//     ca_privkey: &PKeyRef<Private>,
// ) -> Result<(X509, PKey<Private>), ErrorStack> {
//     let rsa = Rsa::generate(2048)?;
//     let privkey = PKey::from_rsa(rsa)?;

//     let req = mk_request(&privkey)?;

//     let mut cert_builder = X509::builder()?;
//     cert_builder.set_version(2)?;
//     let serial_number = {
//         let mut serial = BigNum::new()?;
//         serial.rand(159, MsbOption::MAYBE_ZERO, false)?;
//         serial.to_asn1_integer()?
//     };
//     cert_builder.set_serial_number(&serial_number)?;
//     cert_builder.set_subject_name(req.subject_name())?;
//     cert_builder.set_issuer_name(ca_cert.subject_name())?;
//     cert_builder.set_pubkey(&privkey)?;
//     let not_before = Asn1Time::days_from_now(0)?;
//     cert_builder.set_not_before(&not_before)?;
//     let not_after = Asn1Time::days_from_now(365)?;
//     cert_builder.set_not_after(&not_after)?;

//     cert_builder.append_extension(BasicConstraints::new().build()?)?;

//     cert_builder.append_extension(
//         KeyUsage::new()
//             .critical()
//             .non_repudiation()
//             .digital_signature()
//             .key_encipherment()
//             .build()?,
//     )?;

//     let subject_key_identifier =
//         SubjectKeyIdentifier::new().build(&cert_builder.x509v3_context(Some(ca_cert), None))?;
//     cert_builder.append_extension(subject_key_identifier)?;

//     let auth_key_identifier = AuthorityKeyIdentifier::new()
//         .keyid(false)
//         .issuer(false)
//         .build(&cert_builder.x509v3_context(Some(ca_cert), None))?;
//     cert_builder.append_extension(auth_key_identifier)?;

//     let subject_alt_name = SubjectAlternativeName::new()
//     //     .dns("*.example.com")
//         .dns("nodes.com")
//         .build(&cert_builder.x509v3_context(Some(ca_cert), None))?;
//     cert_builder.append_extension(subject_alt_name)?;

//     cert_builder.sign(&ca_privkey, MessageDigest::sha256())?;
//     let cert = cert_builder.build();

//     Ok((cert, privkey))
// }

fn main() -> Result<(), Box<dyn Error>> {
    let yaml = load_yaml!("cli.yml");
    let m = App::from_yaml(yaml).get_matches();
    let num_nodes:usize =  m.value_of("num_nodes")
        .expect("number of nodes not specified")
        .parse::<usize>()
        .expect("unable to convert number of nodes into a number");
    let num_faults:usize = match m.value_of("num_faults") {
        Some(x) => x.parse::<usize>()
            .expect("unable to convert number of faults into a number"),
        None => (num_nodes-1)/3,
    };
    let delay:u64 = m.value_of("delay")
        .expect("delay value not specified")
        .parse::<u64>()
        .expect("unable to parse delay value into a number");
    let base_port: u16 = m.value_of("base_port")
        .expect("base_port value not specified")
        .parse::<u16>()
        .expect("failed to parse base_port into a number");
    let blocksize: usize = m.value_of("block_size")
        .expect("no block_size specified")
        .parse::<usize>()
        .expect("unable to convert blocksize into a number");
    let client_base_port:u16 = m.value_of("client_base_port")
        .expect("no client_base_port specified")
        .parse::<u16>()
        .expect("unable to parse client_base_port into an integer");
    let t:Algorithm = m.value_of("algorithm")
        .unwrap_or("NOPKI")
        .parse::<Algorithm>()
        .unwrap_or(Algorithm::NOPKI);
    let out = m.value_of("out_type")
        .unwrap_or("json");
    let target = m.value_of("target")
        .expect("target directory for the config not specified");
    let payload:usize = m.value_of("payload")
        .unwrap_or("0")
        .parse()
        .unwrap();
    let local:String = m.value_of("local")
        .unwrap_or("false")
        .parse()
        .unwrap();
    let c_rport:u16 = m.value_of("client_run_port")
        .expect("Client port expected")
        .parse::<u16>()
        .expect("unable to parse client's port into an integer");
    let mut client = Client::new();
    client.block_size = blocksize;
    client.crypto_alg = t.clone();
    client.num_nodes = num_nodes;
    client.num_faults = num_faults;

    let mut node:Vec<Node> = Vec::with_capacity(num_nodes);

    let pk = HashMap::default();
    let mut ip = HashMap::default();
    
    //let (cert, privkey) = new_root_cert()?;
    let mut sec_keys:Vec<Vec<SecretKey>> = Vec::with_capacity(num_nodes);
    (0..num_nodes).for_each(|_i| {
        sec_keys.push(Vec::with_capacity(num_nodes));
    });
    if t == Algorithm::NOPKI{
        // Generate secret keys above and pass them to the context
        for i in 0..num_nodes{
            for j in i..num_nodes{
                let skey:SecretKey = SecretKey::generate();
                sec_keys[i].push(skey.clone());
                if j!= i{
                    sec_keys[j].push(skey.clone());
                }
                //sec_keys.push(SecretKey::generate());
            }
        }
    }
    for i in 0..num_nodes {
        node.push(Node::new());

        node[i].delta = delay;
        node[i].id = i as Replica;
        node[i].num_nodes = num_nodes;
        node[i].num_faults = num_faults;
        node[i].block_size = blocksize;
        node[i].payload = payload;
        node[i].client_port = client_base_port+(i as u16);
        // generate random number for approximate consensus
        let num = rand::thread_rng().gen_range(0, 20000000);
        node[i].prot_payload = format!("a,{},50000,100",num);
        //String::from("a,");
        //node[i].prot_payload = String::from("cc,/home/akhil/research/EEBA/libchatter/");
        node[i].crypto_alg = t.clone();
        match t {
            Algorithm::NOPKI =>{
                for j in 0..num_nodes{
                    node[i].sk_map.insert(j, sec_keys[i][j].to_bytes().to_vec());
                }
            }
        };
        ip.insert(i as Replica, 
        format!("{}:{}", "127.0.0.1", base_port+(i as u16))
        );
        client.net_map.insert(i as Replica, 
        format!("127.0.0.1:{}", client_base_port+(i as u16))
        );


        //let (new_cert, new_pkey) = get_signed_cert(&cert, &privkey)?;

        //node[i].root_cert = cert.to_der()?;
        //node[i].my_cert = new_cert.to_der()?;
        //node[i].my_cert_key = new_pkey.private_key_to_der()?;
    }
    ip.insert(num_nodes, format!("127.0.0.1:{}",c_rport));
    //client.root_cert = cert.to_der()?;

    for i in 0..num_nodes {
        node[i].pk_map = pk.clone();
        node[i].net_map = ip.clone();
    }
    if local != String::from("false"){
        // write ip map to file
        //let filename = format!("ip_file");
        println!("Writing ips to ip_file");
        // write ips to ip_file
        {
            let file = File::create("ip_file")?;
            let mut writer = BufWriter::new(file);
            for iter in 0..num_nodes+1{
                writeln!(writer,"{}",ip.get(&iter).unwrap())?;
            }
            writer.flush()?;
        }
        {
            let file = File::create(format!("{}/syncer",target))?;
            let mut writer = BufWriter::new(file);
            for iter in 0..num_nodes{
                writeln!(writer,"{}",client.net_map.get(&iter).unwrap())?;
            }
            writer.flush()?;
        }
        //write_json(filename, &ip.clone());
    }
    let filename = format!("{}/syncer.json",target);
    write_json(filename, &client.net_map.clone());
    client.server_pk = pk;

    // Write all the files
    for i in 0..num_nodes {
        match out {
            "json" => {
                let filename = format!("{}/nodes-{}.json",target,i);
                write_json(filename, &node[i]);
            },
            "binary" => {
                let filename = format!("{}/nodes-{}.dat",target,i);
                write_bin(filename, &node[i]);
            },
            "toml" => {
                let filename = format!("{}/nodes-{}.toml",target,i);
                write_toml(filename, &node[i]);
            },
            "yaml" => {
                let filename = format!("{}/nodes-{}.yml",target,i);
                write_yaml(filename, &node[i]);
            },
            _ => (),
        }
        node[i].validate()
            .expect("failed to validate node config");
    }

    // Write the client file
    match out {
        "json" => {
            let filename = format!("{}/client.json",target);
            write_json(filename, &client);
        },
        "binary" => {
            let filename = format!("{}/client.dat",target);
            write_bin(filename, &client);
        },
        "toml" => {
            let filename = format!("{}/client.toml",target);
            write_toml(filename, &client);
        },
        "yaml" => {
            let filename = format!("{}/client.yml",target);
            write_yaml(filename, &client);
        },
        _ => (),
    }
    client.validate()
        .expect("failed to validate the client config");

    Ok(())
}

#[test]
fn test_codec() -> Result<(), Box<dyn Error>>{
    // use rustls::{Certificate};

    // let (cert, _key) = new_root_cert()?;
    // let data = cert.to_der()?;
    // //let ok = Certificate(data);
    // //let mut config = ClientConfig::new();
    // //config.root_store.add(&ok)?;
    Ok(())
}