use std::borrow::Cow;
use std::default::Default;
use std::error::Error;
use std::iter::repeat;
use std::mem;
use std::num::*;
use std::ops::*;
use std::str::{self, FromStr};
use std::fmt;
use std::cmp::Ordering;
use std::cmp::Ordering::*;
use std::ascii::AsciiExt;
use self::Sign::*;

fn safe_shr( val: u64, count: i16) -> u64{
    if count >= 0 {
        if count>=64 {
            0
        }else{
            val >> count
        }
    }else{
        if count <= -64 {
            0
        }else{
            val << -count
        }
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug)]
pub enum Sign{
    Negative,
    Neutral,
    Positive,
}

impl<T: Ord + Zero> From<T> for Sign {
    fn from(n: T) -> Self{
        match n.cmp(&T::zero()) {
            Less => Negative,
            Equal => Neutral,
            Greater => Positive,
        }
    }
}

impl From<Sign> for f32 {
    fn from(n: Sign) -> Self{
        match n {
            Negative => -1.0,
            Neutral => 0.0,
            Positive => 1.0,
        }
    }
}

impl From<Sign> for f64 {
    fn from(n: Sign) -> Self{
        match n {
            Negative => -1.0,
            Neutral => 0.0,
            Positive => 1.0,
        }
    }
}

impl fmt::Display for Sign {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Positive => write!(f, "+"),
            Neutral => write!(f, " "),
            Negative => write!(f, "-"),
        }
    }
}

