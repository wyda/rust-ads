use ads::client::{
    ads_client::Connection,
    plc_types::{PlcTypes, Var},
};
use ads::proto::{ads_transition_mode::AdsTransMode, ams_address::*};
use std::net::Ipv4Addr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() {
    //Connect to remote device
    let ams_targed_address = AmsAddress::new(AmsNetId::from([192, 168, 0, 150, 1, 1]), 851);
    let route = Some(Ipv4Addr::new(192, 168, 0, 150));
    let mut connection = Connection::new(route, ams_targed_address);

    match connection.connect() {
        Ok(_) => (),
        Err(e) => {
            println!("Failed to connect to remote ADS device!\n{}", e);
            return;
        }
    };

    //Add device notification
    let var = Var::new("Main._Dint", PlcTypes::DInt);
    let notification_rx;
    match connection.add_device_notification(&var, AdsTransMode::OnChange, 10, 10, 2222) {
        Ok(rx) => notification_rx = rx,
        Err(e) => {
            println!("failed to add device notification!\n{}", e);
            return;
        }
    };

    let cancle = Arc::new(AtomicBool::new(false));
    /*let c = Arc::clone(&cancle);
        ctrlc::set_handler(move || {
            println!("Recieved CTRL-C....");
            c.swap(true, Ordering::Relaxed);
        })
        .expect("Error setting Ctrl-C handler");
    */
    let mut counter = 0;
    while !cancle.load(Ordering::Relaxed) && counter < 10 {
        if let Ok(Ok(stream)) = notification_rx.recv() {
            println!("got following response: \n{:?}", stream);
            counter += 1;
        };
    }

    println!("delete device notifications......");
    connection
        .delete_device_notification(&var, 999)
        .expect("Failed to release handle");
    println!("delete device notifications......");
}
