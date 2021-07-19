use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum AmsAddressError {        
    #[error("Failed parsing address from &str")]
    ParseError{            
        source: std::num::ParseIntError           
    },
    #[error("Supplied address length {}! Expected a length of 6", length)]
    InvalidAddressLength{
        length: usize,
    }, 
}

#[derive(Error, Debug, PartialEq)]
pub enum AdsError {
    
}