#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[expect(non_camel_case_types)]
#[allow(dead_code)] // Some variants are only used with certain features.
#[allow(clippy::upper_case_acronyms)]
pub(crate) enum INS {
    GET_PUBLIC_KEY = 0x02,
    SIGN = 0x04,
    GET_APP_CONFIGURATION = 0x06,
    SIGN_PERSONAL_MESSAGE = 0x08,
    SIGN_ETH_EIP_712 = 0x0C,
    SEND_EIP712_STRUCT_DEFINITION = 0x1A,
    SEND_EIP712_STRUCT_IMPLEMENTATION = 0x1C,
    SIGN_EIP7702_AUTHORIZATION = 0x34,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[expect(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
pub(crate) enum P1 {
    NON_CONFIRM = 0x00,
    /// Partial send, more to come
    PARTIAL = 0x01,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[expect(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
pub(crate) enum P2 {
    NO_CHAINCODE = 0x00,
    /// EIP-712 full implementation (struct definitions + implementations)
    IMPL_FULL = 0x01,
}

/// P2 values for EIP-712 struct definition commands (INS 0x1A)
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub(crate) enum P2Definition {
    /// Send struct name
    NAME = 0x00,
    /// Send struct field definition
    FIELD = 0xFF,
}

/// P2 values for EIP-712 struct implementation commands (INS 0x1C)
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub(crate) enum P2Implementation {
    /// Set root struct
    ROOT = 0x00,
    /// Send array size
    ARRAY = 0x0F,
    /// Send struct field value
    FIELD = 0xFF,
}

/// EIP-712 type descriptor bits and type values.
/// TypeDesc byte layout (MSB to LSB):
///   bit 7: TypeArray (is it an array?)
///   bit 6: TypeSize (is a type size specified?)
///   bits 5-4: unused
///   bits 3-0: type enum
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EIP712Type {
    Custom = 0,
    Int = 1,
    Uint = 2,
    Address = 3,
    Bool = 4,
    String = 5,
    FixedBytes = 6,
    DynamicBytes = 7,
}

pub(crate) const TYPE_ARRAY_BIT: u8 = 0x80;
pub(crate) const TYPE_SIZE_BIT: u8 = 0x40;
