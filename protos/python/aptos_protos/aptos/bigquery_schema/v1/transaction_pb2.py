# -*- coding: utf-8 -*-
# Generated by the protocol buffer compiler.  DO NOT EDIT!
# source: aptos/bigquery_schema/v1/transaction.proto
"""Generated protocol buffer code."""
from google.protobuf import descriptor as _descriptor
from google.protobuf import descriptor_pool as _descriptor_pool
from google.protobuf import symbol_database as _symbol_database
from google.protobuf.internal import builder as _builder

# @@protoc_insertion_point(imports)

_sym_db = _symbol_database.Default()


DESCRIPTOR = _descriptor_pool.Default().AddSerializedFile(
    b'\n*aptos/bigquery_schema/v1/transaction.proto\x12$aptos.bigquery_schema.transaction.v1"\xe0\x02\n\x0bTransaction\x12\x0f\n\x07version\x18\x01 \x02(\x03\x12\x14\n\x0c\x62lock_height\x18\x02 \x02(\x03\x12\x0c\n\x04hash\x18\x03 \x02(\t\x12\x0c\n\x04type\x18\x04 \x02(\t\x12\x0f\n\x07payload\x18\x05 \x01(\t\x12\x19\n\x11state_change_hash\x18\x06 \x02(\t\x12\x17\n\x0f\x65vent_root_hash\x18\x07 \x02(\t\x12\x1d\n\x15state_checkpoint_hash\x18\x08 \x01(\t\x12\x10\n\x08gas_used\x18\t \x02(\x04\x12\x0f\n\x07success\x18\n \x02(\x08\x12\x11\n\tvm_status\x18\x0b \x02(\t\x12\x1d\n\x15\x61\x63\x63umulator_root_hash\x18\x0c \x02(\t\x12\x12\n\nnum_events\x18\r \x02(\x03\x12\x1d\n\x15num_write_set_changes\x18\x0e \x02(\x03\x12\r\n\x05\x65poch\x18\x0f \x02(\x03\x12\x13\n\x0binserted_at\x18\x10 \x02(\x03'
)

_globals = globals()
_builder.BuildMessageAndEnumDescriptors(DESCRIPTOR, _globals)
_builder.BuildTopDescriptorsAndMessages(
    DESCRIPTOR, "aptos.bigquery_schema.v1.transaction_pb2", _globals
)
if _descriptor._USE_C_DESCRIPTORS == False:
    DESCRIPTOR._options = None
    _globals["_TRANSACTION"]._serialized_start = 85
    _globals["_TRANSACTION"]._serialized_end = 437
# @@protoc_insertion_point(module_scope)
