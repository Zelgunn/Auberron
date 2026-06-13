use strum::VariantArray;

#[track_caller]
pub fn assert_enum_discriminants_self_index<T: VariantArray + Copy + Into<usize>>() {
    for (i, variant) in T::VARIANTS.iter().enumerate() {
        assert_eq!((*variant).into(), i);
    }
}
