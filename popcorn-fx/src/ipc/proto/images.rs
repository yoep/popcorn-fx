// This file is generated by rust-protobuf 3.7.1. Do not edit
// .proto file is parsed by protoc 29.3
// @generated

// https://github.com/rust-lang/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![allow(unused_attributes)]
#![cfg_attr(rustfmt, rustfmt::skip)]

#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unused_results)]
#![allow(unused_mut)]

//! Generated file from `images.proto`
// Generated for lite runtime

/// Generated files are compatible only with the same version
/// of protobuf runtime.
const _PROTOBUF_VERSION_CHECK: () = ::protobuf::VERSION_3_7_1;

// @@protoc_insertion_point(message:fx.ipc.proto.Image)
#[derive(PartialEq,Clone,Default,Debug)]
pub struct Image {
    // message fields
    // @@protoc_insertion_point(field:fx.ipc.proto.Image.data)
    pub data: ::std::vec::Vec<u8>,
    // special fields
    // @@protoc_insertion_point(special_field:fx.ipc.proto.Image.special_fields)
    pub special_fields: ::protobuf::SpecialFields,
}

impl<'a> ::std::default::Default for &'a Image {
    fn default() -> &'a Image {
        <Image as ::protobuf::Message>::default_instance()
    }
}

impl Image {
    pub fn new() -> Image {
        ::std::default::Default::default()
    }
}

