use blanket_trait::blanket_trait;

pub trait A {
    type AA;
    fn a() -> i32;

    fn aa(&self) -> i32;
}

#[blanket_trait(impl<T: A> B for T)]
pub trait B {
    fn a(&self) -> i32 {
        T::a()
    }
}

#[blanket_trait(impl<T: A> C for T where T::AA: Send)]
pub trait C {
    fn a(&self) -> i32 {
        self.aa()
    }
}

#[blanket_trait(impl<T: A> D for T where T::AA: Send)]
pub trait D {
    type X = T::AA;
    fn a(&self) -> i32 {
        self.aa()
    }
}


pub trait X {
    fn a(&mut self) -> impl Future<Output = i32>;
}

#[blanket_trait(impl<T: X> Y for T)]
pub trait Y {
    fn b(&mut self) -> impl Future<Output = i32> {
        X::a(self)
    }
}
