// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Timestamp {
    /// Represents seconds of UTC time since Unix epoch
    /// 1970-01-01T00:00:00Z. Must be from 0001-01-01T00:00:00Z to
    /// 9999-12-31T23:59:59Z inclusive.
    #[prost(int64, tag="1")]
    pub seconds: i64,
    /// Non-negative fractions of a second at nanosecond resolution. Negative
    /// second values with fractions must still have non-negative nanos values
    /// that count forward in time. Must be from 0 to 999,999,999
    /// inclusive.
    #[prost(int32, tag="2")]
    pub nanos: i32,
}
/// Encoded file descriptor set for the `aptos.util.timestamp` package
pub const FILE_DESCRIPTOR_SET: &[u8] = &[
    0x0a, 0xf7, 0x06, 0x0a, 0x24, 0x61, 0x70, 0x74, 0x6f, 0x73, 0x2f, 0x75, 0x74, 0x69, 0x6c, 0x2f,
    0x74, 0x69, 0x6d, 0x65, 0x73, 0x74, 0x61, 0x6d, 0x70, 0x2f, 0x74, 0x69, 0x6d, 0x65, 0x73, 0x74,
    0x61, 0x6d, 0x70, 0x2e, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x12, 0x14, 0x61, 0x70, 0x74, 0x6f, 0x73,
    0x2e, 0x75, 0x74, 0x69, 0x6c, 0x2e, 0x74, 0x69, 0x6d, 0x65, 0x73, 0x74, 0x61, 0x6d, 0x70, 0x22,
    0x3b, 0x0a, 0x09, 0x54, 0x69, 0x6d, 0x65, 0x73, 0x74, 0x61, 0x6d, 0x70, 0x12, 0x18, 0x0a, 0x07,
    0x73, 0x65, 0x63, 0x6f, 0x6e, 0x64, 0x73, 0x18, 0x01, 0x20, 0x01, 0x28, 0x03, 0x52, 0x07, 0x73,
    0x65, 0x63, 0x6f, 0x6e, 0x64, 0x73, 0x12, 0x14, 0x0a, 0x05, 0x6e, 0x61, 0x6e, 0x6f, 0x73, 0x18,
    0x02, 0x20, 0x01, 0x28, 0x05, 0x52, 0x05, 0x6e, 0x61, 0x6e, 0x6f, 0x73, 0x42, 0x9c, 0x01, 0x0a,
    0x18, 0x63, 0x6f, 0x6d, 0x2e, 0x61, 0x70, 0x74, 0x6f, 0x73, 0x2e, 0x75, 0x74, 0x69, 0x6c, 0x2e,
    0x74, 0x69, 0x6d, 0x65, 0x73, 0x74, 0x61, 0x6d, 0x70, 0x42, 0x0e, 0x54, 0x69, 0x6d, 0x65, 0x73,
    0x74, 0x61, 0x6d, 0x70, 0x50, 0x72, 0x6f, 0x74, 0x6f, 0x50, 0x01, 0xa2, 0x02, 0x03, 0x41, 0x55,
    0x54, 0xaa, 0x02, 0x14, 0x41, 0x70, 0x74, 0x6f, 0x73, 0x2e, 0x55, 0x74, 0x69, 0x6c, 0x2e, 0x54,
    0x69, 0x6d, 0x65, 0x73, 0x74, 0x61, 0x6d, 0x70, 0xca, 0x02, 0x14, 0x41, 0x70, 0x74, 0x6f, 0x73,
    0x5c, 0x55, 0x74, 0x69, 0x6c, 0x5c, 0x54, 0x69, 0x6d, 0x65, 0x73, 0x74, 0x61, 0x6d, 0x70, 0xe2,
    0x02, 0x20, 0x41, 0x70, 0x74, 0x6f, 0x73, 0x5c, 0x55, 0x74, 0x69, 0x6c, 0x5c, 0x54, 0x69, 0x6d,
    0x65, 0x73, 0x74, 0x61, 0x6d, 0x70, 0x5c, 0x47, 0x50, 0x42, 0x4d, 0x65, 0x74, 0x61, 0x64, 0x61,
    0x74, 0x61, 0xea, 0x02, 0x16, 0x41, 0x70, 0x74, 0x6f, 0x73, 0x3a, 0x3a, 0x55, 0x74, 0x69, 0x6c,
    0x3a, 0x3a, 0x54, 0x69, 0x6d, 0x65, 0x73, 0x74, 0x61, 0x6d, 0x70, 0x4a, 0xd4, 0x04, 0x0a, 0x06,
    0x12, 0x04, 0x03, 0x00, 0x12, 0x01, 0x0a, 0x4e, 0x0a, 0x01, 0x0c, 0x12, 0x03, 0x03, 0x00, 0x12,
    0x32, 0x44, 0x20, 0x43, 0x6f, 0x70, 0x79, 0x72, 0x69, 0x67, 0x68, 0x74, 0x20, 0xc2, 0xa9, 0x20,
    0x41, 0x70, 0x74, 0x6f, 0x73, 0x20, 0x46, 0x6f, 0x75, 0x6e, 0x64, 0x61, 0x74, 0x69, 0x6f, 0x6e,
    0x0a, 0x20, 0x53, 0x50, 0x44, 0x58, 0x2d, 0x4c, 0x69, 0x63, 0x65, 0x6e, 0x73, 0x65, 0x2d, 0x49,
    0x64, 0x65, 0x6e, 0x74, 0x69, 0x66, 0x69, 0x65, 0x72, 0x3a, 0x20, 0x41, 0x70, 0x61, 0x63, 0x68,
    0x65, 0x2d, 0x32, 0x2e, 0x30, 0x0a, 0x0a, 0x08, 0x0a, 0x01, 0x02, 0x12, 0x03, 0x05, 0x00, 0x1d,
    0x0a, 0x0a, 0x0a, 0x02, 0x04, 0x00, 0x12, 0x04, 0x07, 0x00, 0x12, 0x01, 0x0a, 0x0a, 0x0a, 0x03,
    0x04, 0x00, 0x01, 0x12, 0x03, 0x07, 0x08, 0x11, 0x0a, 0x9c, 0x01, 0x0a, 0x04, 0x04, 0x00, 0x02,
    0x00, 0x12, 0x03, 0x0b, 0x02, 0x14, 0x1a, 0x8e, 0x01, 0x20, 0x52, 0x65, 0x70, 0x72, 0x65, 0x73,
    0x65, 0x6e, 0x74, 0x73, 0x20, 0x73, 0x65, 0x63, 0x6f, 0x6e, 0x64, 0x73, 0x20, 0x6f, 0x66, 0x20,
    0x55, 0x54, 0x43, 0x20, 0x74, 0x69, 0x6d, 0x65, 0x20, 0x73, 0x69, 0x6e, 0x63, 0x65, 0x20, 0x55,
    0x6e, 0x69, 0x78, 0x20, 0x65, 0x70, 0x6f, 0x63, 0x68, 0x0a, 0x20, 0x31, 0x39, 0x37, 0x30, 0x2d,
    0x30, 0x31, 0x2d, 0x30, 0x31, 0x54, 0x30, 0x30, 0x3a, 0x30, 0x30, 0x3a, 0x30, 0x30, 0x5a, 0x2e,
    0x20, 0x4d, 0x75, 0x73, 0x74, 0x20, 0x62, 0x65, 0x20, 0x66, 0x72, 0x6f, 0x6d, 0x20, 0x30, 0x30,
    0x30, 0x31, 0x2d, 0x30, 0x31, 0x2d, 0x30, 0x31, 0x54, 0x30, 0x30, 0x3a, 0x30, 0x30, 0x3a, 0x30,
    0x30, 0x5a, 0x20, 0x74, 0x6f, 0x0a, 0x20, 0x39, 0x39, 0x39, 0x39, 0x2d, 0x31, 0x32, 0x2d, 0x33,
    0x31, 0x54, 0x32, 0x33, 0x3a, 0x35, 0x39, 0x3a, 0x35, 0x39, 0x5a, 0x20, 0x69, 0x6e, 0x63, 0x6c,
    0x75, 0x73, 0x69, 0x76, 0x65, 0x2e, 0x0a, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x05,
    0x12, 0x03, 0x0b, 0x02, 0x07, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x01, 0x12, 0x03,
    0x0b, 0x08, 0x0f, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x03, 0x12, 0x03, 0x0b, 0x12,
    0x13, 0x0a, 0xe4, 0x01, 0x0a, 0x04, 0x04, 0x00, 0x02, 0x01, 0x12, 0x03, 0x11, 0x02, 0x12, 0x1a,
    0xd6, 0x01, 0x20, 0x4e, 0x6f, 0x6e, 0x2d, 0x6e, 0x65, 0x67, 0x61, 0x74, 0x69, 0x76, 0x65, 0x20,
    0x66, 0x72, 0x61, 0x63, 0x74, 0x69, 0x6f, 0x6e, 0x73, 0x20, 0x6f, 0x66, 0x20, 0x61, 0x20, 0x73,
    0x65, 0x63, 0x6f, 0x6e, 0x64, 0x20, 0x61, 0x74, 0x20, 0x6e, 0x61, 0x6e, 0x6f, 0x73, 0x65, 0x63,
    0x6f, 0x6e, 0x64, 0x20, 0x72, 0x65, 0x73, 0x6f, 0x6c, 0x75, 0x74, 0x69, 0x6f, 0x6e, 0x2e, 0x20,
    0x4e, 0x65, 0x67, 0x61, 0x74, 0x69, 0x76, 0x65, 0x0a, 0x20, 0x73, 0x65, 0x63, 0x6f, 0x6e, 0x64,
    0x20, 0x76, 0x61, 0x6c, 0x75, 0x65, 0x73, 0x20, 0x77, 0x69, 0x74, 0x68, 0x20, 0x66, 0x72, 0x61,
    0x63, 0x74, 0x69, 0x6f, 0x6e, 0x73, 0x20, 0x6d, 0x75, 0x73, 0x74, 0x20, 0x73, 0x74, 0x69, 0x6c,
    0x6c, 0x20, 0x68, 0x61, 0x76, 0x65, 0x20, 0x6e, 0x6f, 0x6e, 0x2d, 0x6e, 0x65, 0x67, 0x61, 0x74,
    0x69, 0x76, 0x65, 0x20, 0x6e, 0x61, 0x6e, 0x6f, 0x73, 0x20, 0x76, 0x61, 0x6c, 0x75, 0x65, 0x73,
    0x0a, 0x20, 0x74, 0x68, 0x61, 0x74, 0x20, 0x63, 0x6f, 0x75, 0x6e, 0x74, 0x20, 0x66, 0x6f, 0x72,
    0x77, 0x61, 0x72, 0x64, 0x20, 0x69, 0x6e, 0x20, 0x74, 0x69, 0x6d, 0x65, 0x2e, 0x20, 0x4d, 0x75,
    0x73, 0x74, 0x20, 0x62, 0x65, 0x20, 0x66, 0x72, 0x6f, 0x6d, 0x20, 0x30, 0x20, 0x74, 0x6f, 0x20,
    0x39, 0x39, 0x39, 0x2c, 0x39, 0x39, 0x39, 0x2c, 0x39, 0x39, 0x39, 0x0a, 0x20, 0x69, 0x6e, 0x63,
    0x6c, 0x75, 0x73, 0x69, 0x76, 0x65, 0x2e, 0x0a, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x01,
    0x05, 0x12, 0x03, 0x11, 0x02, 0x07, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x01, 0x01, 0x12,
    0x03, 0x11, 0x08, 0x0d, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x01, 0x03, 0x12, 0x03, 0x11,
    0x10, 0x11, 0x62, 0x06, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x33,
];
include!("aptos.util.timestamp.serde.rs");
// @@protoc_insertion_point(module)
