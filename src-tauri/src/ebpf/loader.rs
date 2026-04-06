use aya::maps::perf::AsyncPerfEventArray;
use aya::programs::kprobe::KProbe;
use aya::Ebpf;
use bytes::BytesMut;
use std::net::Ipv4Addr;
use tokio::sync::mpsc;

use crate::debug;
use crate::state::DnsEvent;
use dnsflow_common as common;

pub struct EbpfLoader {
    bpf: Option<Ebpf>,
    pub is_loaded: bool,
    pub interface: Option<String>,
    pub event_receiver: Option<mpsc::Receiver<DnsEvent>>,
}

impl std::fmt::Debug for EbpfLoader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EbpfLoader")
            .field("is_loaded", &self.is_loaded)
            .field("interface", &self.interface)
            .field("bpf", &self.bpf.as_ref().map(|_| "<BPF instance>"))
            .finish()
    }
}

impl EbpfLoader {
    pub fn new() -> Self {
        Self {
            bpf: None,
            is_loaded: false,
            interface: None,
            event_receiver: None,
        }
    }

    pub async fn load_and_attach(
        &mut self,
        event_tx: mpsc::Sender<DnsEvent>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let bpf_path = "target/bpfel-unknown-none/debug/dnsflow-ebpf";

        let mut bpf = match Ebpf::load_file(bpf_path) {
            Ok(bpf) => bpf,
            Err(e) => {
                debug!(
                    "Failed to load BPF object from '{}': {}. \
                     Ensure the eBPF program is compiled (cargo build --target bpfel-unknown-none). \
                     Root/CAP_BPF permissions may be required.",
                    bpf_path, e
                );
                return Err(e.into());
            }
        };

        let program: &mut KProbe = bpf
            .program_mut("dnsflow_kprobe")
            .ok_or("BPF program 'dnsflow_kprobe' not found in object")?
            .try_into()?;
        program.load()?;
        program.attach("udp_sendmsg", 0)?;

        let mut events: AsyncPerfEventArray<_> = bpf
            .take_map("EVENTS")
            .ok_or("BPF map 'EVENTS' not found in object")?
            .try_into()?;

        let mut buf = events.open(0, None)?;

        tokio::spawn(async move {
            let mut buffers: Vec<BytesMut> =
                (0..4).map(|_| BytesMut::with_capacity(1024)).collect();

            loop {
                let event_data = match buf.read_events(&mut buffers).await {
                    Ok(data) => data,
                    Err(e) => {
                        debug!("Error reading BPF events: {}", e);
                        continue;
                    }
                };

                for i in 0..event_data.read {
                    let buf = &buffers[i];

                    if buf.len() < std::mem::size_of::<common::DnsEvent>() {
                        debug!(
                            "Warning: Received undersized event buffer ({} bytes)",
                            buf.len()
                        );
                        continue;
                    }

                    let ptr = buf.as_ptr() as *const common::DnsEvent;
                    let raw = unsafe { ptr.read_unaligned() };

                    let event = DnsEvent {
                        pid: raw.pid as i64,
                        tgid: raw.tgid as i64,
                        uid: raw.uid as i64,
                        gid: raw.gid as i64,
                        comm: String::from_utf8_lossy(&raw.comm)
                            .trim_end_matches('\0')
                            .to_string(),
                        daddr: Ipv4Addr::new(
                            raw.daddr[0],
                            raw.daddr[1],
                            raw.daddr[2],
                            raw.daddr[3],
                        )
                        .to_string(),
                        dport: raw.dport as i64,
                        is_dns: raw.is_dns,
                    };

                    if event_tx.send(event).await.is_err() {
                        debug!("Event receiver dropped, stopping BPF event consumer");
                        return;
                    }
                }
            }
        });

        self.bpf = Some(bpf);
        self.is_loaded = true;

        Ok(())
    }

    pub async fn load(&mut self) -> Result<(), String> {
        let (tx, rx) = mpsc::channel(100);
        self.event_receiver = Some(rx);
        self.load_and_attach(tx).await.map_err(|e| e.to_string())
    }

    pub fn unload(&mut self) {
        self.bpf = None;
        self.is_loaded = false;
        self.interface = None;
        self.event_receiver = None;
    }

    pub async fn unload_async(&mut self) -> Result<(), String> {
        self.unload();
        Ok(())
    }

    pub async fn attach(&mut self, interface: &str) -> Result<(), String> {
        self.interface = Some(interface.to_string());
        Ok(())
    }

    pub fn has_bpf(&self) -> bool {
        self.bpf.is_some()
    }

    pub fn bpf_mut(&mut self) -> Option<&mut Ebpf> {
        self.bpf.as_mut()
    }
}

impl Default for EbpfLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for EbpfLoader {
    fn drop(&mut self) {
        self.unload();
    }
}
