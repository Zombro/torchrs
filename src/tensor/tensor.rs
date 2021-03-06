#![allow(unused_variables)]
use rutorch::*;
use std::ops::{Index, IndexMut};
use std::convert::From;
use std::cmp::max;
use std::fmt;
use std::os::raw::c_void;
use std::hash::{Hash, Hasher};
use serde::{Serialize, Deserialize, Serializer, Deserializer};

use storage::*;
use ::*;
pub use tensor::tensor_ops::*;
use RcMut;

#[derive(Clone, Debug)]
pub struct THVec<T: NumLimits> {
    pub data: Vec<T>,
    pub dims: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct THVecGeneric {
    pub data: Vec<i64>,
    pub dims: Vec<usize>,
}
#[derive(Clone, Debug)]
pub struct THDims {
    pub dims: Vec<usize>,
}

pub enum TensorType {
    Float,
    Double,
    Byte,
    Char,
    Short,
    Int,
    Long,
}

pub trait NumLimits
    : Copy + Default + fmt::Debug + ::num::Num + ::num::NumCast + serde::Serialize
    {
}
impl NumLimits for f32 {}
impl NumLimits for f64 {}
impl NumLimits for i8 {}
impl NumLimits for u8 {}
impl NumLimits for i16 {}
impl NumLimits for u16 {}
impl NumLimits for i32 {}
impl NumLimits for u32 {}
impl NumLimits for i64 {}

#[derive(Hash, Serialize, Deserialize, Debug)]
pub enum TensorKind {
    FloatTensor(Tensor<f32>),
    LongTensor(Tensor<i64>),
    ByteTensor(Tensor<u8>),
}

pub type TensorList<T> = Vec<Tensor<T>>;
pub type TensorKindList = Vec<TensorKind>;
pub type OptTensorKindList = Vec<Option<TensorKind>>;
pub type RefTensorList<'a, T> = Vec<&'a mut Tensor<T>>;
pub type RefTensorKindList<'a> = Vec<&'a TensorKind>;
pub type TensorId = usize;

impl TensorKind {
    pub fn new<S>(&self, args: S) -> Self
        where S: Into<THVecGeneric>
    {
        match *self {
            TensorKind::FloatTensor(ref t) => {
                let tv: THVecGeneric = args.into();
                let tv: THVec<f32> = tv.into();
                let mut newt: Tensor<f32> = t.new(tv.dims);
                if tv.data.len() != 0 {
                    newt.set_storage(tv.data);
                }
                newt.into()
            }
            TensorKind::LongTensor(ref t) => {
                let tv: THVecGeneric = args.into();
                let tv: THVec<i64> = tv.into();
                let mut newt: Tensor<i64> = t.new(tv.dims);
                if tv.data.len() != 0 {
                    newt.set_storage(tv.data);
                }
                newt.into()
            }
            TensorKind::ByteTensor(ref t) => {
                let tv: THVecGeneric = args.into();
                let tv: THVec<u8> = tv.into();
                let mut newt: Tensor<u8> = t.new(tv.dims);
                if tv.data.len() != 0 {
                    newt.set_storage(tv.data);
                }
                newt.into()
            }
        }
    }

    pub fn backend(&self) -> Box<nn::BackendIntf> {
        match *self {
            /* XXX avoid repeated boxing on every call */
            TensorKind::FloatTensor(_) => Box::new(nn::FloatBackend.clone()),
            _ => panic!("no corresponding backend"),
        }
    }
    pub fn s<D>(&self, dim: D) -> Self
        where D: AsRef<[isize]>
    {
        unimplemented!()
    }
}

impl PartialEq for TensorKind {
    fn eq(&self, other: &Self) -> bool {
        use self::TensorKind::{FloatTensor, LongTensor, ByteTensor};
        match (self, other) {
            (&FloatTensor(ref t1), &FloatTensor(ref t2)) => t1.id() == t2.id(),
            (&LongTensor(ref t1), &LongTensor(ref t2)) => t1.id() == t2.id(),
            (&ByteTensor(ref t1), &ByteTensor(ref t2)) => t1.id() == t2.id(),
            _ => false,
        }
    }
}
impl Eq for TensorKind {}
impl Clone for TensorKind {
    fn clone(&self) -> Self {
        use self::TensorKind::{FloatTensor, LongTensor, ByteTensor};
        match *self {
            FloatTensor(ref t) => FloatTensor(t.clone()),
            LongTensor(ref t) => LongTensor(t.clone()),
            ByteTensor(ref t) => ByteTensor(t.clone()),
        }
    }
}

impl<T: NumLimits> From<Tensor<T>> for TensorKind {
    #[allow(unused_variables)]
    default fn from(input: Tensor<T>) -> Self {
        unreachable!()
    }
}

impl From<Tensor<f32>> for TensorKind {
    fn from(input: Tensor<f32>) -> Self {
        TensorKind::FloatTensor(input)
    }
}
impl From<Tensor<u8>> for TensorKind {
    fn from(input: Tensor<u8>) -> Self {
        TensorKind::ByteTensor(input)
    }
}

impl From<Tensor<i64>> for TensorKind {
    fn from(input: Tensor<i64>) -> Self {
        TensorKind::LongTensor(input)
    }
}

impl<T: NumLimits> From<TensorKind> for Tensor<T> {
    #[allow(unused_variables)]
    default fn from(input: TensorKind) -> Self {
        panic!("bad cast")
    }
}

impl<'a, T: NumLimits> From<&'a TensorKind> for &'a Tensor<T> {
    #[allow(unused_variables)]
    default fn from(input: &'a TensorKind) -> Self {
        panic!("bad cast")
    }
}

impl<'a> From<&'a TensorKind> for &'a Tensor<f32> {
    fn from(input: &'a TensorKind) -> Self {
        match *input {
            TensorKind::FloatTensor(ref t) => t,
            _ => unreachable!(),
        }
    }
}

impl<'a> From<&'a TensorKind> for &'a Tensor<i64> {
    fn from(input: &'a TensorKind) -> Self {
        match *input {
            TensorKind::LongTensor(ref t) => t,
            _ => unreachable!(),
        }
    }
}

impl<'a, T: NumLimits> From<&'a mut TensorKind> for &'a mut Tensor<T> {
    #[allow(unused_variables)]
    default fn from(input: &'a mut TensorKind) -> Self {
        panic!("bad cast")
    }
}

impl<'a> From<&'a mut TensorKind> for &'a mut Tensor<f32> {
    fn from(input: &'a mut TensorKind) -> Self {
        match *input {
            TensorKind::FloatTensor(ref mut t) => t,
            _ => unreachable!(),
        }
    }
}

impl<'a> From<&'a mut TensorKind> for &'a mut Tensor<i64> {
    fn from(input: &'a mut TensorKind) -> Self {
        match *input {
            TensorKind::LongTensor(ref mut t) => t,
            _ => unreachable!(),
        }
    }
}

