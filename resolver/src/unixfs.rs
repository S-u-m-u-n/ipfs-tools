// This file is generated by rust-protobuf 2.25.2. Do not edit
// @generated

// https://github.com/rust-lang/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![allow(unused_attributes)]
#![cfg_attr(rustfmt, rustfmt::skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unused_imports)]
#![allow(unused_results)]
//! Generated file from `unixfs.proto`

/// Generated files are compatible only with the same version
/// of protobuf runtime.
// const _PROTOBUF_VERSION_CHECK: () = ::protobuf::VERSION_2_25_2;

#[derive(PartialEq,Clone,Default)]
pub struct Data {
    // message fields
    Type: ::std::option::Option<Data_DataType>,
    Data: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    filesize: ::std::option::Option<u64>,
    pub blocksizes: ::std::vec::Vec<u64>,
    hashType: ::std::option::Option<u64>,
    fanout: ::std::option::Option<u64>,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a Data {
    fn default() -> &'a Data {
        <Data as ::protobuf::Message>::default_instance()
    }
}

impl Data {
    pub fn new() -> Data {
        ::std::default::Default::default()
    }

    // required .unixfs.pb.Data.DataType Type = 1;


    pub fn get_Type(&self) -> Data_DataType {
        self.Type.unwrap_or(Data_DataType::Raw)
    }
    pub fn clear_Type(&mut self) {
        self.Type = ::std::option::Option::None;
    }

    pub fn has_Type(&self) -> bool {
        self.Type.is_some()
    }

    // Param is passed by value, moved
    pub fn set_Type(&mut self, v: Data_DataType) {
        self.Type = ::std::option::Option::Some(v);
    }

    // optional bytes Data = 2;


    pub fn get_Data(&self) -> &[u8] {
        match self.Data.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }
    pub fn clear_Data(&mut self) {
        self.Data.clear();
    }

    pub fn has_Data(&self) -> bool {
        self.Data.is_some()
    }

