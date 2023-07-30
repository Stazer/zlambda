#![no_std]
#![no_main]
#![feature(step_trait)]

////////////////////////////////////////////////////////////////////////////////////////////////////

use aya_bpf::bindings::xdp_action::{XDP_ABORTED, XDP_PASS, XDP_TX};
use aya_bpf::macros::xdp;
use aya_bpf::programs::XdpContext;
use aya_log_ebpf::{error, info};
use aya_bpf::helpers::{bpf_xdp_adjust_tail, bpf_ktime_get_ns};
use core::hint::unreachable_unchecked;
use core::mem::{size_of, swap};
use core::panic::PanicInfo;
use core::num::TryFromIntError;
use network_types::eth::{EthHdr, EtherType};
use network_types::ip::{IpProto, Ipv4Hdr};
use network_types::udp::UdpHdr;
use zlambda_matrix_ebpf::*;
use zlambda_matrix::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

mod matrix;
pub use matrix::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

fn maybe_optimize<T>(value: T) -> T {
    value
    //black_box(value)
}

////////////////////////////////////////////////////////////////////////////////////////////////////

impl<T> Access<T> for XdpContext {
    fn access(&self, index: usize) -> Option<&T> {
        if maybe_optimize(maybe_optimize(self.data()) + index + size_of::<T>()) > maybe_optimize(self.data_end()) {
            return None;
        }

        Some(unsafe { &*((self.data() + index) as *const T) })
    }
}

impl<T> AccessMut<T> for XdpContext {
    fn access_mut(&mut self, index: usize) -> Option<&mut T> {
        if maybe_optimize(maybe_optimize(self.data()) + index + size_of::<T>()) > maybe_optimize(self.data_end()) {
            return None;
        }

        Some(unsafe { &mut *((self.data() + index) as *mut T) })
    }
}

impl<'a, T> AccessMut<T> for &'a XdpContext {
    fn access_mut(&mut self, index: usize) -> Option<&mut T> {
        if maybe_optimize(maybe_optimize(self.data()) + index + size_of::<T>()) > maybe_optimize(self.data_end()) {
            return None;
        }

        Some(unsafe { &mut *((self.data() + index) as *mut T) })
    }
}

impl<T> Mutate<T> for XdpContext {
    fn mutate(&mut self, index: usize, value: T) {
        if maybe_optimize(maybe_optimize(self.data()) + index + size_of::<T>()) > maybe_optimize(self.data_end()) {
            return
        }

        *unsafe { &mut *((self.data() + index) as *mut T) } = value;
    }
}