impl Neg for Sign{
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self{
            Negative => Positive,
            Neutral => Neutral,
            Positive => Negative,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub struct f8_120{
    pub words: (u64, u64),
    pub sign: Sign,
}

impl f8_120 {
    pub fn new( words: (u64, u64), sign: Sign) -> f8_120{
        f8_120{
            words: words,
            sign: sign,
        }
    }

    fn words_cmp(&self, other: &Self) -> Ordering {
        match self.words.0.cmp(&other.words.0) {
            Less => Less,
            Greater => Greater,
            Equal => self.words.1.cmp(&other.words.1),
        }
    }
}

/*
fn add(a: i64, b: i64, mut d: i64) -> i64 {unsafe{
    let c: i64;
    asm!("mulx %3, %2, %0
        movl %rdx, %1"
        : "=r"(c), "=r"(d)
        : "r"(a), "r"(b)
        : "%rdx"
    );
    c
}}
*/

impl PartialEq for f8_120 {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Equal
    }
}

impl Eq for f8_120 {}

impl PartialOrd for f8_120 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for f8_120 {
    fn cmp(&self, other: &Self) -> Ordering {
        let scmp = self.sign.cmp(&other.sign);
        if scmp != Equal { return scmp; }

        match self.sign {
            Neutral  => Equal,
            Positive  => {
                self.words_cmp(&other)
            },
            Negative => {
                other.words_cmp(&self)
            },
        }
    }
}

impl Default for f8_120 {
    fn default() -> Self {
        Self::zero()
    }
}

impl Zero for f8_120 {
    fn zero() -> Self {
        f8_120::new((0, 0), Neutral)
    }
}

impl One for f8_120 {
    fn one() -> Self {
        f8_120::new((1 << (64 - 8), 0), Positive)
    }
}

impl Neg for f8_120{
    type Output = Self;
    fn neg(self) -> Self::Output {
        f8_120::new(self.words, -self.sign)
    }
}



fn add_u128((mut a1, mut a2): (u64, u64), (b1, b2): (u64, u64)) -> (u64, u64){
    unsafe{
        asm!("
            add $1, $5
            adc $0, $4
            "
            : "=r"(a1), "=r"(a2)
            : "0"(a1), "1"(a2), "r"(b1), "r"(b2)
            : "cc"
            : "intel"
        )
    }
    (a1, a2)
}

fn sub_u128((mut a1, mut a2): (u64, u64), (b1, b2): (u64, u64)) -> (u64, u64){
    unsafe{
        asm!("
            sub $1, $5
            sbb $0, $4
            "
            : "=r"(a1), "=r"(a2)
            : "0"(a1), "1"(a2), "r"(b1), "r"(b2)
            : "cc"
            : "intel"
        )
    }
    (a1, a2)
}

impl Add for f8_120{
    type Output = Self;
    fn add(self, other: Self) -> Self::Output{
        if self.sign == Neutral {return other;}
        if other.sign == Neutral {return self;}

        if self.sign == other.sign {
            Self::new(add_u128(self.words, other.words), self.sign)
        }else{
            match self.words_cmp(&other){
                Equal => Self::zero(),
                Greater => Self::new(sub_u128(self.words, other.words), self.sign),
                Less => Self::new(sub_u128(other.words, self.words), other.sign),
            }
        }
    }
}

impl Sub for f8_120{
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output{
        if self.sign == Neutral {return -other;}
        if other.sign == Neutral {return self;}

        if self.sign != other.sign {
            Self::new(add_u128(self.words, other.words), self.sign)
        }else{
            match self.words_cmp(&other){
                Equal => Self::zero(),
                Greater => Self::new(sub_u128(self.words, other.words), self.sign),
                Less => Self::new(sub_u128(other.words, self.words), -self.sign),
            }
        }
    }
}

// fn full_mul(a: u64, b: u64) -> (u64, u64) {unsafe{
//     let c: u64;
//     let d: u64;
//     asm!("mulx $0, $1, $2"
//         :"=r"(c), "=r"(d)
//         : "r"(a), "{rdx}"(b)
//         :
//         : "intel"
//     );
//     (c, d)
// }}

fn mul_u128((a1, a2): (u64, u64), (b1, b2): (u64, u64)) -> (u64, u64) {unsafe{
    let  c: u64;
    let  d: u64;

    asm!("
        mov rdx, $2
        mulx r8, r9, $4
        mulx r10, r11, $5
        mov rdx, $3
        mulx r12, r13, $4
        mulx r14, r14, $5

        add r11, r13
        adc r9, r10
        adc r8, 0
        add r11, r14
        adc r9, r12
        adc r8, 0

        mov r10, r9
        shl r8, 8
        shr r9, 56
        shl r10, 8
        shr r11, 56

        mov $0, r8
        mov $1, r10
        add $0, r9
        add $1, r11
        "
        : "=r"(c), "=r"(d)
        : "r"(a1), "r"(a2), "r"(b1), "r"(b2)
        : "cc", "r8", "r9", "r10", "r11", "r12", "r13", "r14", "rdx"
        : "intel"
    );
    (c, d)
}}

impl Mul for f8_120{
    type Output = Self;
    fn mul(self, other: Self) -> Self::Output{
        if self.sign == Neutral || other.sign == Neutral {
            return Self::zero();
        }

        // let hh = full_mul(self.words.0, other.words.0);
        // let hl = full_mul(self.words.0, other.words.1);
        // let lh = full_mul(self.words.1, other.words.0);
        // let ll = full_mul(self.words.1, other.words.1);
        //
        // let (o3, c1) = ll.0.overflowing_add(lh.1);
        // let (o3, c2) = o3.overflowing_add(hl.1);
        //
        // let (o2, c1) = lh.0.overflowing_add(c1 as u64 + c2 as u64);
        // let (o2, c2) = o2.overflowing_add(hl.0);
        // let (o2, c3) = o2.overflowing_add(hh.1);
        //
        // let o1 = hh.0 + c1 as u64 + c2 as u64 + c3 as u64;
        //
        // let words = ((o1 << 8) + (o2 >> 56), (o2 << 8) + (o3 >> 56));

        let words = mul_u128(self.words, other.words);

        if words.0 == 0 && words.1 == 0 {
            Self::zero()
        }else{
            Self::new(words, if self.sign == other.sign {Positive} else {Negative})
        }
    }
}

impl From<f32> for f8_120 {
    fn from(val: f32) -> Self{
        let (mantissa, exponent, sign) = val.integer_decode();
        let word1 = safe_shr(mantissa, 8-64-exponent);
        let word2 = safe_shr(mantissa, 8-128-exponent);
        if word1 == 0 && word2 == 0 {
            Self::new((word1, word2), Neutral)
        }else{
            Self::new((word1, word2), Sign::from(sign))
        }
    }
}

impl From<f64> for f8_120 {
    fn from(val: f64) -> Self{
        let (mantissa, exponent, sign) = val.integer_decode();
        let word1 = safe_shr(mantissa, 8-64-exponent);
        let word2 = safe_shr(mantissa, 8-128-exponent);
        if word1 == 0 && word2 == 0 {
            Self::new((word1, word2), Neutral)
        }else{
            Self::new((word1, word2), Sign::from(sign))
        }
    }
}

impl From<u8> for f8_120 {
    fn from(val: u8) -> Self{
        Self::new(((val as u64) << 56, 0), Sign::from(val))
    }
}

impl From<f8_120> for f32{
    fn from(val: f8_120) -> Self{
        let mut f = 0.0;
        f += val.words.0 as f32 * 2_f32.powi(8-64);
        f += val.words.1 as f32 * 2_f32.powi(8-128);
        f *= f32::from(val.sign);
        f
    }
}

impl From<f8_120> for f64{
    fn from(val: f8_120) -> Self{
        let mut f = 0.0;
        f += val.words.0 as f64 * 2_f64.powi(8-64);
        f += val.words.1 as f64 * 2_f64.powi(8-128);
        f *= f64::from(val.sign);
        f
    }
}

impl From<f8_120> for i8 {
    fn from(val: f8_120) -> Self{
        if val.sign == Negative {
            -((val.words.0 >> 56) as i8)
        }else{
            (val.words.0 >> 56) as i8
        }
    }
}


impl fmt::Display for f8_120 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", f64::from(self.clone()))
    }
}

