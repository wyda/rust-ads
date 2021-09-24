use chrono::Duration;
use core::ads::*;
use core::connection::AmsConnection;
use core::port::AdsPort;
use core::requests::*;
use core::responses::*;
use std::collections::HashMap;
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::sync::{Arc, RwLock};

// TODO get rid of all those lifetimes

/// max amount of connections the router can handle
pub const MAX_PORTS: usize = 128;

pub const PORT_BASE: usize = 3000;

#[derive(Debug)]
pub struct RouterState<'a> {
    local_ams_net_id: AmsNetId,
    ports: Vec<AdsPort<'a>>,
}

impl<'a> RouterState<'a> {
    pub fn new(local_ams_net_id: AmsNetId) -> Self {
        RouterState {
            local_ams_net_id,
            ports: Vec::with_capacity(MAX_PORTS),
        }
    }
}

/// Router is central Notification Broker -> Dispatches all incoming/outgoing notifications

// TODO are all those mutexes necessary?

/// A Ams Router that manages routes and connections
///
/// workflow:   1. open port
///             2. Add a  route
///             3. check if already available
///             4. create new connection that spwans a TcpListener
// TODO can we remove the lifetimes?
#[derive(Debug)]
pub struct AmsRouter<'a> {
    /// current configuration of the router
    state: Arc<RwLock<RouterState<'a>>>,
    /// all connections to this router
    // TODO refactor as hashmap
    connections: Vec<Arc<RwLock<AmsConnection<'a>>>>,
    //    /// ports used connected to ads devices
    //    ports: Vec<Arc<RwLock<AdsPort<'a>>>>,
    mappings: HashMap<AmsNetId, Arc<RwLock<AmsConnection<'a>>>>,
}

