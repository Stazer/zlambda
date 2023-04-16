#![no_std]
#![no_main]

////////////////////////////////////////////////////////////////////////////////////////////////////

use aya_bpf::bindings::xdp_action::XDP_PASS;
use aya_bpf::macros::xdp;
use aya_bpf::programs::XdpContext;
use aya_log_ebpf::info;
use core::panic::PanicInfo;
use core::hint::unreachable_unchecked;
use serde::Deserialize;
use network_types::eth::{EthHdr, EtherType};
use core::mem::size_of;
use network_types::ip::{Ipv4Hdr, IpProto};
use network_types::tcp::TcpHdr;
use network_types::udp::UdpHdr;

////////////////////////////////////////////////////////////////////////////////////////////////////

const MATRIX_ROWS: usize = 16;
const MATRIX_COLUMNS: usize = 16;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize)]
struct Matrix<'a> {
    rows: u64,
    columns: u64,
    data: &'a [u8],
}

#[inline(always)]
unsafe fn ptr_at<T>(ctx: &XdpContext, offset: usize) -> Result<*const T, ()> {
    let start = ctx.data();
    let end = ctx.data_end();
    let len = size_of::<T>();

    if start + offset + len > end {
        return Err(());
    }

    Ok((start + offset) as *const T)
}


////////////////////////////////////////////////////////////////////////////////////////////////////

const SIZE: usize = 1024 * 1024 * 1024;

static a: [u8; 512] = [0; 512];

#[xdp(name = "main")]
pub fn main(ctx: XdpContext) -> u32 {
    do_main(ctx);

    XDP_PASS

    /*let mut hallo = heapless::Vec::<usize, 512>::default();

    /*for i in 0..511 {
        unsafe {
            a[i] = 5;
        }
    }*/

    info!(&ctx, "Yes");

    XDP_PASS*/
}

fn do_main(ctx: XdpContext) -> Result<u32, ()> {
    let ethhdr: *const EthHdr = unsafe { ptr_at(&ctx, 0)? };
    match unsafe { *ethhdr }.ether_type {
        EtherType::Ipv4 => {}
        _ => return Ok(XDP_PASS),
    }

    let ipv4hdr: *const Ipv4Hdr = unsafe { ptr_at(&ctx, EthHdr::LEN)? };
    let source_addr = u32::from_be(unsafe { *ipv4hdr }.src_addr);

    let udp_port = match unsafe { *ipv4hdr }.proto {
        IpProto::Udp => {
            let udphdr: *const UdpHdr =
                unsafe { ptr_at(&ctx, EthHdr::LEN + Ipv4Hdr::LEN) }?;
            u16::from_be(unsafe { *udphdr }.source)
        }
        _ => return Ok(XDP_PASS),
    };

    /*let source_port = match unsafe { *ipv4hdr }.proto {
        IpProto::Tcp => {
            let tcphdr: *const TcpHdr =
                unsafe { ptr_at(&ctx, EthHdr::LEN + Ipv4Hdr::LEN) }?;
            u16::from_be(unsafe { *tcphdr }.source)
        }
        IpProto::Udp => {
            let udphdr: *const UdpHdr =
                unsafe { ptr_at(&ctx, EthHdr::LEN + Ipv4Hdr::LEN) }?;
            u16::from_be(unsafe { *udphdr }.source)
        }
        _ => return Ok(XDP_PASS),
    };

    info!(&ctx, "SRC IP: {}, SRC PORT: {}", source_addr, source_port);*/

    info!(&ctx, "FROM PORT {}", udp_port);

    Ok(XDP_PASS)
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { unreachable_unchecked() }
}