fn print_full_f32(val: f32){
    let (mantissa, exponent, sign) = val.integer_decode();
    println!("{}{:08b}.{:056b}_{:064b}", Sign::from(sign), safe_shr(mantissa, -exponent) as u8, safe_shr(mantissa, 8-64-exponent) % (1 << 56), safe_shr(mantissa, 8-128-exponent));
}

fn print_full_f64(val: f64){
    let (mantissa, exponent, sign) = val.integer_decode();
    println!("{}{:08b}.{:056b}_{:064b}", Sign::from(sign), safe_shr(mantissa, -exponent) as u8, safe_shr(mantissa, 8-64-exponent) % (1 << 56), safe_shr(mantissa, 8-128-exponent));
}

fn print_full_f8_120(val: f8_120){
    println!("{}{:08b}.{:056b}_{:064b}", val.sign, val.words.0 / (1 << 56), val.words.0 % (1 << 56), val.words.1);
}

#[test]
fn test_conversion(){

    let test32 = |mut val: f32| {
        for _ in 0..128{
            println!("\nValue: {}", val);
            print_full_f32(val);
            let fp = f8_120::from(val);
            print_full_f8_120(fp);
            let f = f32::from(fp);
            print_full_f32(f);
            assert_eq!(f, val);
            val /= 2.0;
            val = (val * 2f32.powi(120)).trunc() / 2f32.powi(120);
        }
    };
    let test64 = |mut val: f64| {
        for _ in 0..128{
            println!("\nValue: {}", val);
            print_full_f64(val);
            let fp = f8_120::from(val);
            print_full_f8_120(fp);
            let f = f64::from(fp);
            print_full_f64(f);
            assert_eq!(f, val);
            val /= 2.0;
            val = (val * 2f64.powi(120)).trunc() / 2f64.powi(120);
        }
    };

    test32(1.0);
    test64(1.0);
    test32(128.0);
    test64(128.0);
    unsafe{
        test32(mem::transmute(0b0_10000110_11111111111111111111111_u32));
        test64(mem::transmute(0b0_10000000110_1111111111111111111111111111111100000000000000000000_u64));
        test64(mem::transmute(0b0_10000000110_1111111111111111111111111111111111111111111111111111_u64));
        test64(mem::transmute(0b0_10000000110_0101010101010101010101010101010101010101010101010101_u64));
        test64(mem::transmute(0b0_10000000110_0110011100011110000111110000011111100000011111110000_u64));
    };

    test32(-1.0);
    test64(-1.0);
    test32(-128.0);
    test64(-128.0);
    unsafe{
        test32(-mem::transmute::<_, f32>(0b0_10000110_11111111111111111111111_u32));
        test64(-mem::transmute::<_, f64>(0b0_10000000110_1111111111111111111111111111111100000000000000000000_u64));
        test64(-mem::transmute::<_, f64>(0b0_10000000110_1111111111111111111111111111111111111111111111111111_u64));
        test64(-mem::transmute::<_, f64>(0b0_10000000110_0101010101010101010101010101010101010101010101010101_u64));
        test64(-mem::transmute::<_, f64>(0b0_10000000110_0110011100011110000111110000011111100000011111110000_u64));
    };
}

#[test]
fn test_cmp(){
    use std::cmp::Ordering;

    let w1 = [0, 1, 5];
    let w2 = [0x0000000000000000, 0x8000000000000000, 0xC000000000000000];
    let mut fix = Vec::new();
    let mut float = Vec::new();
    for a in w1.iter().cloned() {
        for b in w2.iter().cloned() {
            if a == 0 && b == 0{
                let val = f8_120::new((a, b), Neutral);
                fix.push(val);
                float.push(f32::from(val));
            }else{
                let val = f8_120::new((a, b), Positive);
                fix.push(val);
                float.push(f32::from(val));
                let val = f8_120::new((a, b), Negative);
                fix.push(val);
                float.push(f32::from(val));
            }
        }
    }

    for (a1, b1) in fix.iter().cloned().zip(float.iter().cloned()) {
        for (a2, b2) in fix.iter().cloned().zip(float.iter().cloned()) {
            assert!(a1.cmp(&a2) == b1.partial_cmp(&b2).unwrap());
        }
    }
}

