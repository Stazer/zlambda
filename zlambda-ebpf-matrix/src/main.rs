#![no_std]
#![no_main]
#![feature(step_trait)]

////////////////////////////////////////////////////////////////////////////////////////////////////

mod shared;
pub use shared::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

use aya_bpf::bindings::xdp_action::{XDP_ABORTED, XDP_PASS, XDP_TX};
use aya_bpf::macros::xdp;
use aya_bpf::programs::XdpContext;
use aya_log_ebpf::{error, info};
use aya_bpf::helpers::{bpf_xdp_adjust_tail};
use core::hint::unreachable_unchecked;
use core::mem::{size_of, swap};
use core::panic::PanicInfo;
use core::num::TryFromIntError;
use network_types::eth::{EthHdr, EtherType};
use network_types::ip::{IpProto, Ipv4Hdr};
use core::hint::black_box;
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

impl<'a, T> AccessMut<T> for &'a XdpContext {
    fn access_mut(&mut self, index: usize) -> Option<&mut T> {
        if self.data() + index + size_of::<T>() > self.data_end() {
            return None;
        }

        Some(unsafe { &mut *((self.data() + index) as *mut T) })
    }
}

impl<T> Mutate<T> for XdpContext {
    fn mutate(&mut self, index: usize, value: T) {
        if self.data() + index + size_of::<T>() > self.data_end() {
            return
        }

        *unsafe { &mut *((self.data() + index) as *mut T) } = value;
    }
}

/*impl<'a, T> Mutate<T> for &'a XdpContext {
    fn mutate(&mut self, index: usize, value: T) {
        if self.data() + index + size_of::<T>() > self.data_end() {
            error!(*self, "overflow...");
            return
        }

        unsafe { &mut *((self.data() + index) as *mut T) } = value
    }
}*/

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

    let (context, dimension) = {
        let mut reader = AccessReader::new(
            Offset::new(
                context,
                EthHdr::LEN + Ipv4Hdr::LEN + UdpHdr::LEN,
            ),
        );

        let dimension =
            MatrixDimension::<u8>::read(&mut reader).ok_or(MainError::UnexpectedData)?;

        (reader.into_inner().into_inner(), dimension)
    };

    let matrix_size = (dimension.a() * dimension.b()) as usize;

    unsafe {
        bpf_xdp_adjust_tail(context.ctx, matrix_size.try_into()?)
    };

    let s = EthHdr::LEN + Ipv4Hdr::LEN + UdpHdr::LEN + 2 * size_of::<u8>() + 2 * matrix_size;

    let value =
        <XdpContext as AccessMut<u8>>::access_mut(
            context,
            s,
        )
        .ok_or(MainError::UnexpectedData)?;

        *value = 95;

    //let a = black_box(context.data());

    /*if a + s > context.data_end() {
        return Err(MainError::UnexpectedData)
    }*/

    {
        /*let context: &XdpContext = context;

        let left = Matrix::<_, _, u8>::new(
            Offset::new(
                context,
                EthHdr::LEN + Ipv4Hdr::LEN + UdpHdr::LEN + 2 * size_of::<u8>(), /* yep :D */
            ),
            dimension.flip(),
        );

        let right = Matrix::<_, _, u8>::new(
            Offset::new(
                context,
                EthHdr::LEN
                    + Ipv4Hdr::LEN
                    + UdpHdr::LEN
                    + 2 * size_of::<u8>()
                    + dimension.element_count()? * size_of::<u8>(),
            ),
            dimension.clone(),
        );*/

        /*info!(
            context,
            "{} {} {} {}",
            left.get(0, 0).unwrap_or_default().copied().unwrap_or_default(),
            left.get(1, 0).unwrap_or_default().copied().unwrap_or_default(),
            left.get(0, 1).unwrap_or_default().copied().unwrap_or_default(),
            left.get(1, 1).unwrap_or_default().copied().unwrap_or_default(),
        );*/

        /*let mut result = Matrix::<_, _, u8>::new(
            Offset::new(
                &mut context,
                EthHdr::LEN
                    + Ipv4Hdr::LEN
                    + UdpHdr::LEN
                    + 2 * size_of::<u8>()
                    + dimension.element_count()? * size_of::<u8>()
                    //+ dimension.element_count()? * size_of::<u8>(),
            ),
            dimension,
        );*/

        /*let value =
            <XdpContext as AccessMut<u8>>::access_mut(
                context,
                EthHdr::LEN
                    + Ipv4Hdr::LEN
                    + UdpHdr::LEN
                    + 2 * size_of::<u8>()
                    + matrix_size
                    //+ matrix_size
                    //+ dimension.element_count()? * size_of::<u8>()
                    //+ dimension.element_count()? * size_of::<u8>(),
            )
            .ok_or(MainError::UnexpectedData)?;

        *value = 95;*/



        /*context.mutate(
                EthHdr::LEN
                    + Ipv4Hdr::LEN
                    + UdpHdr::LEN,
                    //+ 2 * size_of::<u8>()
                    //+ dimension.element_count()? * size_of::<u8>(),
                    //+ dimension.element_count()? * size_of::<u8>(),
            95u8,
        );*/

        //result.set(0, 0, 95)?;

        /*if let Some(old_value) = result.get_mut(0, 0)? {
            *old_value = 95;
        }*/

        //left.multiply(&right, &mut result)?;
    }

    {
        let udp_header =
            <XdpContext as AccessMut<UdpHdr>>::access_mut(context, EthHdr::LEN + Ipv4Hdr::LEN)
                .ok_or(MainError::UnexpectedData)?;
        swap(&mut udp_header.source, &mut udp_header.dest);
        udp_header.check = 0; // ignore checksum calculation
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

    Ok(XDP_TX)
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { unreachable_unchecked() }
}
