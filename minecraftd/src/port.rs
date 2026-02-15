use std::{
    collections::HashSet,
    sync::{Mutex, MutexGuard, OnceLock},
};

use anyhow::bail;

static PORT_POOL: OnceLock<Mutex<PortPool>> = OnceLock::new();

#[derive(Debug)]
struct PortPool {
    start: u16,
    end: u16,
    used_ports: HashSet<u16>,
}

#[derive(Debug, Clone)]
pub struct Port {
    port: u16,
}

pub fn init_port_pool(start: u16, end: u16) {
    let pool = PortPool {
        start,
        end,
        used_ports: HashSet::new(),
    };
    PORT_POOL
        .set(Mutex::new(pool))
        .expect("PortPool already initialized");
}

fn get_instance() -> MutexGuard<'static, PortPool> {
    let pool = PORT_POOL.get().expect("PortPool not initialized");
    pool.lock().unwrap()
}

impl Port {
    pub fn acquire() -> anyhow::Result<Self> {
        let mut pool = get_instance();
        for port in pool.start..=pool.end {
            if !pool.used_ports.contains(&port) {
                pool.used_ports.insert(port);
                return Ok(Port { port });
            }
        }
        bail!(
            "No available ports. Try increasing the port range using --port-min and --port-max options."
        )
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Drop for Port {
    fn drop(&mut self) {
        let mut pool = get_instance();
        pool.used_ports.remove(&self.port);
    }
}
