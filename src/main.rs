use ads::client::{
    ads_client::Connection,
    plc_types::{PlcTypes, Var},
};
use ads::proto::ads_state::AdsState;
use ads::proto::{ads_transition_mode::AdsTransMode, ams_address::*};
use byteorder::{LittleEndian, ReadBytesExt};
use std::net::Ipv4Addr;

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

    //Read device info
    match connection.read_device_info(123) {
        Ok(r) => {
            println!("Device Info: {:?}", r);
            println!("Device name: {:?}", r.get_device_name().unwrap());
        }
        Err(e) => println!("Error reading Device Info!  {:?}", e),
    }

    //Read state
    match connection.read_state(312) {
        Ok(r) => println!("Device Info: {:?}", r),
        Err(e) => println!("Error reading Device Info!  {:?}", e),
    }

    //Get multiple symhandles
    let mut var_list: Vec<Var> = Vec::new();
    var_list.push(Var::new(
        "_u32",
        PlcTypes::UInt
    ));

    var_list.push(Var::new(
        "_f32",
        PlcTypes::LReal
    ));

    var_list.push(Var::new(
        "_f16",
        PlcTypes::Real
    ));

    let result = connection.sum_get_symhandle(var_list, 132).unwrap();
    println!("{:?}", result);

    //Read by name
    let mut value = 0;
    let var = Var::new("Main.counter", PlcTypes::DInt);
    match connection.read_by_name(&var, 456) {
        Ok(r) => {
            value = r
                .as_slice()
                .read_u32::<LittleEndian>()
                .expect("Failed to read u32 from u8 slice");
            println!("Read value:  {:?}", value);
        }
        Err(e) => println!("Error reading by name   {:?}", e),
    }

    //Write by name
    let var = Var::new("Main.counter", PlcTypes::DInt);
    value += 1;
    match connection.write_by_name(&var, 456, value.to_le_bytes().to_vec()) {
        Ok(r) => println!("Write successfull {:?}", r),
        Err(e) => println!("Error writing by name   {:?}", e),
    }

    //Read by name
    let var = Var::new("Main.counter", PlcTypes::DInt);
    match connection.read_by_name(&var, 98) {
        Ok(r) => {
            let value = r
                .as_slice()
                .read_u32::<LittleEndian>()
                .expect("Failed to read u32 from u8 slice");
            println!("Read value:  {:?}", value);
        }
        Err(e) => println!("Error reading by name   {:?}", e),
    }

    //Add device notification
    let var = Var::new("Main._dint", PlcTypes::DInt);
    let notification_rx;
    match connection.add_device_notification(&var, AdsTransMode::OnChange, 10, 10, 2222) {
        Ok(rx) => notification_rx = rx,
        Err(e) => {
            println!("failed to add device notification!\n{}", e);
            return;
        }
    };

    let mut counter = 0;
    while counter < 2 {
        if let Ok(Ok(stream)) = notification_rx.recv() {
            println!("got following response: \n{:?}", stream);
            counter += 1;
        };
    }

    let var = Var::new("Main._dint", PlcTypes::DInt);
    println!("delete device notifications......");
    connection
        .delete_device_notification(&var, 999) //ToDo Reading response not worknig!
        .expect("Failed to release handle");
    println!("delete device notifications......");

    //Write control device stop
    match connection.write_control(AdsState::AdsStateStop, 0, 8888) {
        Ok(r) => println!("Write control successfull {:?}", r),
        Err(e) => println!("Error write control   {:?}", e),
    }
}
