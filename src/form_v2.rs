//! FC0-FC1 typed variational-form artifact boundary.

include!("form_v2/types_01.rs");
include!("form_v2/types_02.rs");
include!("form_v2/types_03.rs");
include!("form_v2/types_04.rs");
include!("form_v2/validation_01.rs");
include!("form_v2/validation_02.rs");
include!("form_v2/validation_03.rs");
include!("form_v2/validation_04.rs");
include!("form_v2/adapter.rs");

#[cfg(test)]
mod tests {
    use super::*;

    include!("form_v2/tests_01.rs");
    include!("form_v2/tests_02.rs");
    include!("form_v2/tests_03.rs");
    include!("form_v2/tests_04.rs");
    include!("form_v2/tests_05.rs");
    include!("form_v2/tests_06.rs");
}
