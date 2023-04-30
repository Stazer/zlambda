#![no_std]
#![no_main]

////////////////////////////////////////////////////////////////////////////////////////////////////

use aya_bpf::bindings::xdp_action::XDP_PASS;
use aya_bpf::macros::xdp;
use aya_bpf::programs::XdpContext;
//use aya_log_ebpf::info;
use core::panic::PanicInfo;
use core::hint::unreachable_unchecked;
use network_types::eth::{EthHdr, EtherType};
use core::mem::size_of;
use network_types::ip::{Ipv4Hdr, IpProto};
use network_types::udp::UdpHdr;

////////////////////////////////////////////////////////////////////////////////////////////////////

/*#[derive(Deserialize)]
struct Matrix<'a> {
    rows: u64,
    columns: u64,
    data: &'a [u8],
}*/

/*#[inline(always)]
unsafe fn ptr_at<T>(ctx: &XdpContext, offset: usize) -> Result<*const T, ()> {
    let start = ctx.data();
    let end = ctx.data_end();
    let len = size_of::<T>();

    if start + offset + len > end {
        return Err(());
    }

    Ok((start + offset) as *const T)
}*/

////////////////////////////////////////////////////////////////////////////////////////////////////

//const SIZE: usize = 1024 * 1024 * 1024;

//static a: [u8; 512] = [0; 512];

#[xdp(name = "main")]
pub fn main(ctx: XdpContext) -> u32 {
    //info!(&ctx, "Hello World");

    //do_main(ctx);

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

/*fn do_main(ctx: XdpContext) -> Result<u32, ()> {
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
}*/

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { unreachable_unchecked() }
}
/*

use serde::{Deserialize};
use zerovec::ZeroVec;
use zerovec::ule::{AsULE};
use core::ptr::{write_bytes, copy_nonoverlapping};
use serde_json_core::from_slice;
//use std::str::from_utf8;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type MatrixValues<'a, T> = ZeroVec<'a, T>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Debug)]
#[serde(bound = "T: zerovec::ule::AsULE + Deserialize<'de> + 'de")]
pub struct MatrixInput<'a, T>
where
    T: AsULE,
{
    dimension_a: usize,
    dimension_b: usize,
    #[serde(borrow)]
    left: MatrixValues<'a, T>,
    #[serde(borrow)]
    right: MatrixValues<'a, T>,
}

impl<'a, T> MatrixInput<'a, T>
where
    T: AsULE,
{
    #[inline]
    pub fn left<'b>(&'b self) -> MatrixValuesView<'a, 'b, T> {
        MatrixValuesView {
            columns: self.dimension_b,
            values: &self.left,
        }
    }

    #[inline]
    pub fn right<'b>(&'b self) -> MatrixValuesView<'a, 'b, T> {
        MatrixValuesView {
            columns: self.dimension_a,
            values: &self.right,
        }
    }

    #[inline]
    pub fn dimension_a(&self) -> usize {
        self.dimension_a
    }

    #[inline]
    pub fn dimension_b(&self) -> usize {
        self.dimension_b
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct MatrixValuesView<'a, 'b, T>
where
    T: AsULE
{
    columns: usize,
    values: &'a MatrixValues<'b, T>,
}

impl<'a, 'b, T> MatrixValuesView<'a, 'b, T>
where
    T: AsULE,
{
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Option<T> {
        self.values.get(y * self.columns + x)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

use core::fmt::Write;


struct Writer {
    handle: *mut u8,
    written: usize,
}

impl Write for Writer {
    fn write_str(&mut self, string: &str) -> core::fmt::Result {
        for byte in string.bytes() {
            unsafe {
                *self.handle = byte;
                self.handle = self.handle.offset(1);
                self.written += 1;
            }
        }

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/*fn main() {
    go().unwrap();
}*/

fn go() -> Result<(), ()> {

    Ok(())
}

fn calculate(input: &str) {
    //let input = b"{\"dimension_a\":2,\"dimension_b\":2,\"left\":[0,1,2,3],\"right\":[4,5,6,7]}";
    let mut buffer: [u8; 65000] = [b' '; 65000];
    unsafe {
        copy_nonoverlapping(input.as_ptr(), buffer.as_mut_ptr(), input.len());
    }
    let buffer = buffer;

    let input_buffer = &buffer;
    let (matrix_input, _read) = from_slice::<MatrixInput<i32>>(input_buffer).unwrap();
    unsafe {
        write_bytes(buffer.as_ptr().offset(input.len().try_into().map_err(|_error| ())?) as *mut u8, 0, buffer.len() - input.len());
    }

    let start = unsafe {
        input_buffer.as_ptr().offset(input.len().try_into().map_err(|_error| ())?) as *mut u8
    };

    let mut writer = Writer {
        handle: start,
        written: 0,
    };

    write!(writer, "[").map_err(|_error| ())?;
    for i in 0..matrix_input.dimension_a() {
        for j in 0..matrix_input.dimension_b() {
            let mut value = 0;

            for k in 0..matrix_input.dimension_a() {
                value += matrix_input.left().get(k, i).unwrap() * matrix_input.right().get(j, k).unwrap();
            }

            if i == matrix_input.dimension_a() - 1 && j == matrix_input.dimension_b() - 1 {
                write!(writer, "{}", value).map_err(|_error| ())?;
            } else {
                write!(writer, "{},", value).map_err(|_error| ())?;
            }
        }
    }

    write!(writer, "]").map_err(|_error| ())?;

    unsafe {
        copy_nonoverlapping(start, buffer.as_ptr() as *mut u8, writer.written);
        write_bytes(buffer.as_ptr().offset(writer.written.try_into().map_err(|_error| ())?) as *mut u8, 0, buffer.len() - writer.written);
    }

    //format!("{}", from_utf8(&buffer).);
}*/
