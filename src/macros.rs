#[macro_export]
macro_rules! module {
    ($module:ident) => {
        static INSTANCE: $crate::__OnceCell<$module> = $crate::__OnceCell::new();

        impl $crate::Instance<$module> for $module {
            fn set(module: $module) {
                if let Err(_) = INSTANCE.set(module) {
                    panic!("cannot set instance");
                }
            }

            #[inline]
            fn get() -> Option<&'static $module> {
                INSTANCE.get()
            }
        }

        #[no_mangle]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn RedisModule_OnLoad(
            ctx: *mut $crate::rm::raw::RedisModuleCtx,
            argv: *mut *mut $crate::rm::raw::RedisModuleString,
            argc: std::os::raw::c_int,
        ) -> std::os::raw::c_int {
            $crate::on_load::<$module>(ctx, argv, argc) as std::os::raw::c_int
        }

        #[no_mangle]
        #[allow(non_snake_case)]
        pub extern "C" fn RedisModule_OnUnload(ctx: *mut $crate::rm::raw::RedisModuleCtx) -> std::os::raw::c_int {
            $crate::on_unload::<$module>(ctx) as std::os::raw::c_int
        }
    };
}
