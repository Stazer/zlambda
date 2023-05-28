#![no_std]
#![no_main]
#![feature(unwrap_infallible)]

////////////////////////////////////////////////////////////////////////////////////////////////////

mod shared;
pub use shared::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

use aya_bpf::bindings::xdp_action::{XDP_ABORTED, XDP_PASS, XDP_TX};
use aya_bpf::macros::xdp;
use aya_bpf::programs::XdpContext;
use aya_log_ebpf::{error, info};
use aya_bpf::helpers::{bpf_csum_diff, bpf_xdp_adjust_tail};
use core::hint::unreachable_unchecked;
use core::mem::{size_of, swap};
use core::panic::PanicInfo;
use core::num::TryFromIntError;
use network_types::eth::{EthHdr, EtherType};
use network_types::ip::{IpProto, Ipv4Hdr};
use network_types::udp::UdpHdr;
use zlambda_ebpf::EBPF_UDP_PORT;

////////////////////////////////////////////////////////////////////////////////////////////////////

impl<T> Access<T> for XdpContext {
    fn access(&self, index: usize) -> Option<&T> {
        if self.data() + index + size_of::<T>() > self.data_end() {
            return None;
        }

        Some(unsafe { &*((self.data() + index) as *const T) })
    }
}

impl<T> AccessMut<T> for XdpContext {
    fn access_mut(&mut self, index: usize) -> Option<&mut T> {
        if self.data() + index + size_of::<T>() > self.data_end() {
            return None;
        }

        Some(unsafe { &mut *((self.data() + index) as *mut T) })
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

fn do_main(mut context: &mut XdpContext) -> Result<u32, MainError> {
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

    let (mut context, dimension) = {
        let mut reader = AccessReader::new(
            AccessOffset::new(
                context,
                EthHdr::LEN + Ipv4Hdr::LEN + UdpHdr::LEN,
            ),
        );

        let dimension =
            MatrixDimension::<u8>::read(&mut reader).ok_or(MainError::UnexpectedData)?;

        (reader.into_inner().into_inner(), dimension)
    };


    unsafe {
        bpf_xdp_adjust_tail(context.ctx, (dimension.element_count()? + size_of::<u8>()).try_into()?)
    };

    {
        let context: &XdpContext = context;

        let left = MatrixAccess::<_, _, u8>::new(
            AccessOffset::new(
                context,
                EthHdr::LEN + Ipv4Hdr::LEN + UdpHdr::LEN + 2 * size_of::<u8>(), /* yep :D */
            ),
            dimension.flip(),
        );

        let right = MatrixAccess::<_, _, u8>::new(
            AccessOffset::new(
                context,
                EthHdr::LEN
                    + Ipv4Hdr::LEN
                    + UdpHdr::LEN
                    + 2 * size_of::<u8>()
                    + dimension.element_count()? + size_of::<u8>(),
            ),
            dimension,
        );
    }

    {
        let udp_header =
            <XdpContext as AccessMut<UdpHdr>>::access_mut(context, EthHdr::LEN + Ipv4Hdr::LEN)
                .ok_or(MainError::UnexpectedData)?;
        swap(&mut udp_header.source, &mut udp_header.dest);
        udp_header.check = 0;
    }

    {
        let ip_header = <XdpContext as AccessMut<Ipv4Hdr>>::access_mut(&mut context, EthHdr::LEN)
            .ok_or(MainError::UnexpectedData)?;
        swap(&mut ip_header.src_addr, &mut ip_header.dst_addr);
    }

    {
        let ethernet_header = <XdpContext as AccessMut<EthHdr>>::access_mut(&mut context, 0)
            .ok_or(MainError::UnexpectedData)?;
        swap(&mut ethernet_header.src_addr, &mut ethernet_header.dst_addr);
    }

    Ok(XDP_TX)
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { unreachable_unchecked() }
}