impl<'a, T> Mutate<T> for &'a XdpContext {
    fn mutate(&mut self, index: usize, value: T) {
        if maybe_optimize(maybe_optimize(self.data()) + index + size_of::<T>()) > maybe_optimize(self.data_end()) {
            return
        }

        *unsafe { &mut *((self.data() + index) as *mut T) } = value
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum MainError {
    UnexpectedData,
    TryFromIntError(TryFromIntError),
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

#[xdp(name = "main")]
pub fn main(mut context: XdpContext) -> u32 {
    match do_main(&mut context) {
        Err(_error) => XDP_ABORTED,
        Ok(result) => result,
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

fn do_main(context: &mut XdpContext) -> Result<u32, MainError> {
    let program_begin = unsafe { bpf_ktime_get_ns() } as u128;

    if !matches!(
        (<XdpContext as Access<EthHdr>>::access(context, 0).ok_or(MainError::UnexpectedData)?)
            .ether_type,
        EtherType::Ipv4
    ) {
        return Ok(XDP_PASS);
    }

    if !matches!(
        <XdpContext as Access<Ipv4Hdr>>::access(context, EthHdr::LEN)
            .ok_or(MainError::UnexpectedData)?
            .proto,
        IpProto::Udp
    ) {
        return Ok(XDP_PASS);
    }

    if !matches!(
        u16::from_be(
            <XdpContext as Access<UdpHdr>>::access(context, EthHdr::LEN + Ipv4Hdr::LEN)
                .ok_or(MainError::UnexpectedData)?
                .dest,
        ),
        EBPF_UDP_PORT,
    ) {
        return Ok(XDP_PASS);
    }

    let (context, dimension) = {
        let mut reader = AccessReader::new(
            Offset::new(
                context,
                EthHdr::LEN + Ipv4Hdr::LEN + UdpHdr::LEN,
            ),
        );

        let dimension =
            MatrixDimension::read(&mut reader).ok_or(MainError::UnexpectedData)?;

        (reader.into_inner().into_inner(), dimension)
    };

    let matrix_size = dimension.element_count();

    error!(context, "{}", EthHdr::LEN + Ipv4Hdr::LEN + UdpHdr::LEN);

    unsafe {
        bpf_xdp_adjust_tail(context.ctx, MATRIX_SIZE as i32 + 2 * size_of::<u128>() as i32)
    };

    let calculation_elapsed = {
        let context: &XdpContext = context;

        let left = Matrix::<_, MatrixItem>::new(
            Offset::new(
                context,
                EthHdr::LEN + Ipv4Hdr::LEN + UdpHdr::LEN,
            ),
            dimension.flip(),
        );

        let right = Matrix::<_, MatrixItem>::new(
            Offset::new(
                context,
                EthHdr::LEN
                    + Ipv4Hdr::LEN
                    + UdpHdr::LEN
                    + MATRIX_SIZE
            ),
            dimension.clone(),
        );

        let mut result = Matrix::<_, u8>::new(
            Offset::new(
                context,
                EthHdr::LEN
                    + Ipv4Hdr::LEN
                    + UdpHdr::LEN
                    + 2 * MATRIX_SIZE
            ),
            dimension,
        );

        let calculation_begin = unsafe { bpf_ktime_get_ns() } as u128;

        left.multiply(&right, &mut result);

        (unsafe { bpf_ktime_get_ns() } as u128) - calculation_begin
    };

    {
        let udp_header =
            <XdpContext as AccessMut<UdpHdr>>::access_mut(context, EthHdr::LEN + Ipv4Hdr::LEN)
                .ok_or(MainError::UnexpectedData)?;
        swap(&mut udp_header.source, &mut udp_header.dest);
        udp_header.check = 0; // ignore checksum calculation :)
        udp_header.len = u16::from_ne_bytes((u16::from_be_bytes(udp_header.len.to_ne_bytes()) + u16::try_from(matrix_size)?).to_be_bytes());
    }

    {
        let ip_header = <XdpContext as AccessMut<Ipv4Hdr>>::access_mut(context, EthHdr::LEN)
            .ok_or(MainError::UnexpectedData)?;
        swap(&mut ip_header.src_addr, &mut ip_header.dst_addr);
        ip_header.tot_len = u16::from_ne_bytes((u16::from_be_bytes(ip_header.tot_len.to_ne_bytes()) + u16::try_from(matrix_size)?).to_be_bytes());
    }

    {
        let ethernet_header = <XdpContext as AccessMut<EthHdr>>::access_mut(context, 0)
            .ok_or(MainError::UnexpectedData)?;
        swap(&mut ethernet_header.src_addr, &mut ethernet_header.dst_addr);
    }

    {
        let mut times = <XdpContext as AccessMut<u128>>::access_mut(context,
                EthHdr::LEN
                    + Ipv4Hdr::LEN
                    + UdpHdr::LEN
                    + 3 * MATRIX_SIZE
        );

        **times.as_mut().unwrap() = calculation_elapsed;

        let mut times = <XdpContext as AccessMut<u128>>::access_mut(context,
                EthHdr::LEN
                    + Ipv4Hdr::LEN
                    + UdpHdr::LEN
                    + 3 * MATRIX_SIZE
                    + size_of::<u128>(),
        );

        **times.as_mut().unwrap() = (unsafe { bpf_ktime_get_ns() } as u128) - program_begin;
    }

    Ok(XDP_TX)
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { unreachable_unchecked() }
}