impl From<TensorKind> for Tensor<f32> {
    fn from(input: TensorKind) -> Self {
        match input {
            TensorKind::FloatTensor(v) => v,
            _ => unimplemented!(),
        }
    }
}
/*
impl From<TensorKind> for Tensor<f64> {
    fn from(input: TensorKind) -> Self {
        match input {
            TensorKind::DoubleTensor(v) => v,
            _ => unimplemented!(),
        }
    }
}
*/
impl From<TensorKind> for Tensor<i64> {
    fn from(input: TensorKind) -> Self {
        match input {
            TensorKind::LongTensor(v) => v,
            _ => unimplemented!(),
        }
    }
}
impl From<TensorKind> for Tensor<u8> {
    fn from(input: TensorKind) -> Self {
        match input {
            TensorKind::ByteTensor(v) => v,
            _ => unimplemented!(),
        }
    }
}

pub struct Tensor<T: NumLimits> {
    pub value: RcMut<TensorImpl<T, Output = T>>,
}

impl<T: NumLimits> fmt::Debug for Tensor<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Tensor ")
    }
}


impl<T: NumLimits> Serialize for Tensor<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let rt = self.value.borrow().to_rust_tensor();
        let result = rt.serialize(serializer)?;
        Ok(result)
    }
}
impl<'de, T: NumLimits + Deserialize<'de>> Deserialize<'de> for Tensor<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let rt: RustTensor<T> = RustTensor::deserialize(deserializer)?;
        let mut t = ::torch::tensor(());
        t.from_rust_tensor(rt);
        Ok(t)
    }
}

impl<T: NumLimits> Hash for Tensor<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state)
    }
}

impl<T: NumLimits> Index<usize> for Tensor<T> {
    type Output = T;
    fn index(&self, idx: usize) -> &Self::Output {
        let t = unsafe { &mut *self.value.as_ptr() };
        t.index(idx)
    }
}
impl<T: NumLimits> IndexMut<usize> for Tensor<T> {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        let t = unsafe { &mut *self.value.as_ptr() };
        t.index_mut(idx)
    }
}

impl<T: NumLimits> Index<i32> for Tensor<T> {
    type Output = T;
    fn index(&self, idx: i32) -> &Self::Output {
        let t = unsafe { &mut *self.value.as_ptr() };
        t.index(idx as usize)
    }
}

impl<T: NumLimits> Tensor<T> {
    pub fn cast<D>(&self) -> Tensor<D>
        where D: NumLimits
    {
        let t: Tensor<D> = torch::tensor(self.size());
        let s: Vec<D> = self.value
            .borrow()
            .iter()
            .map(|v| <D as num::NumCast>::from(v).unwrap())
            .collect();
        t.value.borrow_mut().set_storage(s.as_slice());
        t
    }
    pub fn from_rust_tensor(&mut self, rt: RustTensor<T>) {
        self.value.borrow_mut().from_rust_tensor(rt);
    }
    pub fn get_storage(&self, data: &mut [T], len: usize) {
        assert!(len >= self.size().iter().product());
        let storage = self.value.borrow().get_storage(data);
    }
    pub fn is_valid(&self) -> bool {
        self.value.borrow().is_valid()
    }
    pub fn new<S>(&self, args: S) -> Self
        where S: Into<THVec<T>>
    {
        let args: THVec<T> = args.into();
        if args.dims.len() == 0 {
            self.value.borrow().new()
        } else {
            ::torch::tensor(args)
        }
    }
    pub fn s<D>(&self, dim: D) -> Self
        where D: AsRef<[isize]>
    {
        self.value.borrow().s(dim.as_ref())
    }
    pub fn iter(&self) -> Box<Iterator<Item = T>> {
        unimplemented!()
    }
    pub fn set_storage(&mut self, args: Vec<T>) {
        self.value.borrow_mut().set_storage(args.as_slice());
    }
}

impl<T: NumLimits> Default for Tensor<T> {
    fn default() -> Self {
        unimplemented!()
    }
}

impl<T: NumLimits> Clone for Tensor<T> {
    fn clone(&self) -> Self {
        Tensor { value: self.value.clone() }
    }
}

