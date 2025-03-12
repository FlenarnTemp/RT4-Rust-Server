use std::cmp::min;
use std::error::Error;
use std::thread::sleep;
use log::debug;
use rs2cache::store::ARCHIVESET;
use cache::file_handler::{ensure_initialized, CACHE, MASTER_INDEX_VEC};
use cache::version_trailer::VersionTrailer;
use io::connection::Connection;
use io::packet::Packet;

const BLOCK_SIZE: usize = 512;
pub const BLOCK_HEADER_SIZE: usize = 1 + 2 + 1;
pub const BLOCK_DELIMITER_SIZE: usize = 1;
pub const BYTES_BEFORE_BLOCK: usize = BLOCK_SIZE - BLOCK_HEADER_SIZE;
pub const BYTES_AFTER_BLOCK: usize = BLOCK_SIZE - BLOCK_DELIMITER_SIZE;

#[derive(Debug)]
pub (crate) enum Js5Request {
    Group {
        urgent: bool,
        archive: u8,
        group: u16,
    },
    LoggedIn,
    LoggedOut,
    Rekey {
        key: u8,
    },
    Connected,
    Disconnect,
    Invalid
}

impl Js5Request {
    pub fn fulfill_request(connection: &mut Connection, request: &Js5Request) -> Result<(), Box<dyn Error>> {
        debug!("Fulfilling request: {:?}", request);

        // Ensure the cache is initialized in this thread before proceeding
        ensure_initialized()?;

        if let Js5Request::Group { urgent, archive, group } = request {
            if *archive == ARCHIVESET && *group == ARCHIVESET as u16 {
                // Handle master index request
                let mut success = false;
                
                MASTER_INDEX_VEC.with(|master_vec_ref| {
                    if let Some(master_index_vec) = &*master_vec_ref.borrow() {
                        let master_index_length = master_index_vec.len();
                        connection.outbound = Packet::new(8 + master_index_length);
                        connection.outbound.p1(ARCHIVESET as i32);
                        connection.outbound.p2(ARCHIVESET as i32);
                        connection.outbound.p1(0);
                        debug!("Master index length: {}", master_index_length);
                        connection.outbound.p4(master_index_length as i32);

                        connection.outbound.pbytes(&master_index_vec, 0, master_index_length);
                        success = true;
                    }
                });

                if !success {
                    return Err("Master index not initialized".into());
                }
            } else {
                // Handle regular file request
                let data = CACHE.with(|cache_ref| -> Result<Vec<u8>, Box<dyn Error>> {
                    let cache_ref = cache_ref.borrow();
                    if let Some(cache) = &*cache_ref {
                        Ok(cache.store.read(*archive, *group as u32)?)
                    } else {
                        Err("Cache not initialized".into())
                    }
                })?;
                
                
                let mut data_packet = Packet::from(data);
                let data_len = data_packet.data.len();
                
                let length = 2 + data_len + (BLOCK_SIZE + data_len) / BYTES_BEFORE_BLOCK + (data_len + BLOCK_SIZE) / BYTES_AFTER_BLOCK + 1;
                
                connection.outbound = Packet::new(length);
                connection.outbound.p1(*archive as i32);
                connection.outbound.p2(*group as i32);

                let compression = data_packet.g1();
                connection.outbound.p1(if *urgent { compression | 0x80 } else { compression } as i32);
                
                let size: usize = (data_packet.g4() + if compression != 0 { 8 } else { 4 }) as usize;

                let mut written = min(size, BYTES_BEFORE_BLOCK);
                connection.outbound.pbytes(&data_packet.data, BLOCK_DELIMITER_SIZE, written);

                while written < size {
                    connection.outbound.p1(0xFF);

                    let chunk_size = min(size - written, BYTES_AFTER_BLOCK);
                    connection.outbound.pbytes(&data_packet.data, (written + BLOCK_DELIMITER_SIZE), chunk_size);
                    written += chunk_size;
                }
            }

            Ok(())
        } else {
            Err("Invalid JS5 request".into())
        }
    }
}