#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(
    clippy::undocumented_unsafe_blocks,
    clippy::missing_const_for_fn,
    clippy::missing_safety_doc,
    clippy::suboptimal_flops,
    unsafe_op_in_unsafe_fn,
    clippy::dbg_macro,
    clippy::use_self,
    missing_docs
)]
#![allow(
    private_bounds,
    clippy::zero_prefixed_literal,
    mixed_script_confusables,
    confusable_idents
)]
use std::mem::ManuallyDrop as MD;
/// # Safety
///
/// this is a transmute. what do you want me to say. the types should, yknow, work? idk.
const unsafe fn transmute_unchecked<T, U>(value: T) -> U {
    // SAFETY: transmutation
    unsafe {
        #[repr(C)]
        union Transmute<T, U> {
            t: MD<T>,
            u: MD<U>,
        }
        MD::into_inner(Transmute { t: MD::new(value) }.u)
    }
}

macro_rules! trt {
    (r $(#[doc = $x: expr])+ f $(#[doc = $z: expr])+ w $(#[doc = $y: expr])+) => {
        use std::io::prelude::*;
        use std::io::Result;
        use std::mem::MaybeUninit as MU;

        $(#[doc = $x])+
        pub trait R: Read {
            $(#[doc = $z])+
            fn r<T: Readable>(&mut self) -> Result<T>;
            /// Reads one byte out of a [`Reader`](Read).
            /// ```
            /// use raad::ne::*;
            /// let mut d = &mut &[1u8, 2][..];
            /// assert_eq!(d.b().unwrap(), 1);
            /// assert_eq!(d.b().unwrap(), 2);
            /// assert!(d.b().is_err());
            /// ```
            fn b(&mut self) -> Result<u8> {
                self.r::<u8>()
            }
        }

        #[doc(hidden)]
        impl<D: Read> R for D {
            fn r<T: Readable>(&mut self) -> Result<T> {
                T::r(self)
            }
        }
        trait Readable
        where
            Self: Sized,
        {
            fn r(from: &mut impl Read) -> Result<Self>;
        }

        impl<const N: usize> Readable for [u8; N] {
            fn r(from: &mut impl Read) -> Result<[u8; N]> {
                let mut buf = [0; N];
                from.read_exact(&mut buf).map(|()| buf)
            }
        }

        $(#[doc = $y])+
        pub trait W: Write {
            /// Writes a type to a [`Writer`](Write)
            fn w<T: Writable>(&mut self, data: T) -> Result<()>;
        }

        #[doc(hidden)]
        impl<D: Write> W for D {
            fn w<T: Writable>(&mut self, data: T) -> Result<()> {
                data._w(self)
            }
        }
        trait Writable {
            fn _w(self, to: &mut impl Write) -> Result<()>;
        }

        impl Writable for &[u8] {
            fn _w(self, to: &mut impl Write) -> Result<()> {
                to.write_all(self)
            }
        }
    };
}

macro_rules! n {
    (writes $bytes:ident $($n:ident)+) => {
        $(
            impl Writable for &[$n] {
                fn _w(self, to: &mut impl Write) -> Result<()> {
                    if (cfg!(target_endian = "little") && stringify!($bytes) == "le") || (cfg!(target_endian = "big") && stringify!($bytes) == "be") {
                        // SAFETY: len correct
                        to.w(unsafe { std::slice::from_raw_parts(self.as_ptr() as *const u8, self.len() * ($n::BITS / 8) as usize) })
                    } else {
                        self.iter().try_for_each(|x| to.w(x))
                    }
                }
            }
            impl<const N: usize> Readable for [$n; N] {
                fn r(from: &mut impl Read) -> Result<[$n; N]> {
                    if (cfg!(target_endian = "little") && stringify!($bytes) == "le") || (cfg!(target_endian = "big") && stringify!($bytes) == "be") {
                        let mut buf = [0; N];
                        // SAFETY: len matches
                        let mut u8s = unsafe { std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, N * ($n::BITS / 8) as usize) };
                        from.read_exact(&mut u8s).map(|()| buf)
                    } else {
                        let mut buf = [MU::<$n>::uninit(); N];
                        for elem in &mut buf{
                            elem.write(from.r::<$n>()?);
                        }
                        // SAFETY: array init
                        Ok(unsafe { crate::transmute_unchecked(buf) })
                    }
                }
            }
        )+
    };
    (float $bytes:ident $([$n:ident <=> $int:ident])+) => {
        $(
        impl Readable for $n {
            fn r(from: &mut impl Read) -> Result<$n> {
                from.r::<$int>().map($n::from_bits)
            }
        }
        )+

        $(
        impl Writable for $n {
            fn _w(self, to: &mut impl Write) -> Result<()> {
                to.w(self.to_bits())
            }
        }
        impl Writable for &$n {
            fn _w(self, to: &mut impl Write) -> Result<()> {
                to.w(self.to_bits())
            }
        }
        macro_rules! bytes {
            ($t:ty) => {
                impl Writable for $t {
                    fn _w(self, to: &mut impl Write) -> Result<()> {
                        to.w(&*self)
                    }
                }
            }
        }
        bytes![Vec<$n>];
        bytes![Box<[$n]>];
        impl<const N: usize> Writable for [$n; N] {
            fn _w(self, to: &mut impl Write) -> Result<()> {
                to.w(&self[..])
            }
        }

        impl Writable for &[$n] {
            fn _w(self, to: &mut impl Write) -> Result<()> {
                if (cfg!(target_endian = "little") && stringify!($bytes) == "le") || (cfg!(target_endian = "big") && stringify!($bytes) == "be") {
                    // SAFETY: len correct
                    to.w(unsafe { std::slice::from_raw_parts(self.as_ptr() as *const u8, self.len() * ($int::BITS / 8) as usize) })
                } else {
                    self.iter().try_for_each(|x| to.w(x))
                }
            }
        }
        impl<const N: usize> Readable for [$n; N] {
            fn r(from: &mut impl Read) -> Result<[$n; N]> {
                if (cfg!(target_endian = "little") && stringify!($bytes) == "le") || (cfg!(target_endian = "big") && stringify!($bytes) == "be") {
                    let mut buf = [0.; N];
                    // SAFETY: len matches
                    let mut u8s = unsafe { std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, N * ($int::BITS / 8) as usize) };
                    from.read_exact(&mut u8s).map(|()| buf)
                } else {
                    let mut buf = [MU::<$n>::uninit(); N];
                    for elem in &mut buf{
                        elem.write(from.r::<$n>()?);
                    }
                    // SAFETY: array init
                    Ok(unsafe { crate::transmute_unchecked(buf) })
                }
            }
        }
        )+
    };
    ($bytes:ident $($n:ident)+) => {
        $(
        impl Readable for $n {
            fn r(from: &mut impl Read) -> Result<$n> {
                from.r::<[u8; { std::mem::size_of::<$n>() }]>().map($n::from_ne_bytes).map($n::$bytes)
            }
        }
        )+

        $(
        impl Writable for $n {
            fn _w(self, to: &mut impl Write) -> Result<()> {
                to.w(self.$bytes().to_ne_bytes())
            }
        }
        impl Writable for &$n {
            fn _w(self, to: &mut impl Write) -> Result<()> {
                to.w(self.$bytes().to_ne_bytes())
            }
        }
        macro_rules! bytes {
            ($t:ty) => {
                impl Writable for $t {
                    fn _w(self, to: &mut impl Write) -> Result<()> {
                        to.w(&*self)
                    }
                }
            }
        }
        bytes![Vec<$n>];
        bytes![Box<[$n]>];
        impl<const N: usize> Writable for [$n; N] {
            fn _w(self, to: &mut impl Write) -> Result<()> {
                to.w(&self[..])
            }
        }
        )+
    };
}

macro_rules! test {
    () => {
        #[test]
        fn x() {
            let data = &mut &[0x12u8, 0x15][..];
            let mut out = vec![];
            out.w(data.r::<[u16; 1]>().unwrap()).unwrap();
            assert_eq!(out, [0x12, 0x15]);

            let mut out = vec![];
            out.w([12.0_f32, 13.]).unwrap();
            assert_eq!((&mut &*out).r::<[f32; 2]>().unwrap(), [12., 13.]);
        }
    };
}
pub mod le {
    //! little endian readers and writers
    trt!(
        r /// Read little endian (commonly native) data.
          /// This trait provides a [`r`](R::r) method for easy reading.
          ///
          /// Without this crate, you would have to do things such as:
          /// ```
          /// use std::io::Read;
          /// let mut data = &mut &[0xff, 0xf1][..];
          /// let mut two_bytes = [0; 2];
          /// data.read(&mut two_bytes).unwrap();
          /// assert_eq!(u16::from_le_bytes(two_bytes), 0xf1ff)
          /// ```
          /// Now, you can simply:
          /// ```
          /// use raad::le::*;
          /// let mut data = &mut &[0xff, 0xf1][..];
          /// assert_eq!(data.r::<u16>().unwrap(), 0xf1ff);
          /// ```
        f /// Read a little endian type.
          /// ```
          /// # #![allow(overflowing_literals)]
          /// use raad::le::*;
          /// let mut data = &mut &[0xc1, 0x00, 0x7c, 0xff][..];
          /// assert_eq!(data.r::<[i16; 2]>().unwrap(), [0x00c1, 0xff7c]);
          /// ```
        w /// Write little endian (commonly native) data.
          /// ```
          /// # use raad::le::*;
          /// let mut wtr = Vec::new();
          /// wtr.w::<[u32; 2]>([267, 1205419366]).unwrap();
          /// assert_eq!(wtr, [11, 1, 0, 0, 102, 61, 217, 71]);
          /// ```
    );
    n![writes le u16 u32 u64 u128 i8 i16 i32 i64 i128];
    n![float le [f32 <=> u32] [f64 <=> u64]];
    n![to_le u8 u16 u32 u64 u128 i8 i16 i32 i64 i128];
    test![];
}

#[doc(alias = "network")]
pub mod be {
    //! big endian readers and writers
    trt!(
        r /// Read big endian (network) data.
          /// ```
          /// use raad::be::*;
          /// // this example doesnt actually care about endianness-- u8's dont have any.
          /// let mut data = &mut &[2u8, 5, 1, 4, 3][..];
          /// assert_eq!(data.r::<[u8; 5]>().unwrap(), [2, 5, 1, 4, 3]);
          /// ```
        f /// Read a big endian (network) type.
          /// ```
          /// use raad::be::*;
          /// let mut data: &mut &[u8] = &mut &[
          ///     0x00, 0x03, 0x43, 0x95, 0x4d, 0x60, 0x86, 0x83,
          ///     0x00, 0x03, 0x43, 0x95, 0x4d, 0x60, 0x86, 0x83
          /// ][..];
          /// assert_eq!(
          ///     data.r::<u128>().unwrap(),
          ///     16947640962301618749969007319746179
          /// );
          /// ```
        w /// Write a big endian (network) type.
          /// ```
          /// use raad::be::*;
          /// let mut wtr = Vec::new();
          /// wtr.w::<[u16; 2]>([517, 768]).unwrap();
          /// assert_eq!(wtr, b"\x02\x05\x03\x00");
          /// ```
    );
    n![writes be u16 u32 u64 u128 i8 i16 i32 i64 i128];
    n![float be [f32 <=> u32] [f64 <=> u64]];
    n![to_be u8 u16 u32 u64 u128 i8 i16 i32 i64 i128];
    test![];
}

pub mod ne {
    //! native endian readers and writers
    #[cfg(target_endian = "big")]
    #[doc(inline)]
    pub use super::be::{R, W};
    #[cfg(target_endian = "little")]
    #[doc(inline)]
    pub use super::le::{R, W};
}