type RefTI<T> = RcMut<TensorImpl<T, Output = T>>;
pub type TIArg<T> = TensorImpl<T, Output = T>;
pub trait TensorImpl<T: NumLimits>: Index<Ix, Output = T> + IndexMut<Ix> {
    fn new(&self) -> Tensor<T>;
    fn abs(&mut self, src: *mut c_void);
    fn acos(&mut self, src: *mut c_void);
    fn add(&mut self, src: *mut c_void, value: T);
    fn addbmm(&mut self,
              beta: T,
              bias: *mut c_void,
              alpha: T,
              mat1: *mut c_void,
              mat2: *mut c_void);
    fn addcdiv(&mut self, bias: *mut c_void, alpha: T, mat1: *mut c_void, mat2: *mut c_void);
    fn addcmul(&mut self, bias: *mut c_void, alpha: T, mat1: *mut c_void, mat2: *mut c_void);
    fn addmm(&mut self,
             beta: T,
             bias: *mut c_void,
             alpha: T,
             mat1: *mut c_void,
             mat2: *mut c_void);
    fn addmv(&mut self,
             beta: T,
             bias: *mut c_void,
             alpha: T,
             mat1: *mut c_void,
             mat2: *mut c_void);
    fn addr(&mut self,
            beta: T,
            bias: *mut c_void,
            alpha: T,
            mat1: *mut c_void,
            mat2: *mut c_void);
    fn addt(&mut self, src: *mut c_void, alpha: T, value: *mut c_void);
    fn asin(&mut self, src: *mut c_void);
    fn atan(&mut self, src: *mut c_void);
    fn atan2(&mut self, src: *mut c_void);
    fn baddbmm(&mut self,
               beta: T,
               bias: *mut c_void,
               alpha: T,
               mat1: *mut c_void,
               mat2: *mut c_void);
    fn bernoulli(&mut self, p: f64);
    fn bmm(&mut self, a: *mut c_void, b: *mut c_void);
    fn ceil(&mut self, src: *mut c_void);
    fn clamp(&mut self, src: *mut c_void, min: T, max: T);
    fn copy(&mut self, src: &RefTI<T>);
    fn cos(&mut self, src: *mut c_void);
    fn cosh(&mut self, src: *mut c_void);
    fn cross(&mut self, src: *mut c_void, dim: Option<i32>);
    fn chunk(&self, n_chunks: usize, dim: usize) -> Vec<Tensor<T>>;
    fn diag(&mut self, src: *mut c_void, diag: u32);
    fn dim(&self) -> i32;
    fn dist(&self, src: *mut c_void, p: u32) -> f64;
    fn div(&mut self, src: *mut c_void, value: T);
    fn divt(&mut self, src: *mut c_void, value: *mut c_void);
    fn dot(&mut self, src: *mut c_void, value: *mut c_void);
    fn eig(&self, eigenvectors: bool, e: *mut c_void, v: *mut c_void);
    fn exp(&mut self, src: *mut c_void);
    fn expand(&self, dims: &[usize]) -> Tensor<T>;
    fn eq_tensor(&self, other: *mut c_void, out: *mut c_void);
    fn fill(&mut self, value: T);
    fn floor(&mut self, src: *mut c_void);
    fn fmod(&mut self, src: *mut c_void, value: T);
    fn frac(&mut self, src: *mut c_void);
    fn from_rust_tensor(&mut self, rt: RustTensor<T>);
    fn gather(&mut self, src: *mut c_void, dim: i32, index: *mut c_void);
    fn gels(&mut self, src: *mut c_void, other: *mut c_void);
    fn ge_tensor(&self, other: *mut c_void, out: *mut c_void);
    fn ge_value(&self, value: T, out: *mut c_void);
    fn get_storage(&self, data: &mut [T]);
    fn gt_tensor(&self, other: *mut c_void, out: *mut c_void);
    fn gt_value(&self, value: T, out: *mut c_void);
    fn index_add(&mut self, dim: i32, index: *mut c_void, tensor: *mut c_void);
    fn index_copy(&mut self, dim: i32, index: *mut c_void, tensor: *mut c_void);
    fn index_fill(&mut self, dim: i32, index: *mut c_void, val: T);
    fn index_select(&mut self, src: *mut c_void, dim: i32, index: *mut c_void);
    fn inner(&self) -> *mut c_void;
    fn is_cuda(&self) -> bool;
    fn is_valid(&self) -> bool;
    fn iter(&self) -> Box<Iterator<Item = T>>;
    fn kthvalue(&self, k: i32, dim: Option<i32>, keepdim: bool, v: *mut c_void, i: *mut c_void);
    fn le_tensor(&self, other: *mut c_void, out: *mut c_void);
    fn le_value(&self, value: T, out: *mut c_void);
    fn len(&self) -> usize;
    fn lerp(&mut self, src: *mut c_void, start: *mut c_void, end: *mut c_void, weight: f32);
    fn log(&mut self, src: *mut c_void);
    fn log1p(&mut self, src: *mut c_void);
    fn lt_tensor(&self, other: *mut c_void, out: *mut c_void);
    fn lt_value(&self, value: T, out: *mut c_void);
    fn masked_fill(&mut self, src: *mut c_void, mask: *mut c_void, value: T);
    fn masked_scatter(&mut self, src: *mut c_void, mask: *mut c_void, src: *mut c_void);
    fn masked_select(&mut self, src: *mut c_void, mask: *mut c_void);
    fn max(&self) -> T;
    fn max_reduce(&self, values: *mut c_void, indices: *mut c_void, dim: usize, keepdim: bool);
    fn mean(&self) -> f64;
    fn mean_reduce(&self, values: *mut c_void, indices: *mut c_void, dim: usize, keepdim: bool);
    fn min(&self) -> T;
    fn min_reduce(&self, values: *mut c_void, indices: *mut c_void, dim: usize, keepdim: bool);
    fn mm(&mut self, mat1: *mut c_void, mat2: *mut c_void);
    fn mul(&mut self, src: *mut c_void, value: T);
    fn mult(&mut self, src: *mut c_void, value: *mut c_void);
    fn mv(&mut self, mat: *mut c_void, vector: *mut c_void);
    fn narrow(&mut self, src: *mut c_void, dim: i32, start: i32, length: i32);
    fn neg(&mut self, src: *mut c_void);
    fn ne_tensor(&self, other: *mut c_void, out: *mut c_void);
    fn norm(&self, p: i32) -> f64;
    fn permute(&mut self, src: *mut c_void, dims: &[usize]);
    fn pin_memory(&mut self, src: *mut c_void);
    fn pow(&mut self, src: *mut c_void);
    fn prod(&self, result: &mut f64);
    fn reciprocal(&mut self, src: *mut c_void);
    fn remainder(&mut self, src: *mut c_void, value: T);
    fn repeat(&mut self, src: *mut c_void, sizes: &[usize]);
    fn resize(&mut self, dims: &[usize]);
    fn round(&mut self, src: *mut c_void);
    fn rsqrt(&mut self, src: *mut c_void);
    fn s(&self, dim: &[isize]) -> Tensor<T>;
    fn select(&self, other: *mut c_void, dim: i32, index: i32);
    fn set_storage(&mut self, v: &[T]);
    fn sigmoid(&mut self, src: *mut c_void);
    fn sign(&mut self, src: *mut c_void);
    fn sin(&mut self, src: *mut c_void);
    fn sinh(&mut self, src: *mut c_void);
    fn size(&self) -> Vec<usize>;
    fn sort(&self, dim: Option<i32>, descending: bool, t: *mut c_void, i: *mut c_void);
    fn sqrt(&mut self, src: *mut c_void);
    fn std(&self) -> f64;
    fn stride(&self) -> Vec<i32>;
    fn sub(&mut self, src: *mut c_void, rhs: *mut c_void);
    fn squeeze(&mut self, dim: Option<usize>);
    fn sum_float(&self, result: &mut f64);
    fn sum_reduce(&mut self, input: *mut c_void, dim: usize, keepdim: bool);
    fn svd(&self, some: bool, u: *mut c_void, s: *mut c_void, v: *mut c_void);
    fn tan(&mut self, src: *mut c_void);
    fn tanh(&mut self, src: *mut c_void);
    fn to_rust_tensor(&self) -> RustTensor<T>;
    fn topk(&self,
            k: i32,
            dim: Option<i32>,
            largest: bool,
            sorted: bool,
            v: *mut c_void,
            i: *mut c_void);
    fn trace(&mut self, src: *mut c_void);
    fn transpose(&mut self, src: *mut c_void, dim0: usize, dim1: usize);
    fn trunc(&mut self, src: *mut c_void);
    fn unfold(&self, src: *mut c_void, dim: i32, size: i32, step: i32);
    fn uniform(&mut self, range: (f64, f64));
    fn unsqueeze(&mut self, dim: usize);
    fn var(&self) -> f64;
    fn view(&self, dims: &[isize]) -> Tensor<T>;
    fn zero(&mut self);
}

pub struct Generator {
    t: *mut THGenerator,
}
impl Generator {
    pub fn new() -> Self {
        let t = unsafe { THGenerator_new() };
        Generator { t: t }
    }
}
#[derive(Serialize, Deserialize)]
pub struct RustTensor<T> {
    size: Vec<i64>,
    stride: Vec<i64>,
    storage: Vec<T>,
}

macro_rules!  unsafe_index {
    ($p:ident, $idx:ident) => {
         unsafe { & *(*(* $p .t).storage).data.offset($idx as isize) }
    }
}
macro_rules!  unsafe_index_mut {
    ($p:ident, $idx:ident) => {
         unsafe { &mut *(*(* $p .t).storage).data.offset($idx as isize) }
    }
}