    // Param is passed by value, moved
    pub fn set_Data(&mut self, v: ::std::vec::Vec<u8>) {
        self.Data = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_Data(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.Data.is_none() {
            self.Data.set_default();
        }
        self.Data.as_mut().unwrap()
    }

    // Take field
    pub fn take_Data(&mut self) -> ::std::vec::Vec<u8> {
        self.Data.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    // optional uint64 filesize = 3;


    pub fn get_filesize(&self) -> u64 {
        self.filesize.unwrap_or(0)
    }
    pub fn clear_filesize(&mut self) {
        self.filesize = ::std::option::Option::None;
    }

    pub fn has_filesize(&self) -> bool {
        self.filesize.is_some()
    }

    // Param is passed by value, moved
    pub fn set_filesize(&mut self, v: u64) {
        self.filesize = ::std::option::Option::Some(v);
    }

    // repeated uint64 blocksizes = 4;


    pub fn get_blocksizes(&self) -> &[u64] {
        &self.blocksizes
    }
    pub fn clear_blocksizes(&mut self) {
        self.blocksizes.clear();
    }

    // Param is passed by value, moved
    pub fn set_blocksizes(&mut self, v: ::std::vec::Vec<u64>) {
        self.blocksizes = v;
    }

    // Mutable pointer to the field.
    pub fn mut_blocksizes(&mut self) -> &mut ::std::vec::Vec<u64> {
        &mut self.blocksizes
    }

    // Take field
    pub fn take_blocksizes(&mut self) -> ::std::vec::Vec<u64> {
        ::std::mem::replace(&mut self.blocksizes, ::std::vec::Vec::new())
    }

    // optional uint64 hashType = 5;


    pub fn get_hashType(&self) -> u64 {
        self.hashType.unwrap_or(0)
    }
    pub fn clear_hashType(&mut self) {
        self.hashType = ::std::option::Option::None;
    }

    pub fn has_hashType(&self) -> bool {
        self.hashType.is_some()
    }

    // Param is passed by value, moved
    pub fn set_hashType(&mut self, v: u64) {
        self.hashType = ::std::option::Option::Some(v);
    }

    // optional uint64 fanout = 6;


    pub fn get_fanout(&self) -> u64 {
        self.fanout.unwrap_or(0)
    }
    pub fn clear_fanout(&mut self) {
        self.fanout = ::std::option::Option::None;
    }

    pub fn has_fanout(&self) -> bool {
        self.fanout.is_some()
    }

    // Param is passed by value, moved
    pub fn set_fanout(&mut self, v: u64) {
        self.fanout = ::std::option::Option::Some(v);
    }
}

impl ::protobuf::Message for Data {
    fn is_initialized(&self) -> bool {
        if self.Type.is_none() {
            return false;
        }
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_proto2_enum_with_unknown_fields_into(wire_type, is, &mut self.Type, 1, &mut self.unknown_fields)?
                },
                2 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.Data)?;
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint64()?;
                    self.filesize = ::std::option::Option::Some(tmp);
                },
                4 => {
                    ::protobuf::rt::read_repeated_uint64_into(wire_type, is, &mut self.blocksizes)?;
                },
                5 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint64()?;
                    self.hashType = ::std::option::Option::Some(tmp);
                },
                6 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint64()?;
                    self.fanout = ::std::option::Option::Some(tmp);
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(v) = self.Type {
            my_size += ::protobuf::rt::enum_size(1, v);
        }
        if let Some(ref v) = self.Data.as_ref() {
            my_size += ::protobuf::rt::bytes_size(2, &v);
        }
        if let Some(v) = self.filesize {
            my_size += ::protobuf::rt::value_size(3, v, ::protobuf::wire_format::WireTypeVarint);
        }
        for value in &self.blocksizes {
            my_size += ::protobuf::rt::value_size(4, *value, ::protobuf::wire_format::WireTypeVarint);
        };
        if let Some(v) = self.hashType {
            my_size += ::protobuf::rt::value_size(5, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.fanout {
            my_size += ::protobuf::rt::value_size(6, v, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.Type {
            os.write_enum(1, ::protobuf::ProtobufEnum::value(&v))?;
        }
        if let Some(ref v) = self.Data.as_ref() {
            os.write_bytes(2, &v)?;
        }
        if let Some(v) = self.filesize {
            os.write_uint64(3, v)?;
        }
        for v in &self.blocksizes {
            os.write_uint64(4, *v)?;
        };
        if let Some(v) = self.hashType {
            os.write_uint64(5, v)?;
        }
        if let Some(v) = self.fanout {
            os.write_uint64(6, v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &dyn (::std::any::Any) {
        self as &dyn (::std::any::Any)
    }
    fn as_any_mut(&mut self) -> &mut dyn (::std::any::Any) {
        self as &mut dyn (::std::any::Any)
    }
    fn into_any(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn (::std::any::Any)> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> Data {
        Data::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            let mut fields = ::std::vec::Vec::new();
            fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeEnum<Data_DataType>>(
                "Type",
                |m: &Data| { &m.Type },
                |m: &mut Data| { &mut m.Type },
            ));
            fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                "Data",
                |m: &Data| { &m.Data },
                |m: &mut Data| { &mut m.Data },
            ));
            fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint64>(
                "filesize",
                |m: &Data| { &m.filesize },
                |m: &mut Data| { &mut m.filesize },
            ));
            fields.push(::protobuf::reflect::accessor::make_vec_accessor::<_, ::protobuf::types::ProtobufTypeUint64>(
                "blocksizes",
                |m: &Data| { &m.blocksizes },
                |m: &mut Data| { &mut m.blocksizes },
            ));
            fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint64>(
                "hashType",
                |m: &Data| { &m.hashType },
                |m: &mut Data| { &mut m.hashType },
            ));
            fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint64>(
                "fanout",
                |m: &Data| { &m.fanout },
                |m: &mut Data| { &mut m.fanout },
            ));
            ::protobuf::reflect::MessageDescriptor::new_pb_name::<Data>(
                "Data",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static Data {
        static instance: ::protobuf::rt::LazyV2<Data> = ::protobuf::rt::LazyV2::INIT;
        instance.get(Data::new)
    }
}

impl ::protobuf::Clear for Data {
    fn clear(&mut self) {
        self.Type = ::std::option::Option::None;
        self.Data.clear();
        self.filesize = ::std::option::Option::None;
        self.blocksizes.clear();
        self.hashType = ::std::option::Option::None;
        self.fanout = ::std::option::Option::None;
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Data {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Data {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Message(self)
    }
}

#[derive(Clone,PartialEq,Eq,Debug,Hash)]
pub enum Data_DataType {
    Raw = 0,
    Directory = 1,
    File = 2,
    Metadata = 3,
    Symlink = 4,
    HAMTShard = 5,
}

impl ::protobuf::ProtobufEnum for Data_DataType {
    fn value(&self) -> i32 {
        *self as i32
    }

    fn from_i32(value: i32) -> ::std::option::Option<Data_DataType> {
        match value {
            0 => ::std::option::Option::Some(Data_DataType::Raw),
            1 => ::std::option::Option::Some(Data_DataType::Directory),
            2 => ::std::option::Option::Some(Data_DataType::File),
            3 => ::std::option::Option::Some(Data_DataType::Metadata),
            4 => ::std::option::Option::Some(Data_DataType::Symlink),
            5 => ::std::option::Option::Some(Data_DataType::HAMTShard),
            _ => ::std::option::Option::None
        }
    }

    fn values() -> &'static [Self] {
        static values: &'static [Data_DataType] = &[
            Data_DataType::Raw,
            Data_DataType::Directory,
            Data_DataType::File,
            Data_DataType::Metadata,
            Data_DataType::Symlink,
            Data_DataType::HAMTShard,
        ];
        values
    }

    fn enum_descriptor_static() -> &'static ::protobuf::reflect::EnumDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::EnumDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            ::protobuf::reflect::EnumDescriptor::new_pb_name::<Data_DataType>("Data.DataType", file_descriptor_proto())
        })
    }
}

