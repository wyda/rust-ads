use ads::client::{
    ads_client::Connection,
    plc_types::{PlcTypes, Var},
};
use ads::proto::ads_state::AdsState;
use ads::proto::{ads_transition_mode::AdsTransMode, ams_address::*};
use byteorder::{LittleEndian, ReadBytesExt};
use std::net::Ipv4Addr;
use std::thread::sleep;
use std::time::Duration;

//Playground for testing proto

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

    //Write control device start
    match connection.write_control(AdsState::AdsStateStart, 0, 8888) {
        Ok(r) => println!("Write control successfull {:?}", r),
        Err(e) => println!("Error write control   {:?}", e),
    }

    //Read state.
    match connection.read_state(312) {
        Ok(r) => {
            if r.ads_state == AdsState::AdsStateStop {
                //Write control device start
                match connection.write_control(AdsState::AdsStateStart, 0, 8888) {
                    Ok(r) => println!("Write control successfull {:?}", r),
                    Err(e) => println!("Error write control   {:?}", e),
                }
            }
        }
        Err(e) => println!("Error reading Device Info!  {:?}", e),
    }

    //Get multiple symhandles
    let mut var_list: Vec<Var> = vec![
        Var::new("Main._udint".to_string(), PlcTypes::UDInt, None),
        Var::new("Main._lreal".to_string(), PlcTypes::LReal, None),
        Var::new("Main._int".to_string(), PlcTypes::Int, None),
        Var::new("Main.counter".to_string(), PlcTypes::DInt, None),
    ];

    if connection.sumup_get_symhandle(&var_list, 132).is_ok() {
        println!("got handles for all variables");
    } else {
        println!("failed to get all handles");
    }

    //Read by name
    let mut value = 0;
    let var = Var::new("Main.counter".to_string(), PlcTypes::DInt, None);
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
    let var = Var::new("Main.counter".to_string(), PlcTypes::DInt, None);
    value += 1;
    match connection.write_by_name(&var, 456, value.to_le_bytes().to_vec()) {
        Ok(r) => println!("Write successfull {:?}", r),
        Err(e) => println!("Error writing by name   {:?}", e),
    }

    //Read by name
    let var = Var::new("Main.counter".to_string(), PlcTypes::DInt, None);
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
    let var = Var::new("Main._dint".to_string(), PlcTypes::DInt, None);
    let notification_rx;
    match connection.add_device_notification(&var, AdsTransMode::OnChange, 10, 10, 2222) {
        Ok(rx) => notification_rx = rx,
        Err(e) => {
            println!("failed to add device notification!\n{}", e);
            return;
        }
    };
    println!("added device notification");
    let mut counter = 0;
    while counter <= 1000 {
        match notification_rx.recv() {
            Ok(s) => match s {
                Ok(r) => {
                    println!("{} got following response: \n{:?}", counter, r);
                    counter += 1;
                }
                Err(e) => {
                    println!("Error from notification {:?}", e);
                    panic!()
                }
            },
            Err(e) => {
                println!("error reading notification {:?}", e);
                panic!()
            }
        }
    }

    println!("try delete device notifications......");
    connection
        .delete_device_notification(&var, 999) //ToDo Reading response not worknig!
        .expect("Failed to release handle");
    println!("delete device notifications......");

    //Sumup read by name
    match connection.sumup_read_by_name(&var_list, 101) {
        Ok(read_result) => {
            if let Some(data) = read_result.get("Main._dint") {
                println!("{:?}", data.as_slice().read_u32::<LittleEndian>());
            }

            if let Some(data) = read_result.get("Main._lreal") {
                println!("{:?}", data.as_slice().read_f64::<LittleEndian>());
            }

            if let Some(data) = read_result.get("Main._int") {
                println!("{:?}", data.as_slice().read_u16::<LittleEndian>());
            }
        }
        Err(e) => println!("Sumup_read_by_name failed with error: {:?}", e),
    }

    //Sumup write by name
    var_list[0].data = vec![1, 0, 0, 0];
    var_list[1].data = vec![2, 0, 0, 0, 2, 0, 0, 0];
    var_list[2].data = vec![3, 0];
    var_list[3].data = vec![4, 0, 0, 0];

    match connection.sumup_write_by_name(&var_list, 101) {
        Ok(read_result) => {
            if let Some(result) = read_result.get("Main._udint") {
                println!("Main._udint -> {:?}", result);
            }

            if let Some(result) = read_result.get("Main._lreal") {
                println!("Main._lreal -> {:?}", result);
            }

            if let Some(result) = read_result.get("Main._int") {
                println!("Main._int -> {:?}", result);
            }

            if let Some(result) = read_result.get("Main.counter") {
                println!("Main.counter -> {:?}", result);
            }
        }
        Err(e) => println!("Sumup_read_by_name failed with error: {:?}", e),
    }

    //Write control device stop
    match connection.write_control(AdsState::AdsStateStop, 0, 8888) {
        Ok(r) => println!("Write control successfull {:?}", r),
        Err(e) => println!("Error write control   {:?}", e),
    }

    if let Some(sender) = connection.tx_thread_cancel {
        println!("cancel reader thread -> {:?}", sender.send(true));
    }

    println!("Sleep 5 seconds");
    sleep(Duration::from_millis(5000))
}