macro_rules! impl_tensor_impl {
    ($name:ident, $type:ident, $thname:ident, $storage_name:ident) => {
        pub struct $name {
            t: *mut $thname,
        }
        impl $name {
            pub fn new() -> Self {
                unsafe {
                    $name {
                        t: concat_idents!($thname, _new)(),
                    }
                }
            }
            fn from_parts(t: *mut $thname) -> Self {
                $name { t: t}
            }
            fn to_rust_tensor(&self) -> RustTensor<$type> {
                let mut size: Vec<i64> = Vec::new();
                let mut stride: Vec<i64> = Vec::new();
                let mut storage = Vec::new();
                let (offset, nd) = (self.storage_offset(), self.dim());
                let need_stride = unsafe {(*self.t).stride != std::ptr::null_mut()};

                for i in 0..nd {
                    let s = unsafe { &*(*self.t).size.offset(i as isize) };
                    size.push(*s);
                }
                if need_stride {
                    for i in 0..nd {
                        let s = unsafe { &*(*self.t).stride.offset(i as isize) };
                        stride.push(*s);
                    }
                }
                let s = self.storage();
                /* XXX this has to be the slowest way possible */
                for i in s.iter() {
                    storage.push(*i);
                }
                RustTensor {size: size, stride: stride, storage: storage}
            }
            pub fn with_capacity<D>(dims: D) -> Self
                where D: AsRef<[usize]>
            {
                let dims_long : Vec<i64> = dims.as_ref().iter().map(|t| *t as i64).collect();
                let dims = dims.as_ref();
                let sizes = LongStorage::with_data(dims_long.as_slice());
                let size = dims.iter().product();
                let storage = $storage_name ::with_capacity(size);
                let t = unsafe {
                    concat_idents!($thname, _newWithStorage)(storage.t,
                                                             0,
                                                             sizes.t,
                                                             std::ptr::null_mut())
                };
                $name { t: t}
            }
            pub fn randn<D>(dims: D) -> Self
                where D: AsRef<[usize]>
            {
                let dims = dims.as_ref();
                unimplemented!()
            }
            fn len(&self) -> usize {
                self.size().iter().product()
            }
            fn storage(&self) -> $storage_name {
                let s = unsafe { (*self.t).storage };
                $storage_name ::from_raw_parts(s)
            }
            fn storage_offset(&self) -> usize {
                (unsafe {(*self.t).storageOffset}) as usize
            }
        }

        impl TensorImpl<$type> for $name {
            fn abs(&mut self, src: *mut c_void) {
                let srcp = src as *mut $thname;
                unsafe { concat_idents!($thname, _abs)(self.t, srcp) };
            }
            fn acos(&mut self, src: *mut c_void) {
                let srcp = src as *mut $thname;
                unsafe { concat_idents!($thname, _acos)(self.t, srcp) };
            }
            fn add(&mut self, src: *mut c_void, value: $type) {
                let srcp = src as *mut $thname;
                unsafe {concat_idents!($thname, _add)(self.t, srcp, value)};
            }
            fn addbmm(&mut self,
                    beta: $type,
                    bias: *mut c_void,
                    alpha: $type,
                    mat1: *mut c_void,
                    mat2: *mut c_void) {
                let biasp = bias as *mut $thname;
                let (mat1p, mat2p) = (mat1 as *mut $thname, mat2 as *mut $thname);
                unsafe {
                    concat_idents!($thname, _addbmm)(self.t,
                                                    beta,
                                                    biasp,
                                                    alpha,
                                                    mat1p,
                                                    mat2p)
                };
            }
            fn addcdiv(&mut self,
                    bias: *mut c_void,
                    alpha: $type,
                    mat1: *mut c_void,
                    mat2: *mut c_void) {
                let biasp = bias as *mut $thname;
                let (mat1p, mat2p) = (mat1 as *mut $thname, mat2 as *mut $thname);
                unsafe {
                    concat_idents!($thname, _addcdiv)(self.t,
                                                    biasp,
                                                    alpha,
                                                    mat1p,
                                                    mat2p)
                };
            }
            fn addcmul(&mut self,
                    bias: *mut c_void,
                    alpha: $type,
                    mat1: *mut c_void,
                    mat2: *mut c_void) {
                let biasp = bias as *mut $thname;
                let (mat1p, mat2p) = (mat1 as *mut $thname, mat2 as *mut $thname);
                unsafe {
                    concat_idents!($thname, _addcmul)(self.t,
                                                    biasp,
                                                    alpha,
                                                    mat1p,
                                                    mat2p)
                };
            }
            fn addmm(&mut self,
                    beta: $type,
                    bias: *mut c_void,
                    alpha: $type,
                    mat1: *mut c_void,
                    mat2: *mut c_void) {
                let biasp = bias as *mut $thname;
                let (mat1p, mat2p) = (mat1 as *mut $thname, mat2 as *mut $thname);
                unsafe {
                    concat_idents!($thname, _addmm)(self.t,
                                                    beta,
                                                    biasp,
                                                    alpha,
                                                    mat1p,
                                                    mat2p)
                };
            }
            fn addmv(&mut self,
                    beta: $type,
                    bias: *mut c_void,
                    alpha: $type,
                    mat1: *mut c_void,
                    mat2: *mut c_void) {
                let biasp = bias as *mut $thname;
                let (mat1p, mat2p) = (mat1 as *mut $thname, mat2 as *mut $thname);
                unsafe {
                    concat_idents!($thname, _addmv)(self.t,
                                                    beta,
                                                    biasp,
                                                    alpha,
                                                    mat1p,
                                                    mat2p)
                };
            }
            fn addr(&mut self,
                    beta: $type,
                    bias: *mut c_void,
                    alpha: $type,
                    mat1: *mut c_void,
                    mat2: *mut c_void) {
                let biasp = bias as *mut $thname;
                let (mat1p, mat2p) = (mat1 as *mut $thname, mat2 as *mut $thname);
                unsafe {
                    concat_idents!($thname, _addr)(self.t,
                                                    beta,
                                                    biasp,
                                                    alpha,
                                                    mat1p,
                                                    mat2p)
                };
            }
            fn addt(&mut self,
                    src: *mut c_void,
                    alpha: $type,
                    value: *mut c_void) {
                let srcp = src as *mut $thname;
                let valuep = value as *mut $thname;
                unsafe { concat_idents!($thname, _cadd)(self.t, valuep, alpha, srcp)};
            }
            fn asin(&mut self, src: *mut c_void) {
                let srcp = src as *mut $thname;
                unsafe { concat_idents!($thname, _asin)(self.t, srcp) };
             }
            fn atan(&mut self, src: *mut c_void) {
                let srcp = src as *mut $thname;
                unsafe { concat_idents!($thname, _atan)(self.t, srcp) };
             }
            fn atan2(&mut self, src: *mut c_void) {
                unimplemented!();
             }
            fn baddbmm(&mut self,
                    beta: $type,
                    bias: *mut c_void,
                    alpha: $type,
                    mat1: *mut c_void,
                    mat2: *mut c_void) {
                let biasp = bias as *mut $thname;
                let (mat1p, mat2p) = (mat1 as *mut $thname, mat2 as *mut $thname);
                unsafe {
                    concat_idents!($thname, _baddbmm)(self.t,
                                                    beta,
                                                    biasp,
                                                    alpha,
                                                    mat1p,
                                                    mat2p)
                };
            }
            fn bernoulli(&mut self, p: f64) {
                let g = Generator::new();
                unsafe { concat_idents!($thname, _bernoulli)(self.t, g.t, p) };
            }
            fn bmm(&mut self, a: *mut c_void, b: *mut c_void) {
                unimplemented!()
            }
            fn ceil(&mut self, src: *mut c_void) {
                let srcp = src as *mut $thname;
                unsafe { concat_idents!($thname, _ceil)(self.t, srcp) };
             }
            fn clamp(&mut self, src: *mut c_void, min: $type, max: $type) {
                unimplemented!()
            }
            fn copy(&mut self, src: &RefTI<$type>) {
                let t = src.borrow_mut().inner() as *mut $thname;
                unsafe {concat_idents!($thname, _copy)(self.t, t)}
            }
            fn cos(&mut self, src: *mut c_void) {
                let srcp = src as *mut $thname;
                unsafe { concat_idents!($thname, _cos)(self.t, srcp) };
             }
            fn cosh(&mut self, src: *mut c_void) {
                let srcp = src as *mut $thname;
                unsafe { concat_idents!($thname, _cosh)(self.t, srcp) };
             }
            fn cross(&mut self, src: *mut c_void, dim: Option<i32>) {
                unimplemented!()
            }
            fn chunk(&self, n_chunks: usize, dim: usize) -> Vec<Tensor<$type>> {
                unimplemented!()
            }
            fn diag(&mut self, src: *mut c_void, diag: u32) {
                unimplemented!()
            }
            fn dim(&self) -> i32 {
                unsafe {(*self.t).nDimension}
            }
            fn dist(&self, other: *mut c_void, p: u32) -> f64 {
                unimplemented!()
            }
            fn div(&mut self, src: *mut c_void, value: $type) {
                let srcp = src as *mut $thname;
                unsafe {concat_idents!($thname, _div)(self.t, srcp, value)};
            }
            fn divt(&mut self,
                    src: *mut c_void,
                    value: *mut c_void) {
                unimplemented!()
            }
            fn dot(&mut self,
                    src: *mut c_void,
                    value: *mut c_void) {
                let srcp = src as *mut $thname;
                let valuep = value as *mut $thname;
            }
            fn eig(&self, eigenvectors: bool, e: *mut c_void, v: *mut c_void) {
                unimplemented!()
            }
            fn eq_tensor(&self, other: *mut c_void, out: *mut c_void) {
                let outp = out as *mut THByteTensor;
                let otherp = other as *mut $thname;
                unsafe {concat_idents!($thname, _eqTensor)(outp, self.t, otherp) }
            }
            fn exp(&mut self, src: *mut c_void) {
                let srcp = src as *mut $thname;
                unsafe { concat_idents!($thname, _exp)(self.t, srcp) };
             }
            fn expand(&self, dims: &[usize]) -> Tensor<$type> {
                let dims_long : Vec<i64> = dims.iter().map(|t| *t as i64).collect();
                let size = LongStorage::with_data(dims_long.as_slice());
                let newt = unsafe {
                    concat_idents!($thname, _newExpand)(self.t, size.t)
                };
                let t = $name :: from_parts(newt);
                Tensor { value: RcMutNew(t) }
            }
            fn fill(&mut self, value: $type) {
                unimplemented!()
            }
            fn floor(&mut self, src: *mut c_void) {
                let srcp = src as *mut $thname;
                unsafe { concat_idents!($thname, _floor)(self.t, srcp) };
             }
            fn fmod(&mut self, src: *mut c_void, value: $type) {
                unimplemented!()
            }
            fn frac(&mut self, src: *mut c_void) {
                let srcp = src as *mut $thname;
                unsafe { concat_idents!($thname, _frac)(self.t, srcp) };
             }
            fn from_rust_tensor(&mut self, rt: RustTensor<$type>) {
                let size : Vec<usize> = rt.size.iter().map(|t| *t as usize).collect();
                self.resize(size.as_slice());
                let s = rt.storage.as_ptr() as *const c_void;
                let d = (unsafe { ((*(*self.t).storage).data) }) as *mut c_void;
                unsafe {memcpy(d, s, rt.storage.len()) };
            }
            fn gather(&mut self, src: *mut c_void, dim: i32, index: *mut c_void) {
                unimplemented!()
            }
            fn gels(&mut self, src: *mut c_void, other: *mut c_void) {
                unimplemented!()
            }
            fn ge_tensor(&self, other: *mut c_void, out: *mut c_void) {
                unimplemented!()
            }
            fn ge_value(&self, value: $type, out: *mut c_void) {
                unimplemented!()
            }
            fn get_storage(&self, data: &mut [$type]) {
                /* XXX presupposes contiguity */
                let offset = self.storage_offset() as isize;
                let s = unsafe {(*(*self.t).storage).data.offset(offset)};
                let s = s as *const c_void;
                let d = data.as_mut_ptr() as *mut c_void;
                unsafe {memcpy(d, s, data.len()) };
            }
            fn gt_tensor(&self, other: *mut c_void, out: *mut c_void) {
                unimplemented!()
            }
            fn gt_value(&self, value: $type, out: *mut c_void) {
                unimplemented!()
            }
            fn index_add(&mut self, dim: i32, index: *mut c_void, tensor: *mut c_void) {
                unimplemented!()
            }
            fn index_copy(&mut self, dim: i32, index: *mut c_void, tensor: *mut c_void) {
                unimplemented!()
            }
            fn index_fill(&mut self, dim: i32, index: *mut c_void, val: $type) {
                unimplemented!()
            }
            fn index_select(&mut self, src: *mut c_void, dim: i32, index: *mut c_void) {
                unimplemented!()
            }
            fn inner(&self) -> *mut c_void {
                self.t as *mut c_void
            }
            fn is_cuda(&self) -> bool {
                false
            }
            fn is_valid(&self) -> bool {
                self.is_valid()
            }
            fn iter(&self) -> Box<Iterator<Item=$type>> {
                unimplemented!()
            }
            fn kthvalue(&self,
                        k: i32,
                        dim: Option<i32>,
                        keepdim: bool,
                        v: *mut c_void,
                        i: *mut c_void) {
                unimplemented!()
            }
            fn le_tensor(&self, other: *mut c_void, out: *mut c_void) {
                unimplemented!()
            }
            fn le_value(&self, value: $type, out: *mut c_void) {
                unimplemented!()
            }
            fn len(&self) -> usize {
                self.len()
            }
            fn lerp(&mut self,
                    src: *mut c_void,
                    start: *mut c_void,
                    end: *mut c_void,
                    weight: f32) {
                unimplemented!()
            }
            fn log(&mut self, src: *mut c_void) {
                let srcp = src as *mut $thname;
                unsafe { concat_idents!($thname, _log)(self.t, srcp) };
             }
            fn log1p(&mut self, src: *mut c_void) {
                let srcp = src as *mut $thname;
                unsafe { concat_idents!($thname, _log1p)(self.t, srcp) };
             }
            fn lt_tensor(&self, other: *mut c_void, out: *mut c_void) {
                unimplemented!()
            }
            fn lt_value(&self, value: $type, out: *mut c_void) {
                unimplemented!()
            }
            fn masked_fill(&mut self, src: *mut c_void, mask: *mut c_void, value: $type) {
                unimplemented!()
            }
            fn masked_scatter(&mut self, src: *mut c_void, mask: *mut c_void, source: *mut c_void) {
                unimplemented!()
            }
            fn masked_select(&mut self, src: *mut c_void, mask: *mut c_void) {
                unimplemented!()
            }
            fn max(&self) -> $type {
                unimplemented!()
            }
            fn max_reduce(&self,
                          values: *mut c_void,
                          indices: *mut c_void,
                          dim: usize,
                          keepdim: bool) {
                let valuesp = values as *mut $thname;
                let indicesp = values as *mut THLongTensor;
                let dim = dim as i32;
                let keep = keepdim as i32;
                unsafe { concat_idents!($thname, _max)(valuesp, indicesp, self.t, dim, keep) }
            }
            fn mean(&self) -> f64 {
                unimplemented!()
            }
            fn mean_reduce(&self,
                           values: *mut c_void,
                           indices: *mut c_void,
                           dim: usize,
                           keepdim: bool) {
                unimplemented!()
            }
            fn min(&self) -> $type {
                unimplemented!()
            }
            fn min_reduce(&self,
                          values: *mut c_void,
                          indices: *mut c_void,
                          dim: usize,
                          keepdim: bool) {
                let valuesp = values as *mut $thname;
                let indicesp = values as *mut THLongTensor;
                let dim = dim as i32;
                let keep = keepdim as i32;
                assert!(dim < self.dim());
                unsafe { concat_idents!($thname, _min)(valuesp, indicesp, self.t, dim, keep) }
            }
            fn mm(&mut self, mat1: *mut c_void, mat2: *mut c_void) {
                let (mat1p, mat2p) = (mat1 as *mut $thname, mat2 as *mut $thname);
                unsafe {
                    concat_idents!($thname, _addmm)(self.t,
                                                    0 as $type,
                                                    self.t,
                                                    1 as $type,
                                                    mat1p,
                                                    mat2p)
                };
            }
            fn mul(&mut self, src: *mut c_void, value: $type) {
                let srcp = src as *mut $thname;
                unsafe { concat_idents!($thname, _mul)(self.t, srcp, value)};
            }
            fn mult(&mut self,
                    src: *mut c_void,
                    value: *mut c_void) {
                let srcp = src as *mut $thname;
                let valuep = value as *mut $thname;
                unsafe { concat_idents!($thname, _cmul)(self.t, valuep, srcp)};
            }
            fn mv(&mut self, mat: *mut c_void, vector: *mut c_void) {
                unimplemented!()
            }
            fn narrow(&mut self, src: *mut c_void, dim: i32, start: i32, length: i32) {
                unimplemented!()
            }
            fn ne_tensor(&self, other: *mut c_void, out: *mut c_void) {
                unimplemented!()
            }
            fn neg(&mut self, src: *mut c_void) {
                let srcp = src as *mut $thname;
                unsafe { concat_idents!($thname, _neg)(self.t, srcp) };
             }
            fn new(&self) -> Tensor<$type> {
                Tensor { value: RcMutNew($name ::new()) }
            }
            fn norm(&self, p: i32) -> f64 {
                unimplemented!()
            }
            fn permute(&mut self, src: *mut c_void, dims: &[usize]) {
                unimplemented!()
            }
            fn pin_memory(&mut self, src: *mut c_void) {
                unimplemented!();
             }
            fn pow(&mut self, src: *mut c_void) {
                let srcp = src as *mut $thname;
                unsafe { concat_idents!($thname, _pow)(self.t, srcp, 1.) };
             }
            fn prod(&self, mut result: &mut f64) {
                let val: $type;
                let r = unsafe { concat_idents!($thname, _prodall)(self.t) };
                *result = <f64 as num::NumCast>::from(r).unwrap();
            }
            fn reciprocal(&mut self, src: *mut c_void) {
                unimplemented!();
             }
            fn remainder(&mut self, src: *mut c_void, value: $type) {
                unimplemented!()
            }
            fn repeat(&mut self, src: *mut c_void, sizes: &[usize]) {
                unimplemented!()
            }
            fn resize(&mut self, dims: &[usize]) {
                let dims : Vec<i64> = dims.iter().map(|v| *v as i64).collect();
                let dims = LongStorage::with_data(dims);
                unsafe { concat_idents!($thname, _resize)(self.t, dims.t, std::ptr::null_mut()) };
            }
            fn round(&mut self, src: *mut c_void) {
                let srcp = src as *mut $thname;
                unsafe { concat_idents!($thname, _round)(self.t, srcp) };
             }
            fn rsqrt(&mut self, src: *mut c_void) {
                let srcp = src as *mut $thname;
                unsafe { concat_idents!($thname, _rsqrt)(self.t, srcp) };
             }
            fn s(&self, dims: &[isize]) -> Tensor<$type> {
                let sizes = self.size();
                if sizes.len() < dims.len() {
                    panic!("bad slice index {:?}", dims);
                }
                for i in 0..dims.len() {
                    if dims[i] as usize >= sizes[i] || dims[i] < -1 {
                        panic!("{} out of range {:?}", dims[i], sizes[i]);
                    }
                }
                let mut ptr: *mut $thname = self.t;
                for (i, dim) in dims.iter().enumerate() {
                    if *dim == -1 { continue }
                    ptr = unsafe {concat_idents!($thname, _newSelect)(ptr, i as i32, *dim as i64)};
                }
                let t = $name :: from_parts(ptr);
                Tensor { value: RcMutNew(t) }
            }
            fn select(&self, other: *mut c_void, dim: i32, index: i32) {
                unimplemented!()
            }
            fn set_storage(&mut self, v: &[$type]) {
                let storage_offset = self.storage_offset();
                let mut s = self.storage();
                assert_eq!(v.len(), s.len());
                // XXX memcpy
                for i in 0..s.len() {
                    s[(storage_offset + i)] = v[i]
                }
            }
            fn sigmoid(&mut self, src: *mut c_void) {
                let srcp = src as *mut $thname;
                unsafe { concat_idents!($thname, _sigmoid)(self.t, srcp) };
             }
            fn sign(&mut self, src: *mut c_void) {
                unimplemented!();
             }
            fn sin(&mut self, src: *mut c_void) {
                let srcp = src as *mut $thname;
                unsafe { concat_idents!($thname, _sin)(self.t, srcp) };
             }
            fn sinh(&mut self, src: *mut c_void) {
                let srcp = src as *mut $thname;
                unsafe { concat_idents!($thname, _sinh)(self.t, srcp) };
             }
            fn size(&self) -> Vec<usize> {
                let d = unsafe { std::slice::from_raw_parts((*self.t).size as *mut usize,
                                                            (*self.t).nDimension as usize)};
                d.to_vec()
            }
            fn sort(&self, dim: Option<i32>, descending: bool, t: *mut c_void, i: *mut c_void) {
                unimplemented!()
            }
            fn sqrt(&mut self, src: *mut c_void) {
                let srcp = src as *mut $thname;
                unsafe { concat_idents!($thname, _sqrt)(self.t, srcp) };
             }
            fn squeeze(&mut self, dim: Option<usize>) {
                let mut dims = Vec::new();
                if let Some(t) = dim {
                    dims.push(t)
                } else {
                    for (i, d) in self.size().iter().enumerate() {
                        if *d == 1 {
                            dims.push(i);
                        }
                    }
                    dims.reverse();
                }
                for d in dims {
                    let p = ::std::ptr::null_mut();
                    unsafe {concat_idents!($thname, _squeeze1d)(self.t, p, d as i32) };
                }
            }
            fn std(&self) -> f64 {
                unimplemented!()
            }
            fn stride(&self) -> Vec<i32> {
                unimplemented!()
            }
            fn sub(&mut self, src: *mut c_void, rhs: *mut c_void) {
                unimplemented!();
             }
            fn sum_float(&self, mut result: &mut f64) {
                let r = unsafe { concat_idents!($thname, _sumall)(self.t) };
                let f = <f64 as ::num::NumCast>::from(r);
                if let Some(res) = f {
                    *result = res;
                } else {
                    panic!("can't cast r: {}", r);
                }
            }
            fn sum_reduce(&mut self, input: *mut c_void, dim: usize, keepdim: bool) {
                let input = input as *mut $thname;
                unsafe  { concat_idents!($thname, _sum)(self.t, input, dim as i32, keepdim as i32)};
            }
            fn svd(&self, some: bool, u: *mut c_void, s: *mut c_void, v: *mut c_void) {
                unimplemented!()
            }
            fn tan(&mut self, src: *mut c_void) {
                let srcp = src as *mut $thname;
                unsafe { concat_idents!($thname, _tan)(self.t, srcp) };
             }
            fn tanh(&mut self, src: *mut c_void) {
                let srcp = src as *mut $thname;
                unsafe { concat_idents!($thname, _tanh)(self.t, srcp) };
             }
            fn to_rust_tensor(&self) -> RustTensor<$type> {
                self.to_rust_tensor()
            }
            fn topk(&self,
                    k: i32,
                    dim: Option<i32>,
                    largest: bool,
                    sorted: bool,
                    v: *mut c_void,
                    i: *mut c_void) {
                unimplemented!()
            }
            fn trace(&mut self, src: *mut c_void) {
                unimplemented!();
             }
            fn transpose(&mut self, src: *mut c_void, dim0: usize, dim1: usize) {
                let (dim0, dim1) = (dim0 as i32, dim1 as i32);
                let srcp = src as *mut $thname;
                unsafe {concat_idents!($thname, _transpose)(self.t, srcp, dim0, dim1)};
            }
            fn trunc(&mut self, src: *mut c_void) {
                let srcp = src as *mut $thname;
                unsafe { concat_idents!($thname, _trunc)(self.t, srcp) };
             }
            fn unfold(&self, src: *mut c_void, dim: i32, size: i32, step: i32) {
                unimplemented!()
            }
            fn uniform(&mut self, range: (f64, f64)) {
                let g = Generator::new();
                #[allow(unused_unsafe)]
                unsafe { concat_idents!($thname, _uniform)(self.t, g.t, range.0, range.1) };
            }
            fn unsqueeze(&mut self, dim: usize) {
                let p = ::std::ptr::null_mut();
                unsafe {concat_idents!($thname, _squeeze1d)(self.t, p, dim as i32) };
            }
            fn var(&self) -> f64 {
                unimplemented!()
            }
            fn view(&self, dims: &[isize]) -> Tensor<$type> {
                let dims_long : Vec<i64> = dims.iter().map(|t| *t as i64).collect();
                let size = LongStorage::with_data(dims_long.as_slice());
                let inferred_size = unsafe {
                    let numel = concat_idents!($thname, _nElement)(self.t);
                    let p = THLongStorage_newInferSize(size.t, numel);
                    LongStorage {t: p}
                };
                let t = unsafe { concat_idents!($thname, _newView)(self.t, inferred_size.t)  };
                let t = $name :: from_parts(t);
                Tensor {value: RcMutNew(t) }
            }
            fn zero(&mut self) {
                unsafe { concat_idents!($thname, _zero)(self.t) };
            }
        }
        impl Default for $name {
            fn default() -> Self {
                $name ::new()
            }
        }
        impl<'a> Index<&'a [isize]> for $name {
            type Output = $type;

            fn index(&self, idx: &'a [isize]) -> &Self::Output {
                let mut index = 0;
                let lastidx = max(0, idx.len() as isize - 1) as usize;
                let dims = self.size();
                if idx.len() != dims.len() {
                    panic!("bad dimlen")
                }
                for i in 0..lastidx {
                    if idx[i] >= dims[i] as isize {
                        panic!("bad dimlen")
                    }
                    index += idx[i] as usize * dims[i];
                }
                if idx[lastidx] >= dims[lastidx] as isize {
                    panic!("bad dimlen")
                }
                index += idx[lastidx] as usize;
                unsafe_index!(self, index)
            }
        }

        impl<'a> IndexMut<&'a [isize]> for $name {
            fn index_mut(&mut self, idx: &'a [isize]) -> &mut Self::Output {
                let mut index = 0;
                let lastidx = max(0, idx.len() as isize - 1) as usize;
                let dims = self.size();
                if idx.len() != dims.len() {
                    panic!("bad dimlen")
                }
                for i in 0..lastidx {
                    if idx[i] >= dims[i] as isize {
                        panic!("bad dimlen")
                    }
                    index += idx[i] as usize * dims[i];
                }
                if idx[lastidx] >= dims[lastidx] as isize {
                    panic!("bad dimlen")
                }
                index += idx[lastidx] as usize;
                unsafe_index_mut!(self, index)
            }
        }

        impl Index<usize> for $name {
            type Output = $type;
            fn index(&self, idx: usize) -> &Self::Output {
                let dims = self.dim();
                if dims != 1 {
                    panic!("bad index size 1D index an {}D dims: {:?}", dims, self.size())
                };
                let size = self.size();
                if size[0] <= idx {
                    panic!("idx {} out of range {}", idx, size[0])
                };
                unsafe_index!(self, idx)
            }
        }
        impl IndexMut<usize> for $name {
            fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
                let dims = self.dim();
                if dims != 1 {
                    panic!("bad index size 1D index an {}D dims: {:?}", dims, self.size())
                };
                let size = self.size();
                if size[0] <= idx {
                    panic!("idx {} out of range {}", idx, size[0])
                };
                unsafe_index_mut!(self, idx)
            }
        }
        impl Drop for $name {
            fn drop(&mut self) {
                unsafe { concat_idents!($thname, _free)(self.t) }
            }
        }
        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer
            {
                let rt = self.to_rust_tensor();
                let result = rt.serialize(serializer)?;
                Ok(result)
            }
        }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: Deserializer<'de>
            {
                unimplemented!()
            }
        }
    }
}

