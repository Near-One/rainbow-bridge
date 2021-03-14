#[macro_export]
macro_rules! impl_admin_controlled {
    ($contract: ident, $paused: ident) => {
        use admin_controlled::{AdminControlled as AdminControlledInner, Mask as MaskInner};
        use near_sdk as near_sdk_inner;

        #[near_bindgen]
        impl AdminControlledInner for $contract {
            #[result_serializer(borsh)]
            fn get_paused(&self) -> MaskInner {
                self.$paused
            }

            #[result_serializer(borsh)]
            fn set_paused(&mut self, #[serializer(borsh)] paused: MaskInner) {
                near_sdk_inner::assert_self();
                self.$paused = paused;
            }
        }
    };
}
