#[macro_export]
macro_rules! module {
    ($module:ident) => {
        static INSTANCE: $crate::__OnceCell<$module> = $crate::__OnceCell::new();

        type __Instance = $crate::Instance<$module, __InstanceMngr>;

        struct __InstanceMngr;

        impl $crate::InstanceMngr<$module> for __InstanceMngr {
            #[inline]
            fn get() -> Option<&'static $module> {
                INSTANCE.get()
            }
            fn set(m: $module) {
                if let Err(_) = INSTANCE.set(m) {
                    panic!("cannot set instance");
                }
            }
        }

        #[no_mangle]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn RedisModule_OnLoad(
            ctx: *mut redis_module::raw::RedisModuleCtx,
            argv: *mut *mut redis_module::raw::RedisModuleString,
            argc: std::os::raw::c_int,
        ) -> std::os::raw::c_int {
            __Instance::on_load(ctx, argv, argc) as std::os::raw::c_int
        }

        #[no_mangle]
        #[allow(non_snake_case)]
        pub extern "C" fn RedisModule_OnUnload(
            ctx: *mut redis_module::raw::RedisModuleCtx,
        ) -> std::os::raw::c_int {
            __Instance::on_unload(ctx) as std::os::raw::c_int
        }
    };
}