impl FloatTensor {
    pub fn is_valid(&self) -> bool {
        !self.storage().iter().any(|d| d.is_nan() || d.is_infinite())
    }
}
impl DoubleTensor {
    pub fn is_valid(&self) -> bool {
        !self.storage().iter().any(|d| d.is_nan() || d.is_infinite())
    }
}
impl ByteTensor {
    pub fn is_valid(&self) -> bool {
        true
    }
}
impl LongTensor {
    pub fn is_valid(&self) -> bool {
        true
    }
}


#[allow(non_snake_case, unused_variables)]
pub fn THByteTensor_uniform(self_: *mut THByteTensor,
                            _generator: *mut THGenerator,
                            a: f64,
                            b: f64) {
    panic!("no such function")
}
#[allow(non_snake_case, unused_variables)]
pub fn THLongTensor_uniform(self_: *mut THLongTensor,
                            _generator: *mut THGenerator,
                            a: f64,
                            b: f64) {
    panic!("no such function")
}



macro_rules! impl_tensor_unary_stub {
    ($thname:ident, $fnname:ident) => {
        #[allow(non_snake_case)]
        unsafe fn $fnname (r_: *mut $thname, t: *mut $thname) {
            panic!("no such function {:?}", r_);
        }
    }
}
impl_tensor_unary_stub!(THByteTensor, THByteTensor_abs);
impl_tensor_unary_stub!(THByteTensor, THByteTensor_acos);
impl_tensor_unary_stub!(THByteTensor, THByteTensor_asin);
impl_tensor_unary_stub!(THByteTensor, THByteTensor_atan);
impl_tensor_unary_stub!(THByteTensor, THByteTensor_ceil);
impl_tensor_unary_stub!(THByteTensor, THByteTensor_cos);
impl_tensor_unary_stub!(THByteTensor, THByteTensor_cosh);
impl_tensor_unary_stub!(THByteTensor, THByteTensor_exp);
impl_tensor_unary_stub!(THByteTensor, THByteTensor_floor);
impl_tensor_unary_stub!(THByteTensor, THByteTensor_frac);
impl_tensor_unary_stub!(THByteTensor, THByteTensor_log);
impl_tensor_unary_stub!(THByteTensor, THByteTensor_log1p);
impl_tensor_unary_stub!(THByteTensor, THByteTensor_neg);
impl_tensor_unary_stub!(THByteTensor, THByteTensor_round);
impl_tensor_unary_stub!(THByteTensor, THByteTensor_rsqrt);
impl_tensor_unary_stub!(THByteTensor, THByteTensor_sqrt);
impl_tensor_unary_stub!(THByteTensor, THByteTensor_sigmoid);
impl_tensor_unary_stub!(THByteTensor, THByteTensor_sin);
impl_tensor_unary_stub!(THByteTensor, THByteTensor_sinh);
impl_tensor_unary_stub!(THByteTensor, THByteTensor_tan);
impl_tensor_unary_stub!(THByteTensor, THByteTensor_tanh);
impl_tensor_unary_stub!(THByteTensor, THByteTensor_trunc);
#[allow(non_snake_case)]
unsafe fn THByteTensor_pow(r_: *mut THByteTensor, t: *mut THByteTensor, value: f64) {
    panic!("no such function {:?}", r_);
}

