#![no_std]
#![no_main]
#![feature(step_trait)]

////////////////////////////////////////////////////////////////////////////////////////////////////

use aya_bpf::bindings::xdp_action::{XDP_ABORTED, XDP_PASS, XDP_TX, XDP_REDIRECT};
use aya_bpf::macros::xdp;
use aya_bpf::programs::XdpContext;
use aya_log_ebpf::{error, info};
use aya_bpf::helpers::{bpf_xdp_adjust_tail, bpf_ktime_get_ns, bpf_xdp_get_buff_len};
use core::hint::{black_box, unreachable_unchecked};
use core::mem::{size_of, swap};
use core::panic::PanicInfo;
use core::num::TryFromIntError;
use core::slice::{from_raw_parts_mut, from_raw_parts};
use network_types::eth::{EthHdr, EtherType};
use network_types::ip::{IpProto, Ipv4Hdr};
use network_types::udp::UdpHdr;
use zlambda_matrix_ebpf::*;
use zlambda_matrix::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[inline(always)]
fn ptr_at<T>(ctx: &XdpContext, offset: usize) -> *const T {
    (ctx.data() + offset) as *const T
}

#[inline(always)]
fn ptr_mut_at<T>(ctx: &XdpContext, offset: usize) -> *mut T {
    (ctx.data() + offset) as *mut T
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum MainError {
    UnexpectedData,
    TryFromIntError(TryFromIntError),
}

impl From<()> for MainError {
    fn from(_error: ()) -> Self {
        Self::UnexpectedData
    }
}

impl From<MainError> for &str {
    fn from(error: MainError) -> Self {
        match error {
            MainError::UnexpectedData => "Unexpected data",
            MainError::TryFromIntError(_error) => "try from int error",
        }
    }
}

impl From<TryFromIntError> for MainError {
    fn from(error: TryFromIntError) -> Self {
        Self::TryFromIntError(error)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[xdp]
pub fn xdp_main(context: XdpContext) -> u32 {
    unsafe { xdp_main_unsafe(context) }
}

unsafe fn xdp_main_unsafe(context: XdpContext) -> u32 {
    let program_begin = bpf_ktime_get_ns() as u128;

    if context.data() + EthHdr::LEN + Ipv4Hdr::LEN + UdpHdr::LEN > context.data_end() {
        return XDP_PASS;
    }

    let ethhdr = ptr_at::<EthHdr>(&context, 0);

    if !matches!((*ethhdr).ether_type, EtherType::Ipv4) {
        return XDP_PASS;
    }

    let ipv4hdr = ptr_at::<Ipv4Hdr>(&context, EthHdr::LEN);

    if !matches!((*ipv4hdr).proto, IpProto::Udp) {
        return XDP_PASS;
    }

    let udphdr = ptr_at::<UdpHdr>(&context, EthHdr::LEN + Ipv4Hdr::LEN);

    if !matches!(u16::from_be((*udphdr).dest), EBPF_UDP_PORT) {
        return XDP_PASS;
    }

    if context.data() + EthHdr::LEN + Ipv4Hdr::LEN + UdpHdr::LEN + 3 * MATRIX_SIZE + 2 * size_of::<u128>() > context.data_end() {
        return XDP_PASS;
    }

    let calculation_begin = bpf_ktime_get_ns() as u128;

    for i in 0..MATRIX_DIMENSION_SIZE {
        for j in 0..MATRIX_DIMENSION_SIZE {
            let mut value = 0;

            for k in 0..MATRIX_DIMENSION_SIZE {
                if context.data() + EthHdr::LEN + Ipv4Hdr::LEN + UdpHdr::LEN + 3 * MATRIX_SIZE + 2 * size_of::<u128>() > context.data_end() {
                    return XDP_PASS;
                }

                let left_value = *ptr_at::<u8>(
                    &context,
                    EthHdr::LEN
                        + Ipv4Hdr::LEN
                        + UdpHdr::LEN
                        + (i * MATRIX_DIMENSION_SIZE + k)
                );

                if context.data() + EthHdr::LEN + Ipv4Hdr::LEN + UdpHdr::LEN + 3 * MATRIX_SIZE + 2 * size_of::<u128>() > context.data_end() {
                    return XDP_PASS;
                }

                let right_value = *ptr_at::<u8>(
                    &context,
                    EthHdr::LEN
                        + Ipv4Hdr::LEN
                        + UdpHdr::LEN
                        + MATRIX_SIZE
                        + (k * MATRIX_DIMENSION_SIZE + j)
                );

                value += left_value * right_value;
            }

            if context.data() + EthHdr::LEN + Ipv4Hdr::LEN + UdpHdr::LEN + 3 * MATRIX_SIZE + 2 * size_of::<u128>() > context.data_end() {
                return XDP_PASS;
            }

            let result_value = ptr_mut_at::<u8>(
                &context,
                EthHdr::LEN
                    + Ipv4Hdr::LEN
                    + UdpHdr::LEN
                    + 2 * MATRIX_SIZE
                    + (i * MATRIX_DIMENSION_SIZE + j)
            );

            *result_value = value;
        }
    }

    let calculation_end = bpf_ktime_get_ns() as u128;

    if context.data() + EthHdr::LEN + Ipv4Hdr::LEN + UdpHdr::LEN + 3 * MATRIX_SIZE + 2 * size_of::<u128>() > context.data_end() {
        return XDP_PASS;
    }

    {
        let udp_header = ptr_mut_at::<UdpHdr>(
            &context,
            EthHdr::LEN + Ipv4Hdr::LEN,
        );

        swap(&mut (*udp_header).source, &mut (*udp_header).dest);
        (*udp_header).check = 0; // ignore checksum calculation :)
    }

    if context.data() + EthHdr::LEN + Ipv4Hdr::LEN + UdpHdr::LEN + 3 * MATRIX_SIZE + 2 * size_of::<u128>() > context.data_end() {
        return XDP_PASS;
    }

    {
        let ip_header = ptr_mut_at::<Ipv4Hdr>(
            &context,
            EthHdr::LEN,
        );

        swap(&mut (*ip_header).src_addr, &mut (*ip_header).dst_addr);
    }

    if context.data() + EthHdr::LEN + Ipv4Hdr::LEN + UdpHdr::LEN + 3 * MATRIX_SIZE + 2 * size_of::<u128>() > context.data_end() {
        return XDP_PASS;
    }

    {
        let ethernet_header = ptr_mut_at::<EthHdr>(
            &context,
            0,
        );

        swap(&mut (*ethernet_header).src_addr, &mut (*ethernet_header).dst_addr);
    }

    if context.data() + EthHdr::LEN + Ipv4Hdr::LEN + UdpHdr::LEN + 3 * MATRIX_SIZE + 2 * size_of::<u128>() > context.data_end() {
        return XDP_PASS;
    }

    {
        let calculation = ptr_mut_at::<u128>(
            &context,
            EthHdr::LEN
                + Ipv4Hdr::LEN
                + UdpHdr::LEN
                + 3 * MATRIX_SIZE
        );

        *calculation = calculation_end - calculation_begin;
    }

    if context.data() + EthHdr::LEN + Ipv4Hdr::LEN + UdpHdr::LEN + 3 * MATRIX_SIZE + 2 * size_of::<u128>() > context.data_end() {
        return XDP_PASS;
    }

    {
        let program = ptr_mut_at::<u128>(
            &context,
            EthHdr::LEN
                + Ipv4Hdr::LEN
                + UdpHdr::LEN
                + 3 * MATRIX_SIZE
                + size_of::<u128>(),
        );

        *program = bpf_ktime_get_ns() as u128 - program_begin;
    }

    XDP_TX
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { unreachable_unchecked() }
}