impl ::std::marker::Copy for Data_DataType {
}

impl ::std::default::Default for Data_DataType {
    fn default() -> Self {
        Data_DataType::Raw
    }
}

impl ::protobuf::reflect::ProtobufValue for Data_DataType {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Enum(::protobuf::ProtobufEnum::descriptor(self))
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct Metadata {
    // message fields
    MimeType: ::protobuf::SingularField<::std::string::String>,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a Metadata {
    fn default() -> &'a Metadata {
        <Metadata as ::protobuf::Message>::default_instance()
    }
}

impl Metadata {
    pub fn new() -> Metadata {
        ::std::default::Default::default()
    }

    // optional string MimeType = 1;


    pub fn get_MimeType(&self) -> &str {
        match self.MimeType.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }
    pub fn clear_MimeType(&mut self) {
        self.MimeType.clear();
    }

    pub fn has_MimeType(&self) -> bool {
        self.MimeType.is_some()
    }

    // Param is passed by value, moved
    pub fn set_MimeType(&mut self, v: ::std::string::String) {
        self.MimeType = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_MimeType(&mut self) -> &mut ::std::string::String {
        if self.MimeType.is_none() {
            self.MimeType.set_default();
        }
        self.MimeType.as_mut().unwrap()
    }

    // Take field
    pub fn take_MimeType(&mut self) -> ::std::string::String {
        self.MimeType.take().unwrap_or_else(|| ::std::string::String::new())
    }
}

impl ::protobuf::Message for Metadata {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.MimeType)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.MimeType.as_ref() {
            my_size += ::protobuf::rt::string_size(1, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.MimeType.as_ref() {
            os.write_string(1, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &dyn (::std::any::Any) {
        self as &dyn (::std::any::Any)
    }
    fn as_any_mut(&mut self) -> &mut dyn (::std::any::Any) {
        self as &mut dyn (::std::any::Any)
    }
    fn into_any(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn (::std::any::Any)> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> Metadata {
        Metadata::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            let mut fields = ::std::vec::Vec::new();
            fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                "MimeType",
                |m: &Metadata| { &m.MimeType },
                |m: &mut Metadata| { &mut m.MimeType },
            ));
            ::protobuf::reflect::MessageDescriptor::new_pb_name::<Metadata>(
                "Metadata",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static Metadata {
        static instance: ::protobuf::rt::LazyV2<Metadata> = ::protobuf::rt::LazyV2::INIT;
        instance.get(Metadata::new)
    }
}

impl ::protobuf::Clear for Metadata {
    fn clear(&mut self) {
        self.MimeType.clear();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Metadata {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Metadata {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Message(self)
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x0cunixfs.proto\x12\tunixfs.pb\"\xa0\x02\n\x04Data\x12.\n\x04Type\x18\
    \x01\x20\x02(\x0e2\x18.unixfs.pb.Data.DataTypeR\x04TypeB\0\x12\x14\n\x04\
    Data\x18\x02\x20\x01(\x0cR\x04DataB\0\x12\x1c\n\x08filesize\x18\x03\x20\
    \x01(\x04R\x08filesizeB\0\x12\x20\n\nblocksizes\x18\x04\x20\x03(\x04R\nb\
    locksizesB\0\x12\x1c\n\x08hashType\x18\x05\x20\x01(\x04R\x08hashTypeB\0\
    \x12\x18\n\x06fanout\x18\x06\x20\x01(\x04R\x06fanoutB\0\"X\n\x08DataType\
    \x12\x07\n\x03Raw\x10\0\x12\r\n\tDirectory\x10\x01\x12\x08\n\x04File\x10\
    \x02\x12\x0c\n\x08Metadata\x10\x03\x12\x0b\n\x07Symlink\x10\x04\x12\r\n\
    \tHAMTShard\x10\x05\x1a\0:\0\"*\n\x08Metadata\x12\x1c\n\x08MimeType\x18\
    \x01\x20\x01(\tR\x08MimeTypeB\0:\0B\0b\x06proto2\
";

static file_descriptor_proto_lazy: ::protobuf::rt::LazyV2<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::rt::LazyV2::INIT;

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::Message::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    file_descriptor_proto_lazy.get(|| {
        parse_descriptor_proto()
    })
}