impl_tensor_unary_stub!(THLongTensor, THLongTensor_acos);
impl_tensor_unary_stub!(THLongTensor, THLongTensor_asin);
impl_tensor_unary_stub!(THLongTensor, THLongTensor_atan);
impl_tensor_unary_stub!(THLongTensor, THLongTensor_ceil);
impl_tensor_unary_stub!(THLongTensor, THLongTensor_cos);
impl_tensor_unary_stub!(THLongTensor, THLongTensor_cosh);
impl_tensor_unary_stub!(THLongTensor, THLongTensor_exp);
impl_tensor_unary_stub!(THLongTensor, THLongTensor_floor);
impl_tensor_unary_stub!(THLongTensor, THLongTensor_frac);
impl_tensor_unary_stub!(THLongTensor, THLongTensor_log);
impl_tensor_unary_stub!(THLongTensor, THLongTensor_log1p);
impl_tensor_unary_stub!(THLongTensor, THLongTensor_neg);
impl_tensor_unary_stub!(THLongTensor, THLongTensor_round);
impl_tensor_unary_stub!(THLongTensor, THLongTensor_rsqrt);
impl_tensor_unary_stub!(THLongTensor, THLongTensor_sqrt);
impl_tensor_unary_stub!(THLongTensor, THLongTensor_sigmoid);
impl_tensor_unary_stub!(THLongTensor, THLongTensor_sin);
impl_tensor_unary_stub!(THLongTensor, THLongTensor_sinh);
impl_tensor_unary_stub!(THLongTensor, THLongTensor_tan);
impl_tensor_unary_stub!(THLongTensor, THLongTensor_tanh);
impl_tensor_unary_stub!(THLongTensor, THLongTensor_trunc);
#[allow(non_snake_case)]
unsafe fn THLongTensor_pow(r_: *mut THLongTensor, t: *mut THLongTensor, value: f64) {
    panic!("no such function {:?}", r_);
}



impl_tensor_impl!(FloatTensor, f32, THFloatTensor, FloatStorage);
impl_tensor_impl!(DoubleTensor, f64, THDoubleTensor, DoubleStorage);
impl_tensor_impl!(LongTensor, i64, THLongTensor, LongStorage);
impl_tensor_impl!(ByteTensor, u8, THByteTensor, ByteStorage);

pub fn make_vec(val: usize, count: usize) -> Vec<isize> {
    let mut vec = Vec::new();
    for _ in 0..count {
        vec.push(val as isize)
    }
    vec
}
