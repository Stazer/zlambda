#![no_std]
#![no_main]

////////////////////////////////////////////////////////////////////////////////////////////////////

use aya_bpf::bindings::xdp_action::XDP_PASS;
use aya_bpf::macros::xdp;
use aya_bpf::programs::XdpContext;
use aya_log_ebpf::info;
use core::panic::PanicInfo;
use core::hint::unreachable_unchecked;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[xdp(name = "application")]
pub fn application(ctx: XdpContext) -> u32 {
    info!(&ctx, "received a packet");

    XDP_PASS
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { unreachable_unchecked() }
}
