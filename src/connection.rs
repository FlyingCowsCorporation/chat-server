use std::io;
use std::io::prelude::*;

use std::net::TcpStream;
use std::net::Shutdown;
use std::net::SocketAddr;

pub struct Connection {
    stream : TcpStream,
}

const BUF_SIZE : usize = 512;

impl Connection {
    pub fn new(stream : TcpStream) -> Connection {
        Connection{
            stream
        }
    }

    pub fn read_more(&mut self, num_bytes : usize) -> Result<String, io::Error> {
        println!("    Reading an additional {} bytes...", num_bytes);
        match self.read_string() {
            Ok(data) => {
                if data.len() >= num_bytes {
                    println!("    Done. Read {} more bytes (needed {}).", data.len(), num_bytes);
                    Ok(data)
                } else {
                    println!("    Not yet done. Read {} more bytes (needed {}).", data.len(), num_bytes);
                    self.read_more(num_bytes - data.len())
                }
            },
            Err(err) => Err(err),
        }
    }

    pub fn read_string(&mut self) -> Result<String, io::Error> {

        let mut buffer = [0; BUF_SIZE];

        match self.stream.read(&mut buffer) {
            Ok(read) => {
                let mut data = String::from_utf8_lossy(&buffer[..read]).to_string();
                //println!("DATA: \"{}\"", data);
                if read == BUF_SIZE {
                    match self.has_data_available() {
                        Ok(available) => {
                            if available {
                                println!("  > Buffer is full, there is more data.");
                                match self.read_string() {
                                    Ok(next_data) => {
                                        data.push_str(&next_data);
                                        Ok(data)
                                    },
                                    Err(e) => Err(e),
                                }
                            } else {
                                println!("  > Buffer was filled perfectly, no more data.");
                                Ok(data)
                            }
                        }
                        Err(e) => Err(e),
                    }

                   
                } else {
                    Ok(data)
                }
            },
            Err(e) => Err(e)
        }
    }

    fn has_data_available(&self) -> Result<bool, io::Error> {

        // Set stream to nonblocking to ensure we won't block
        // while checking if there is data available.
        self.stream.set_nonblocking(true)?;

        let peek_result = self.stream.peek(&mut [0; 1]);
        
        // Return to a blocking state.
        self.stream.set_nonblocking(false)?;

        Ok(0 != match peek_result {
            Ok(read) => read,
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // Peek would block, so we can assume there
                // is currently no data available on the stream.
                0
            },
            Err(err) => {
                return Err(err);
            }
        })
    }

    pub fn write_data(&mut self, data : &str) {
        match self.stream.write(data.as_bytes()) {
            Ok(_) => {},
            Err(_) => {
                println!("--> Write failed: connection closed.");
                return;
            },
        }
        match self.stream.flush() {
            Ok(_) => {},
            Err(_) => println!("--> Flush failed: connection closed."), 
        }
    }

    pub fn peer_addr(&self) -> SocketAddr {
         self.stream.peer_addr().unwrap()
    }

    pub fn shutdown(&mut self) -> Result<(), io::Error>{
        self.stream.shutdown(Shutdown::Both)
    }
}