use ads::proto::{
    ams_address::*,
    response::AdsNotificationStream,
    ads_transition_mode::AdsTransMode,
};
use ads::client::{
    ads_client::Connection,
    plc_types::{
        Var,
        PlcTypes,
    },
};
use std::net::Ipv4Addr;
use std::convert::TryInto;

fn main() {
    //Connect to remote device
    let ams_targed_address = AmsAddress::new(AmsNetId::from([192, 168, 0, 150, 1, 1]), 851);
    let route = Some(Ipv4Addr::new(192, 168, 0, 150));
    let mut connection = Connection::new(route, ams_targed_address);
    
    match connection.connect() {
        Ok(_) => (),
        Err(e) => {
            println!("Failed to connect to remote ADS device!\n{}", e);
            return
        }
    };

    //Add device notification
    let var = Var::new("Main.counter", PlcTypes::DInt);
    let notification_rx;
    match connection.add_device_notification(&var, AdsTransMode::OnChange, 10, 10, 1111) {
        Ok(rx) => notification_rx = rx,
        Err(e) => {
            println!("failed to add device notification!\n{}", e);
            return
        },
    };    

    let mut valid = true;
    while valid {
        match notification_rx.try_recv() {
            Ok(r) => {                
                if let Ok(stream) = r {
                    let response: AdsNotificationStream = match stream.try_into() {
                        Ok(r) => r,
                        Err(e) => {
                            println!("{:?}", e);
                            continue                            
                        }
                    };
                    println!("got following response: \n{:?}", response);
                }                
            }
            Err(e) => {
                println!("{:?}", e);
                valid = false;
                continue
            }            
        };
    }    
}