impl ::protobuf::Message for Image {
    const NAME: &'static str = "Image";

    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::Result<()> {
        while let Some(tag) = is.read_raw_tag_or_eof()? {
            match tag {
                10 => {
                    self.data = is.read_bytes()?;
                },
                tag => {
                    ::protobuf::rt::read_unknown_or_skip_group(tag, is, self.special_fields.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u64 {
        let mut my_size = 0;
        if !self.data.is_empty() {
            my_size += ::protobuf::rt::bytes_size(1, &self.data);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.special_fields.unknown_fields());
        self.special_fields.cached_size().set(my_size as u32);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::Result<()> {
        if !self.data.is_empty() {
            os.write_bytes(1, &self.data)?;
        }
        os.write_unknown_fields(self.special_fields.unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn special_fields(&self) -> &::protobuf::SpecialFields {
        &self.special_fields
    }

    fn mut_special_fields(&mut self) -> &mut ::protobuf::SpecialFields {
        &mut self.special_fields
    }

    fn new() -> Image {
        Image::new()
    }

    fn clear(&mut self) {
        self.data.clear();
        self.special_fields.clear();
    }

    fn default_instance() -> &'static Image {
        static instance: Image = Image {
            data: ::std::vec::Vec::new(),
            special_fields: ::protobuf::SpecialFields::new(),
        };
        &instance
    }
}

/// Nested message and enums of message `Image`
pub mod image {
    #[derive(Clone,Copy,PartialEq,Eq,Debug,Hash)]
    // @@protoc_insertion_point(enum:fx.ipc.proto.Image.Error)
    pub enum Error {
        // @@protoc_insertion_point(enum_value:fx.ipc.proto.Image.Error.UNAVAILABLE)
        UNAVAILABLE = 0,
    }

    impl ::protobuf::Enum for Error {
        const NAME: &'static str = "Error";

        fn value(&self) -> i32 {
            *self as i32
        }

        fn from_i32(value: i32) -> ::std::option::Option<Error> {
            match value {
                0 => ::std::option::Option::Some(Error::UNAVAILABLE),
                _ => ::std::option::Option::None
            }
        }

        fn from_str(str: &str) -> ::std::option::Option<Error> {
            match str {
                "UNAVAILABLE" => ::std::option::Option::Some(Error::UNAVAILABLE),
                _ => ::std::option::Option::None
            }
        }

        const VALUES: &'static [Error] = &[
            Error::UNAVAILABLE,
        ];
    }

    impl ::std::default::Default for Error {
        fn default() -> Self {
            Error::UNAVAILABLE
        }
    }

}

// @@protoc_insertion_point(message:fx.ipc.proto.GetPosterPlaceholderRequest)
#[derive(PartialEq,Clone,Default,Debug)]
pub struct GetPosterPlaceholderRequest {
    // special fields
    // @@protoc_insertion_point(special_field:fx.ipc.proto.GetPosterPlaceholderRequest.special_fields)
    pub special_fields: ::protobuf::SpecialFields,
}

impl<'a> ::std::default::Default for &'a GetPosterPlaceholderRequest {
    fn default() -> &'a GetPosterPlaceholderRequest {
        <GetPosterPlaceholderRequest as ::protobuf::Message>::default_instance()
    }
}

impl GetPosterPlaceholderRequest {
    pub fn new() -> GetPosterPlaceholderRequest {
        ::std::default::Default::default()
    }
}

impl ::protobuf::Message for GetPosterPlaceholderRequest {
    const NAME: &'static str = "GetPosterPlaceholderRequest";

    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::Result<()> {
        while let Some(tag) = is.read_raw_tag_or_eof()? {
            match tag {
                tag => {
                    ::protobuf::rt::read_unknown_or_skip_group(tag, is, self.special_fields.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u64 {
        let mut my_size = 0;
        my_size += ::protobuf::rt::unknown_fields_size(self.special_fields.unknown_fields());
        self.special_fields.cached_size().set(my_size as u32);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::Result<()> {
        os.write_unknown_fields(self.special_fields.unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn special_fields(&self) -> &::protobuf::SpecialFields {
        &self.special_fields
    }

    fn mut_special_fields(&mut self) -> &mut ::protobuf::SpecialFields {
        &mut self.special_fields
    }

    fn new() -> GetPosterPlaceholderRequest {
        GetPosterPlaceholderRequest::new()
    }

    fn clear(&mut self) {
        self.special_fields.clear();
    }

    fn default_instance() -> &'static GetPosterPlaceholderRequest {
        static instance: GetPosterPlaceholderRequest = GetPosterPlaceholderRequest {
            special_fields: ::protobuf::SpecialFields::new(),
        };
        &instance
    }
}

// @@protoc_insertion_point(message:fx.ipc.proto.GetPosterPlaceholderResponse)
#[derive(PartialEq,Clone,Default,Debug)]
pub struct GetPosterPlaceholderResponse {
    // message fields
    // @@protoc_insertion_point(field:fx.ipc.proto.GetPosterPlaceholderResponse.image)
    pub image: ::protobuf::MessageField<Image>,
    // special fields
    // @@protoc_insertion_point(special_field:fx.ipc.proto.GetPosterPlaceholderResponse.special_fields)
    pub special_fields: ::protobuf::SpecialFields,
}

impl<'a> ::std::default::Default for &'a GetPosterPlaceholderResponse {
    fn default() -> &'a GetPosterPlaceholderResponse {
        <GetPosterPlaceholderResponse as ::protobuf::Message>::default_instance()
    }
}

impl GetPosterPlaceholderResponse {
    pub fn new() -> GetPosterPlaceholderResponse {
        ::std::default::Default::default()
    }
}

impl ::protobuf::Message for GetPosterPlaceholderResponse {
    const NAME: &'static str = "GetPosterPlaceholderResponse";

    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::Result<()> {
        while let Some(tag) = is.read_raw_tag_or_eof()? {
            match tag {
                10 => {
                    ::protobuf::rt::read_singular_message_into_field(is, &mut self.image)?;
                },
                tag => {
                    ::protobuf::rt::read_unknown_or_skip_group(tag, is, self.special_fields.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u64 {
        let mut my_size = 0;
        if let Some(v) = self.image.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint64_size(len) + len;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.special_fields.unknown_fields());
        self.special_fields.cached_size().set(my_size as u32);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::Result<()> {
        if let Some(v) = self.image.as_ref() {
            ::protobuf::rt::write_message_field_with_cached_size(1, v, os)?;
        }
        os.write_unknown_fields(self.special_fields.unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn special_fields(&self) -> &::protobuf::SpecialFields {
        &self.special_fields
    }

    fn mut_special_fields(&mut self) -> &mut ::protobuf::SpecialFields {
        &mut self.special_fields
    }

    fn new() -> GetPosterPlaceholderResponse {
        GetPosterPlaceholderResponse::new()
    }

    fn clear(&mut self) {
        self.image.clear();
        self.special_fields.clear();
    }

    fn default_instance() -> &'static GetPosterPlaceholderResponse {
        static instance: GetPosterPlaceholderResponse = GetPosterPlaceholderResponse {
            image: ::protobuf::MessageField::none(),
            special_fields: ::protobuf::SpecialFields::new(),
        };
        &instance
    }
}

// @@protoc_insertion_point(message:fx.ipc.proto.GetArtworkPlaceholderRequest)
#[derive(PartialEq,Clone,Default,Debug)]
pub struct GetArtworkPlaceholderRequest {
    // special fields
    // @@protoc_insertion_point(special_field:fx.ipc.proto.GetArtworkPlaceholderRequest.special_fields)
    pub special_fields: ::protobuf::SpecialFields,
}

impl<'a> ::std::default::Default for &'a GetArtworkPlaceholderRequest {
    fn default() -> &'a GetArtworkPlaceholderRequest {
        <GetArtworkPlaceholderRequest as ::protobuf::Message>::default_instance()
    }
}

impl GetArtworkPlaceholderRequest {
    pub fn new() -> GetArtworkPlaceholderRequest {
        ::std::default::Default::default()
    }
}

impl ::protobuf::Message for GetArtworkPlaceholderRequest {
    const NAME: &'static str = "GetArtworkPlaceholderRequest";

    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::Result<()> {
        while let Some(tag) = is.read_raw_tag_or_eof()? {
            match tag {
                tag => {
                    ::protobuf::rt::read_unknown_or_skip_group(tag, is, self.special_fields.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u64 {
        let mut my_size = 0;
        my_size += ::protobuf::rt::unknown_fields_size(self.special_fields.unknown_fields());
        self.special_fields.cached_size().set(my_size as u32);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::Result<()> {
        os.write_unknown_fields(self.special_fields.unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn special_fields(&self) -> &::protobuf::SpecialFields {
        &self.special_fields
    }

    fn mut_special_fields(&mut self) -> &mut ::protobuf::SpecialFields {
        &mut self.special_fields
    }

    fn new() -> GetArtworkPlaceholderRequest {
        GetArtworkPlaceholderRequest::new()
    }

    fn clear(&mut self) {
        self.special_fields.clear();
    }

    fn default_instance() -> &'static GetArtworkPlaceholderRequest {
        static instance: GetArtworkPlaceholderRequest = GetArtworkPlaceholderRequest {
            special_fields: ::protobuf::SpecialFields::new(),
        };
        &instance
    }
}

// @@protoc_insertion_point(message:fx.ipc.proto.GetArtworkPlaceholderResponse)
#[derive(PartialEq,Clone,Default,Debug)]
pub struct GetArtworkPlaceholderResponse {
    // message fields
    // @@protoc_insertion_point(field:fx.ipc.proto.GetArtworkPlaceholderResponse.image)
    pub image: ::protobuf::MessageField<Image>,
    // special fields
    // @@protoc_insertion_point(special_field:fx.ipc.proto.GetArtworkPlaceholderResponse.special_fields)
    pub special_fields: ::protobuf::SpecialFields,
}

impl<'a> ::std::default::Default for &'a GetArtworkPlaceholderResponse {
    fn default() -> &'a GetArtworkPlaceholderResponse {
        <GetArtworkPlaceholderResponse as ::protobuf::Message>::default_instance()
    }
}

impl GetArtworkPlaceholderResponse {
    pub fn new() -> GetArtworkPlaceholderResponse {
        ::std::default::Default::default()
    }
}

impl ::protobuf::Message for GetArtworkPlaceholderResponse {
    const NAME: &'static str = "GetArtworkPlaceholderResponse";

    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::Result<()> {
        while let Some(tag) = is.read_raw_tag_or_eof()? {
            match tag {
                18 => {
                    ::protobuf::rt::read_singular_message_into_field(is, &mut self.image)?;
                },
                tag => {
                    ::protobuf::rt::read_unknown_or_skip_group(tag, is, self.special_fields.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u64 {
        let mut my_size = 0;
        if let Some(v) = self.image.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint64_size(len) + len;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.special_fields.unknown_fields());
        self.special_fields.cached_size().set(my_size as u32);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::Result<()> {
        if let Some(v) = self.image.as_ref() {
            ::protobuf::rt::write_message_field_with_cached_size(2, v, os)?;
        }
        os.write_unknown_fields(self.special_fields.unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn special_fields(&self) -> &::protobuf::SpecialFields {
        &self.special_fields
    }

    fn mut_special_fields(&mut self) -> &mut ::protobuf::SpecialFields {
        &mut self.special_fields
    }

    fn new() -> GetArtworkPlaceholderResponse {
        GetArtworkPlaceholderResponse::new()
    }

    fn clear(&mut self) {
        self.image.clear();
        self.special_fields.clear();
    }

    fn default_instance() -> &'static GetArtworkPlaceholderResponse {
        static instance: GetArtworkPlaceholderResponse = GetArtworkPlaceholderResponse {
            image: ::protobuf::MessageField::none(),
            special_fields: ::protobuf::SpecialFields::new(),
        };
        &instance
    }
}

// @@protoc_insertion_point(message:fx.ipc.proto.GetFanartRequest)
#[derive(PartialEq,Clone,Default,Debug)]
pub struct GetFanartRequest {
    // message fields
    // @@protoc_insertion_point(field:fx.ipc.proto.GetFanartRequest.media)
    pub media: ::protobuf::MessageField<super::media::media::Item>,
    // special fields
    // @@protoc_insertion_point(special_field:fx.ipc.proto.GetFanartRequest.special_fields)
    pub special_fields: ::protobuf::SpecialFields,
}

impl<'a> ::std::default::Default for &'a GetFanartRequest {
    fn default() -> &'a GetFanartRequest {
        <GetFanartRequest as ::protobuf::Message>::default_instance()
    }
}

impl GetFanartRequest {
    pub fn new() -> GetFanartRequest {
        ::std::default::Default::default()
    }
}

impl ::protobuf::Message for GetFanartRequest {
    const NAME: &'static str = "GetFanartRequest";

    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::Result<()> {
        while let Some(tag) = is.read_raw_tag_or_eof()? {
            match tag {
                10 => {
                    ::protobuf::rt::read_singular_message_into_field(is, &mut self.media)?;
                },
                tag => {
                    ::protobuf::rt::read_unknown_or_skip_group(tag, is, self.special_fields.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u64 {
        let mut my_size = 0;
        if let Some(v) = self.media.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint64_size(len) + len;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.special_fields.unknown_fields());
        self.special_fields.cached_size().set(my_size as u32);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::Result<()> {
        if let Some(v) = self.media.as_ref() {
            ::protobuf::rt::write_message_field_with_cached_size(1, v, os)?;
        }
        os.write_unknown_fields(self.special_fields.unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn special_fields(&self) -> &::protobuf::SpecialFields {
        &self.special_fields
    }

    fn mut_special_fields(&mut self) -> &mut ::protobuf::SpecialFields {
        &mut self.special_fields
    }

    fn new() -> GetFanartRequest {
        GetFanartRequest::new()
    }

    fn clear(&mut self) {
        self.media.clear();
        self.special_fields.clear();
    }

    fn default_instance() -> &'static GetFanartRequest {
        static instance: GetFanartRequest = GetFanartRequest {
            media: ::protobuf::MessageField::none(),
            special_fields: ::protobuf::SpecialFields::new(),
        };
        &instance
    }
}

// @@protoc_insertion_point(message:fx.ipc.proto.GetFanartResponse)
#[derive(PartialEq,Clone,Default,Debug)]
pub struct GetFanartResponse {
    // message fields
    // @@protoc_insertion_point(field:fx.ipc.proto.GetFanartResponse.result)
    pub result: ::protobuf::EnumOrUnknown<super::message::response::Result>,
    // @@protoc_insertion_point(field:fx.ipc.proto.GetFanartResponse.image)
    pub image: ::protobuf::MessageField<Image>,
    // @@protoc_insertion_point(field:fx.ipc.proto.GetFanartResponse.error)
    pub error: ::std::option::Option<::protobuf::EnumOrUnknown<image::Error>>,
    // special fields
    // @@protoc_insertion_point(special_field:fx.ipc.proto.GetFanartResponse.special_fields)
    pub special_fields: ::protobuf::SpecialFields,
}

impl<'a> ::std::default::Default for &'a GetFanartResponse {
    fn default() -> &'a GetFanartResponse {
        <GetFanartResponse as ::protobuf::Message>::default_instance()
    }
}

impl GetFanartResponse {
    pub fn new() -> GetFanartResponse {
        ::std::default::Default::default()
    }
}

impl ::protobuf::Message for GetFanartResponse {
    const NAME: &'static str = "GetFanartResponse";

    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::Result<()> {
        while let Some(tag) = is.read_raw_tag_or_eof()? {
            match tag {
                8 => {
                    self.result = is.read_enum_or_unknown()?;
                },
                18 => {
                    ::protobuf::rt::read_singular_message_into_field(is, &mut self.image)?;
                },
                24 => {
                    self.error = ::std::option::Option::Some(is.read_enum_or_unknown()?);
                },
                tag => {
                    ::protobuf::rt::read_unknown_or_skip_group(tag, is, self.special_fields.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u64 {
        let mut my_size = 0;
        if self.result != ::protobuf::EnumOrUnknown::new(super::message::response::Result::OK) {
            my_size += ::protobuf::rt::int32_size(1, self.result.value());
        }
        if let Some(v) = self.image.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint64_size(len) + len;
        }
        if let Some(v) = self.error {
            my_size += ::protobuf::rt::int32_size(3, v.value());
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.special_fields.unknown_fields());
        self.special_fields.cached_size().set(my_size as u32);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::Result<()> {
        if self.result != ::protobuf::EnumOrUnknown::new(super::message::response::Result::OK) {
            os.write_enum(1, ::protobuf::EnumOrUnknown::value(&self.result))?;
        }
        if let Some(v) = self.image.as_ref() {
            ::protobuf::rt::write_message_field_with_cached_size(2, v, os)?;
        }
        if let Some(v) = self.error {
            os.write_enum(3, ::protobuf::EnumOrUnknown::value(&v))?;
        }
        os.write_unknown_fields(self.special_fields.unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn special_fields(&self) -> &::protobuf::SpecialFields {
        &self.special_fields
    }

    fn mut_special_fields(&mut self) -> &mut ::protobuf::SpecialFields {
        &mut self.special_fields
    }

    fn new() -> GetFanartResponse {
        GetFanartResponse::new()
    }

    fn clear(&mut self) {
        self.result = ::protobuf::EnumOrUnknown::new(super::message::response::Result::OK);
        self.image.clear();
        self.error = ::std::option::Option::None;
        self.special_fields.clear();
    }

    fn default_instance() -> &'static GetFanartResponse {
        static instance: GetFanartResponse = GetFanartResponse {
            result: ::protobuf::EnumOrUnknown::from_i32(0),
            image: ::protobuf::MessageField::none(),
            error: ::std::option::Option::None,
            special_fields: ::protobuf::SpecialFields::new(),
        };
        &instance
    }
}

// @@protoc_insertion_point(message:fx.ipc.proto.GetPosterRequest)
#[derive(PartialEq,Clone,Default,Debug)]
pub struct GetPosterRequest {
    // message fields
    // @@protoc_insertion_point(field:fx.ipc.proto.GetPosterRequest.media)
    pub media: ::protobuf::MessageField<super::media::media::Item>,
    // special fields
    // @@protoc_insertion_point(special_field:fx.ipc.proto.GetPosterRequest.special_fields)
    pub special_fields: ::protobuf::SpecialFields,
}

impl<'a> ::std::default::Default for &'a GetPosterRequest {
    fn default() -> &'a GetPosterRequest {
        <GetPosterRequest as ::protobuf::Message>::default_instance()
    }
}

impl GetPosterRequest {
    pub fn new() -> GetPosterRequest {
        ::std::default::Default::default()
    }
}

impl ::protobuf::Message for GetPosterRequest {
    const NAME: &'static str = "GetPosterRequest";

    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::Result<()> {
        while let Some(tag) = is.read_raw_tag_or_eof()? {
            match tag {
                10 => {
                    ::protobuf::rt::read_singular_message_into_field(is, &mut self.media)?;
                },
                tag => {
                    ::protobuf::rt::read_unknown_or_skip_group(tag, is, self.special_fields.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u64 {
        let mut my_size = 0;
        if let Some(v) = self.media.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint64_size(len) + len;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.special_fields.unknown_fields());
        self.special_fields.cached_size().set(my_size as u32);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::Result<()> {
        if let Some(v) = self.media.as_ref() {
            ::protobuf::rt::write_message_field_with_cached_size(1, v, os)?;
        }
        os.write_unknown_fields(self.special_fields.unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn special_fields(&self) -> &::protobuf::SpecialFields {
        &self.special_fields
    }

    fn mut_special_fields(&mut self) -> &mut ::protobuf::SpecialFields {
        &mut self.special_fields
    }

    fn new() -> GetPosterRequest {
        GetPosterRequest::new()
    }

    fn clear(&mut self) {
        self.media.clear();
        self.special_fields.clear();
    }

    fn default_instance() -> &'static GetPosterRequest {
        static instance: GetPosterRequest = GetPosterRequest {
            media: ::protobuf::MessageField::none(),
            special_fields: ::protobuf::SpecialFields::new(),
        };
        &instance
    }
}

// @@protoc_insertion_point(message:fx.ipc.proto.GetPosterResponse)
#[derive(PartialEq,Clone,Default,Debug)]
pub struct GetPosterResponse {
    // message fields
    // @@protoc_insertion_point(field:fx.ipc.proto.GetPosterResponse.result)
    pub result: ::protobuf::EnumOrUnknown<super::message::response::Result>,
    // @@protoc_insertion_point(field:fx.ipc.proto.GetPosterResponse.image)
    pub image: ::protobuf::MessageField<Image>,
    // @@protoc_insertion_point(field:fx.ipc.proto.GetPosterResponse.error)
    pub error: ::std::option::Option<::protobuf::EnumOrUnknown<image::Error>>,
    // special fields
    // @@protoc_insertion_point(special_field:fx.ipc.proto.GetPosterResponse.special_fields)
    pub special_fields: ::protobuf::SpecialFields,
}

impl<'a> ::std::default::Default for &'a GetPosterResponse {
    fn default() -> &'a GetPosterResponse {
        <GetPosterResponse as ::protobuf::Message>::default_instance()
    }
}

impl GetPosterResponse {
    pub fn new() -> GetPosterResponse {
        ::std::default::Default::default()
    }
}

impl ::protobuf::Message for GetPosterResponse {
    const NAME: &'static str = "GetPosterResponse";

    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::Result<()> {
        while let Some(tag) = is.read_raw_tag_or_eof()? {
            match tag {
                8 => {
                    self.result = is.read_enum_or_unknown()?;
                },
                18 => {
                    ::protobuf::rt::read_singular_message_into_field(is, &mut self.image)?;
                },
                24 => {
                    self.error = ::std::option::Option::Some(is.read_enum_or_unknown()?);
                },
                tag => {
                    ::protobuf::rt::read_unknown_or_skip_group(tag, is, self.special_fields.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u64 {
        let mut my_size = 0;
        if self.result != ::protobuf::EnumOrUnknown::new(super::message::response::Result::OK) {
            my_size += ::protobuf::rt::int32_size(1, self.result.value());
        }
        if let Some(v) = self.image.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint64_size(len) + len;
        }
        if let Some(v) = self.error {
            my_size += ::protobuf::rt::int32_size(3, v.value());
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.special_fields.unknown_fields());
        self.special_fields.cached_size().set(my_size as u32);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::Result<()> {
        if self.result != ::protobuf::EnumOrUnknown::new(super::message::response::Result::OK) {
            os.write_enum(1, ::protobuf::EnumOrUnknown::value(&self.result))?;
        }
        if let Some(v) = self.image.as_ref() {
            ::protobuf::rt::write_message_field_with_cached_size(2, v, os)?;
        }
        if let Some(v) = self.error {
            os.write_enum(3, ::protobuf::EnumOrUnknown::value(&v))?;
        }
        os.write_unknown_fields(self.special_fields.unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn special_fields(&self) -> &::protobuf::SpecialFields {
        &self.special_fields
    }

    fn mut_special_fields(&mut self) -> &mut ::protobuf::SpecialFields {
        &mut self.special_fields
    }

    fn new() -> GetPosterResponse {
        GetPosterResponse::new()
    }

    fn clear(&mut self) {
        self.result = ::protobuf::EnumOrUnknown::new(super::message::response::Result::OK);
        self.image.clear();
        self.error = ::std::option::Option::None;
        self.special_fields.clear();
    }

    fn default_instance() -> &'static GetPosterResponse {
        static instance: GetPosterResponse = GetPosterResponse {
            result: ::protobuf::EnumOrUnknown::from_i32(0),
            image: ::protobuf::MessageField::none(),
            error: ::std::option::Option::None,
            special_fields: ::protobuf::SpecialFields::new(),
        };
        &instance
    }
}

// @@protoc_insertion_point(message:fx.ipc.proto.GetImageRequest)
#[derive(PartialEq,Clone,Default,Debug)]
pub struct GetImageRequest {
    // message fields
    // @@protoc_insertion_point(field:fx.ipc.proto.GetImageRequest.url)
    pub url: ::std::string::String,
    // special fields
    // @@protoc_insertion_point(special_field:fx.ipc.proto.GetImageRequest.special_fields)
    pub special_fields: ::protobuf::SpecialFields,
}

impl<'a> ::std::default::Default for &'a GetImageRequest {
    fn default() -> &'a GetImageRequest {
        <GetImageRequest as ::protobuf::Message>::default_instance()
    }
}

impl GetImageRequest {
    pub fn new() -> GetImageRequest {
        ::std::default::Default::default()
    }
}

impl ::protobuf::Message for GetImageRequest {
    const NAME: &'static str = "GetImageRequest";

    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::Result<()> {
        while let Some(tag) = is.read_raw_tag_or_eof()? {
            match tag {
                10 => {
                    self.url = is.read_string()?;
                },
                tag => {
                    ::protobuf::rt::read_unknown_or_skip_group(tag, is, self.special_fields.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u64 {
        let mut my_size = 0;
        if !self.url.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.url);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.special_fields.unknown_fields());
        self.special_fields.cached_size().set(my_size as u32);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::Result<()> {
        if !self.url.is_empty() {
            os.write_string(1, &self.url)?;
        }
        os.write_unknown_fields(self.special_fields.unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn special_fields(&self) -> &::protobuf::SpecialFields {
        &self.special_fields
    }

    fn mut_special_fields(&mut self) -> &mut ::protobuf::SpecialFields {
        &mut self.special_fields
    }

    fn new() -> GetImageRequest {
        GetImageRequest::new()
    }

    fn clear(&mut self) {
        self.url.clear();
        self.special_fields.clear();
    }

    fn default_instance() -> &'static GetImageRequest {
        static instance: GetImageRequest = GetImageRequest {
            url: ::std::string::String::new(),
            special_fields: ::protobuf::SpecialFields::new(),
        };
        &instance
    }
}

// @@protoc_insertion_point(message:fx.ipc.proto.GetImageResponse)
#[derive(PartialEq,Clone,Default,Debug)]
pub struct GetImageResponse {
    // message fields
    // @@protoc_insertion_point(field:fx.ipc.proto.GetImageResponse.result)
    pub result: ::protobuf::EnumOrUnknown<super::message::response::Result>,
    // @@protoc_insertion_point(field:fx.ipc.proto.GetImageResponse.image)
    pub image: ::protobuf::MessageField<Image>,
    // @@protoc_insertion_point(field:fx.ipc.proto.GetImageResponse.error)
    pub error: ::std::option::Option<::protobuf::EnumOrUnknown<image::Error>>,
    // special fields
    // @@protoc_insertion_point(special_field:fx.ipc.proto.GetImageResponse.special_fields)
    pub special_fields: ::protobuf::SpecialFields,
}

impl<'a> ::std::default::Default for &'a GetImageResponse {
    fn default() -> &'a GetImageResponse {
        <GetImageResponse as ::protobuf::Message>::default_instance()
    }
}

impl GetImageResponse {
    pub fn new() -> GetImageResponse {
        ::std::default::Default::default()
    }
}

impl ::protobuf::Message for GetImageResponse {
    const NAME: &'static str = "GetImageResponse";

    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::Result<()> {
        while let Some(tag) = is.read_raw_tag_or_eof()? {
            match tag {
                8 => {
                    self.result = is.read_enum_or_unknown()?;
                },
                18 => {
                    ::protobuf::rt::read_singular_message_into_field(is, &mut self.image)?;
                },
                24 => {
                    self.error = ::std::option::Option::Some(is.read_enum_or_unknown()?);
                },
                tag => {
                    ::protobuf::rt::read_unknown_or_skip_group(tag, is, self.special_fields.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u64 {
        let mut my_size = 0;
        if self.result != ::protobuf::EnumOrUnknown::new(super::message::response::Result::OK) {
            my_size += ::protobuf::rt::int32_size(1, self.result.value());
        }
        if let Some(v) = self.image.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint64_size(len) + len;
        }
        if let Some(v) = self.error {
            my_size += ::protobuf::rt::int32_size(3, v.value());
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.special_fields.unknown_fields());
        self.special_fields.cached_size().set(my_size as u32);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::Result<()> {
        if self.result != ::protobuf::EnumOrUnknown::new(super::message::response::Result::OK) {
            os.write_enum(1, ::protobuf::EnumOrUnknown::value(&self.result))?;
        }
        if let Some(v) = self.image.as_ref() {
            ::protobuf::rt::write_message_field_with_cached_size(2, v, os)?;
        }
        if let Some(v) = self.error {
            os.write_enum(3, ::protobuf::EnumOrUnknown::value(&v))?;
        }
        os.write_unknown_fields(self.special_fields.unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn special_fields(&self) -> &::protobuf::SpecialFields {
        &self.special_fields
    }

    fn mut_special_fields(&mut self) -> &mut ::protobuf::SpecialFields {
        &mut self.special_fields
    }

    fn new() -> GetImageResponse {
        GetImageResponse::new()
    }

    fn clear(&mut self) {
        self.result = ::protobuf::EnumOrUnknown::new(super::message::response::Result::OK);
        self.image.clear();
        self.error = ::std::option::Option::None;
        self.special_fields.clear();
    }

    fn default_instance() -> &'static GetImageResponse {
        static instance: GetImageResponse = GetImageResponse {
            result: ::protobuf::EnumOrUnknown::from_i32(0),
            image: ::protobuf::MessageField::none(),
            error: ::std::option::Option::None,
            special_fields: ::protobuf::SpecialFields::new(),
        };
        &instance
    }
}