#[test]
fn test_add_sub(){
    use std::cmp::Ordering;

    let w1 = [0, 1, 5];
    let w2 = [0x0000000000000000, 0x8000000000000000, 0xC000000000000000];
    let mut fix = Vec::new();
    let mut float = Vec::new();
    for a in w1.iter().cloned() {
        for b in w2.iter().cloned() {
            if a == 0 && b == 0{
                let val = f8_120::new((a, b), Neutral);
                fix.push(val);
                float.push(f32::from(val));
            }else{
                let val = f8_120::new((a, b), Positive);
                fix.push(val);
                float.push(f32::from(val));
                let val = f8_120::new((a, b), Negative);
                fix.push(val);
                float.push(f32::from(val));
            }
        }
    }

    for (a1, b1) in fix.iter().cloned().zip(float.iter().cloned()) {
        for (a2, b2) in fix.iter().cloned().zip(float.iter().cloned()) {
            println!("\n\n{} + {} = {}", b1, b2, b1+b2);
            print_full_f8_120(a1);
            println!(" {}", (0..130).map(|_| "+").collect::<String>());
            print_full_f8_120(a2);
            println!(" {}", (0..130).map(|_| "=").collect::<String>());
            print_full_f8_120(a1 + a2);
            println!("should be:");
            print_full_f32(b1 + b2);
            assert!(f32::from(a1 + a2) == b1 + b2);

            println!("\n{} - {} = {}", b1, b2, b1-b2);
            print_full_f8_120(a1);
            println!(" {}", (0..130).map(|_| "-").collect::<String>());
            print_full_f8_120(a2);
            println!(" {}", (0..130).map(|_| "=").collect::<String>());
            print_full_f8_120(a1 - a2);
            println!("should be:");
            print_full_f32(b1 - b2);
            assert!(f32::from(a1 - a2) == b1 - b2);
        }
    }
}

#[test]
fn test_mul(){
    let f: Vec<f64> = vec![1.0, 10.0, 127.0, 170.0, 85.0];
    let mut floats = Vec::new();
    for v in f {
        floats.push(v);
        floats.push(-v);
        floats.push(v * 2f64.powi(-1));
        floats.push(v * 2f64.powi(-60));
        floats.push(v * 2f64.powi(-64));
        floats.push(v * 2f64.powi(-100));
        floats.push(v * 2f64.powi(-120));
    }

    let cut = |mut x: f64| {
        x = (x * 2f64.powi(120)).trunc() / 2f64.powi(120);
        x = (x / 2f64.powi(8)).fract() * 2f64.powi(8);
        x
    };

    let mut floats: Vec<f64> = floats.iter().map(|&x| cut(x)).filter(|&x| x.is_normal()).collect();
    floats.push(0.0);
    println!("{:?}", floats);


    for f1 in floats.iter().cloned() {
        for f2 in floats.iter().cloned() {
            println!("\n\n{} * {} = {}", f1, f2, f1*f2);
            let fix1 = f8_120::from(f1);
            let fix2 = f8_120::from(f2);
            let fix3 = fix1 * fix2;
            let f3 = f8_120::from(f1*f2);
            print_full_f8_120(fix1);
            println!(" {}", (0..130).map(|_| "*").collect::<String>());
            print_full_f8_120(fix2);
            println!(" {}", (0..130).map(|_| "=").collect::<String>());
            print_full_f8_120(fix3);
            println!("should be:");
            print_full_f8_120(f3);
            assert!(fix3 == f3);
        }
    }
}

extern crate test;
use self::test::Bencher;

macro_rules! times100{
    ($a:expr) => {{
        $a;$a;$a;$a;$a;$a;$a;$a;$a;$a;
        $a;$a;$a;$a;$a;$a;$a;$a;$a;$a;
        $a;$a;$a;$a;$a;$a;$a;$a;$a;$a;
        $a;$a;$a;$a;$a;$a;$a;$a;$a;$a;
        $a;$a;$a;$a;$a;$a;$a;$a;$a;$a;
        $a;$a;$a;$a;$a;$a;$a;$a;$a;$a;
        $a;$a;$a;$a;$a;$a;$a;$a;$a;$a;
        $a;$a;$a;$a;$a;$a;$a;$a;$a;$a;
        $a;$a;$a;$a;$a;$a;$a;$a;$a;$a;
        $a;$a;$a;$a;$a;$a;$a;$a;$a;$a;
    }}
}

#[bench]
fn bench_mul(bench: &mut Bencher) {
    let a = f8_120::from(0.001234);
    let b = f8_120::from(7.0);
    bench.iter(|| times100!(a * b));
}

#[bench]
fn bench_add(bench: &mut Bencher) {
    let a = f8_120::from(0.001234);
    let b = f8_120::from(7.0);
    bench.iter(|| times100!(a + b));
}
