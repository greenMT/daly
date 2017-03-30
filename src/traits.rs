
pub mod vec {
    pub trait ConvertingStack<T> {
        fn pop_into<U>(&mut self) -> U where U: From<T>;

        fn pop_2_into<U>(&mut self) -> (U, U) where U: From<T>;

        fn push_from<U: Into<T>>(&mut self, val: U);
    }

    impl<T> ConvertingStack<T> for Vec<T> {
        fn pop_into<U>(&mut self) -> U
            where U: From<T>
        {
            self.pop().unwrap().into()
        }

        fn pop_2_into<U>(&mut self) -> (U, U)
            where U: From<T>
        {
            (self.pop().unwrap().into(), self.pop().unwrap().into())
        }

        fn push_from<U: Into<T>>(&mut self, val: U) {
            self.push(val.into());
        }
    }
}