impl<'a> AmsRouter<'a> {
    /// create a new AmsRouter with the local ams net id
    pub fn new(local_ams_net_id: AmsNetId) -> AmsRouter<'a> {
        let state = Arc::new(RwLock::new(RouterState::new(local_ams_net_id)));
        AmsRouter {
            state,
            connections: Vec::new(),
            mappings: HashMap::new(),
        }
    }

    pub fn open_port(&mut self) -> Result<u16> {
        let mut lock = self.state.write().map_err(|_| AdsError::SyncError)?;

        for port in lock.ports.iter_mut() {
            if port.is_closed() {
                return Ok(port.open());
            }
        }
        if lock.ports.len() >= MAX_PORTS {
            return Err(AdsError::NoMemoryLeft);
        }
        let open_port = PORT_BASE + lock.ports.len();
        lock.ports.push(AdsPort::new(open_port as u16, State::OPEN));
        Ok(open_port as u16)
    }

    fn port_in_range(&self, port: usize) -> Result<bool> {
        let lock = self.state.write().map_err(|_| AdsError::SyncError)?;
        Ok(port >= PORT_BASE && port < PORT_BASE + lock.ports.len())
    }

    pub fn close_port(&mut self, port: u16) -> Result<u16> {
        if !self.port_in_range(port as usize)? {
            return Err(AdsError::BadPort(port));
        }
        let mut lock = self.state.write().map_err(|_| AdsError::SyncError)?;
        let mut p = lock
            .ports
            .get_mut((port as usize) - (PORT_BASE + 1))
            .ok_or(AdsError::BadPort(port))?;
        if p.is_open() {
            Ok(p.close())
        } else {
            Err(AdsError::PortNotOpen(port))
        }
    }

    fn is_port_open(&self, port: u16) -> Result<bool> {
        if !self.port_in_range(port as usize)? {
            return Err(AdsError::BadPort(port));
        }
        let lock = self.state.read().map_err(|_| AdsError::SyncError)?;
        let p = lock
            .ports
            .get((port as usize) - (PORT_BASE + 1))
            .ok_or(AdsError::BadPort(port))?;
        Ok(p.is_open())
    }

    /// add a new route with the ams net id targeting the ipv4 address
    pub fn add_route(
        &'a mut self,
        ams_id: AmsNetId,
        ipv4: Ipv4Addr,
    ) -> Result<Arc<RwLock<AmsConnection<'a>>>> {
        // TODO refactor
        let lock = {
            let mut cc = None;
            for conn in &self.connections {
                if let Ok(lock) = conn.read() {
                    if *lock.ams_id() == ams_id {
                        cc = Some(conn.clone());
                        break;
                    }
                } else {
                    return Err(AdsError::InvalidAddress);
                }
            }
            cc
        };
        match lock {
            Some(rw) => {
                let conn = rw.read().map_err(|_| AdsError::SyncError)?;
                let res = match conn.dest_id() {
                    ipv4 => Err(AdsError::PortAlreadyInUse(3000)),
                    // TODO fix this necessary clone
                    _ => Ok(rw.clone()),
                };
                res
            }
            _ => {
                // insert a new connection for the ams id
                let conn = Arc::new(RwLock::new(AmsConnection::new(
                    Arc::clone(&self.state),
                    ipv4,
                    ams_id,
                )));
                self.connections.push(Arc::clone(&conn));
                Ok(conn)
            }
        }
    }

    /// create route with the ipv4 address derived from the netid
    pub fn add_route_derive(
        &'a mut self,
        ams_id: AmsNetId,
    ) -> Result<Arc<RwLock<AmsConnection<'a>>>> {
        let ipv4 = ams_id.clone().into();
        self.add_route(ams_id, ipv4)
    }

    pub fn delete_route(&mut self, addr: &AmsNetId) -> Result<()> {
        self.mappings
            .remove(addr)
            .map(|_| ())
            .ok_or(AdsError::InvalidAddress)
    }

    // TODO figure out how to pass both ams net id and ipv4 addr as ref?! mb as trait object?!
    fn any_conn(&'a self, addr: &AmsNetId) -> Option<Result<&'a RwLock<AmsConnection>>> {
        for conn in &self.connections {
            if let Ok(lock) = conn.read() {
                if lock.ams_id() == addr {
                    return Some(Ok(conn));
                }
            } else {
                return Some(Err(AdsError::InvalidAddress));
            }
        }
        None
    }

    /// find the Connection matching the ams id
    pub fn connection(&'a self, addr: &AmsNetId) -> Result<&'a RwLock<AmsConnection>> {
        match self.any_conn(addr) {
            Some(lock) => lock,
            _ => Err(AdsError::InvalidAddress),
        }
    }

    /// get the local address to this router described by the ams net id and the port
    pub fn local_address(&self, port: u16) -> Result<AmsAddress> {
        let is_open = self.is_port_open(port)?;
        if !is_open {
            Err(AdsError::PortNotOpen(port))
        } else {
            let lock = self.state.read().map_err(|_| AdsError::SyncError)?;
            Ok(AmsAddress::new(lock.local_ams_net_id.clone(), port))
        }
    }

    /// update the current ams net id of the router
    pub fn set_local_net_id(&'a mut self, net_id: AmsNetId) -> Result<&'a RwLock<RouterState>> {
        let mut lock = self.state.write().map_err(|_| AdsError::SyncError)?;
        lock.local_ams_net_id = net_id;
        Ok(&self.state)
    }

    pub fn add_notification<T: AdsCommandPayload>(
        &mut self,
        request: &AdsRequest<T>,
    ) -> Result<()> {
        unimplemented!()
    }

    pub fn read_request_sync<T: AmsRequest>(
        &mut self,
        req: T,
        addr: &AmsAddress,
    ) -> Result<AdsReadResponse> {
        let open = self.is_port_open(addr.port)?;
        if !open {
            return Err(AdsError::PortNotOpen(addr.port));
        }

        // TODO 1. get the matching connection to targeted address
        // 2. execute the request on the connection

        Err(AdsError::ConnectionError)
    }

    pub fn set_port_timeout(&'a mut self, port: u16, timeout: Duration) -> Result<()> {
        if !self.port_in_range(port as usize)? {
            return Err(AdsError::BadPort(port));
        }
        let mut lock = self.state.write().map_err(|_| AdsError::SyncError)?;
        let mut p = lock
            .ports
            .get_mut((port as usize) - (PORT_BASE + 1))
            .ok_or(AdsError::BadPort(port))?;
        p.timeout = Some(timeout);
        Ok(())
    }
}
