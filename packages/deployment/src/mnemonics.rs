pub const DEFAULT_MNEMONIC: &str = "latin bleak rare super measure cannon defense zero van duck awkward sure super spring cattle post destroy rabbit open bronze ozone pelican taxi elevator";

const LOCAL_MNEMONIC_PARAM: &str = "LOCAL_MNEMONIC";
const TEST_MNEMONIC_PARAM: &str = "TEST_MNEMONIC";
const MAIN_MNEMONIC_PARAM: &str = "MAIN_MNEMONIC";

pub fn use_local_mnemonic(mnemonic: &str) {
    std::env::set_var(LOCAL_MNEMONIC_PARAM, mnemonic);
}

pub fn use_test_mnemonic(mnemonic: &str) {
    std::env::set_var(TEST_MNEMONIC_PARAM, mnemonic);
}

pub fn use_main_mnemonic(mnemonic: &str) {
    std::env::set_var(MAIN_MNEMONIC_PARAM, mnemonic);
}

pub fn use_mnemonic(mnemonic: &str) {
    use_local_mnemonic(mnemonic);
    use_test_mnemonic(mnemonic);
    use_main_mnemonic(mnemonic);
}
