#![no_std]
#![no_main]

use aya_bpf::{bindings::xdp_action, macros::xdp, programs::XdpContext};
use aya_bpf::maps::HashMap;
use aya_bpf::macros::map;
use aya_log_ebpf::info;

#[map(name = "BACKEND_PORTS")]
static mut BACKEND_PORTS: HashMap<u16, u16> =
    HashMap::<u16, u16>::with_max_entries(10, 0);

#[xdp(name = "myapp")]
pub fn myapp(ctx: XdpContext) -> u32 {
    match try_myapp(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

fn try_myapp(ctx: XdpContext) -> Result<u32, u32> {
    info!(&ctx, "received a packet");
    Ok(xdp_action::XDP_PASS)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
