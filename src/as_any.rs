pub trait AsAny {
    fn as_any<'a>(&'a self) -> &'a dyn std::any::Any;
    fn as_any_mut<'a>(&'a mut self) -> &'a mut dyn std::any::Any;
}

#[macro_export]
macro_rules! derive_as_any {
    ($name: ty) => {
        impl $crate::as_any::AsAny for $name {
            fn as_any<'a>(&'a self) -> &'a dyn std::any::Any {
                self
            }
            fn as_any_mut<'a>(&'a mut self) -> &'a mut dyn std::any::Any {
                self
            }
        }
    };
}

#[macro_export]
macro_rules! downcast_for {
    ($name: tt) => {
        #[allow(dead_code)]
        fn downcast<T: 'static + $name>(this: &dyn $name) -> Option<&T> {
            this.as_any().downcast_ref()
        }
        #[allow(dead_code)]
        fn downcast_mut<T: 'static + $name>(this: &mut dyn $name) -> Option<&mut T> {
            this.as_any_mut().downcast_mut()
        }
    };
    ($name: tt, $fn_name: tt, $fn_name_mut: tt) => {
        fn $fn_name<T: 'static + $name>(this: &dyn $name) -> Option<&T> {
            this.as_any().downcast_ref()
        }
        fn $fn_name_mut<T: 'static + $name>(this: &mut dyn $name) -> Option<&mut T> {
            this.as_any_mut().downcast_mut()
        }
    };
}