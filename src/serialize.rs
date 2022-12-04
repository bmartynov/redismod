use core::fmt::{Display, Formatter};
use std::time::Duration;
use redis_module::{raw as rraw, RedisModuleIO, RedisModuleTimerID};

use serde::{Serialize, ser};
use crate::Saver;

#[derive(Debug)]
pub struct Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Error").finish()
    }
}

impl std::error::Error for Error {}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self where T: Display {
        todo!()
    }
}


pub struct Serializer<S: Saver> {
    // This string starts empty and JSON is appended as values are serialized.
    // output: String,

    saver: S,
}

impl<'a, S: Saver> ser::Serializer for &'a mut Serializer<S> {
    // The output type produced by this `Serializer` during successful
    // serialization. Most serializers that produce text or binary output should
    // set `Ok = ()` and serialize into an `io::Write` or buffer contained
    // within the `Serializer` instance, as happens here. Serializers that build
    // in-memory data structures may be simplified by using `Ok` to propagate
    // the data structure around.
    type Ok = ();

    // The error type when some error occurs during serialization.
    type Error = Error;

    // Associated types for keeping track of additional state while serializing
    // compound data structures like sequences and maps. In this case no
    // additional state is required beyond what is already stored in the
    // Serializer struct.
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    // Here we go with the simple methods. The following 12 methods receive one
    // of the primitive types of the data model and map it to JSON by appending
    // into the output string.
    fn serialize_bool(self, v: bool) -> Result<(), Self::Error> {
        // self.output += if v { "true" } else { "false" };

        // rraw::save_unsigned(self.rdb, v as u64);

        self.saver.unsigned(v as u64);

        Ok(())
    }

    // JSON does not distinguish between different sizes of integers, so all
    // signed integers will be serialized the same and all unsigned integers
    // will be serialized the same. Other formats, especially compact binary
    // formats, may need independent logic for the different sizes.
    fn serialize_i8(self, v: i8) -> Result<(), Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<(), Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<(), Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    // Not particularly efficient but this is example code anyway. A more
    // performant approach would be to use the `itoa` crate.
    fn serialize_i64(self, v: i64) -> Result<(), Self::Error> {
        // self.output += &v.to_string();

        // rraw::save_signed(self.rdb, v as i64);
        self.saver.signed(v as i64);

        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<(), Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<(), Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<(), Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<(), Self::Error> {
        self.saver.unsigned(v as u64);

        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<(), Self::Error> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<(), Self::Error> {
        // self.output += &v.to_string();

        self.saver.double(v as f64);

        Ok(())
    }

    // Serialize a char as a single-character string. Other formats may
    // represent this differently.
    fn serialize_char(self, v: char) -> Result<(), Self::Error> {
        self.serialize_str(&v.to_string())
    }

    // This only works for strings that don't require escape sequences but you
    // get the idea. For example it would emit invalid JSON if the input string
    // contains a '"' character.
    fn serialize_str(self, v: &str) -> Result<(), Self::Error> {
        // self.output += "\"";
        // self.output += v;
        // self.output += "\"";

        // rraw::save_string(self.rdb, v);

        self.saver.string(v);

        Ok(())
    }

    // Serialize a byte array as an array of bytes. Could also use a base64
    // string here. Binary formats will typically represent byte arrays more
    // compactly.
    fn serialize_bytes(self, v: &[u8]) -> Result<(), Self::Error> {
        // use serde::ser::SerializeSeq;
        // let mut seq = self.serialize_seq(Some(v.len()))?;
        // for byte in v {
        //     seq.serialize_element(byte)?;
        // }
        //
        //
        //
        // seq.end()

        // rraw::save_slice(self.rdb, v);

        self.saver.buffer(v);

        Ok(())
    }

    // An absent optional is represented as the JSON `null`.
    fn serialize_none(self) -> Result<(), Self::Error> {
        self.serialize_unit()
    }

    // A present optional is represented as just the contained value. Note that
    // this is a lossy representation. For example the values `Some(())` and
    // `None` both serialize as just `null`. Unfortunately this is typically
    // what people expect when working with JSON. Other formats are encouraged
    // to behave more intelligently if possible.
    fn serialize_some<T>(self, value: &T) -> Result<(), Self::Error>
        where
            T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // In Serde, unit means an anonymous value containing no data. Map this to
    // JSON as `null`.
    fn serialize_unit(self) -> Result<(), Self::Error> {
        self.serialize_i64(-1)?;

        Ok(())
    }

    // Unit struct means a named value containing no data. Again, since there is
    // no data, map this to JSON as `null`. There is no need to serialize the
    // name in most formats.
    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), Self::Error> {
        self.serialize_unit()
    }

    // When serializing a unit variant (or any other kind of variant), formats
    // can choose whether to keep track of it by index or by name. Binary
    // formats typically use the index of the variant and human-readable formats
    // typically use the name.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<(), Self::Error> {
        self.serialize_str(variant)
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain.
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
        where
            T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // Note that newtype variant (and all of the other variant serialization
    // methods) refer exclusively to the "externally tagged" enum
    // representation.
    //
    // Serialize this to JSON in externally tagged form as `{ NAME: VALUE }`.
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
        where
            T: ?Sized + Serialize,
    {
        // self.output += "{";
        // variant.serialize(&mut *self)?;
        // self.output += ":";
        // value.serialize(&mut *self)?;
        // self.output += "}";
        Ok(())
    }

    // Now we get to the serialization of compound types.
    //
    // The start of the sequence, each value, and the end are three separate
    // method calls. This one is responsible only for serializing the start,
    // which in JSON is `[`.
    //
    // The length of the sequence may or may not be known ahead of time. This
    // doesn't make a difference in JSON because the length is not represented
    // explicitly in the serialized form. Some serializers may only be able to
    // support sequences for which the length is known up front.
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        // self.output += "[";
        Ok(self)
    }

    // Tuples look just like sequences in JSON. Some formats may be able to
    // represent tuples more efficiently by omitting the length, since tuple
    // means that the corresponding `Deserialize implementation will know the
    // length without needing to look at the serialized data.
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    // Tuple structs look just like sequences in JSON.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }`. Again
    // this method is only responsible for the externally tagged representation.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        // self.output += "{";
        // variant.serialize(&mut *self)?;
        // self.output += ":[";
        Ok(self)
    }

    // Maps are represented in JSON as `{ K: V, K: V, ... }`.
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        // self.output += "{";
        Ok(self)
    }

    // Structs look just like maps in JSON. In particular, JSON requires that we
    // serialize the field names of the struct. Other formats may be able to
    // omit the field names when serializing structs because the corresponding
    // Deserialize implementation is required to know what the keys are without
    // looking at the serialized data.
    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }`.
    // This is the externally tagged representation.
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        // self.output += "{";
        // variant.serialize(&mut *self)?;
        // self.output += ":{";
        Ok(self)
    }
}

// The following 7 impls deal with the serialization of compound types like
// sequences and maps. Serialization of such types is begun by a Serializer
// method and followed by zero or more calls to serialize individual elements of
// the compound type and one call to end the compound type.
//
// This impl is SerializeSeq so these methods are called after `serialize_seq`
// is called on the Serializer.
impl<'a, S: Saver> ser::SerializeSeq for &'a mut Serializer<S> {
    // Must match the `Ok` type of the serializer.
    type Ok = ();
    // Must match the `Error` type of the serializer.
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: ?Sized + Serialize,
    {
        // if !self.output.ends_with('[') {
        //     self.output += ",";
        // }
        value.serialize(&mut **self)
    }

    // Close the sequence.
    fn end(self) -> Result<(), Self::Error> {
        // self.output += "]";
        Ok(())
    }
}

// Same thing but for tuples.
impl<'a, S: Saver> ser::SerializeTuple for &'a mut Serializer<S> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: ?Sized + Serialize,
    {
        // if !self.output.ends_with('[') {
        //     self.output += ",";
        // }
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Self::Error> {
        // self.output += "]";
        Ok(())
    }
}

// Same thing but for tuple structs.
impl<'a, S: Saver> ser::SerializeTupleStruct for &'a mut Serializer<S> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: ?Sized + Serialize,
    {
        // if !self.output.ends_with('[') {
        //     self.output += ",";
        // }
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Self::Error> {
        // self.output += "]";
        Ok(())
    }
}

// Tuple variants are a little different. Refer back to the
// `serialize_tuple_variant` method above:
//
//    self.output += "{";
//    variant.serialize(&mut *self)?;
//    self.output += ":[";
//
// So the `end` method in this impl is responsible for closing both the `]` and
// the `}`.
impl<'a, S: Saver> ser::SerializeTupleVariant for &'a mut Serializer<S> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: ?Sized + Serialize,
    {
        // if !self.output.ends_with('[') {
        //     self.output += ",";
        // }
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Self::Error> {
        // self.output += "]}";
        Ok(())
    }
}

// Some `Serialize` types are not able to hold a key and value in memory at the
// same time so `SerializeMap` implementations are required to support
// `serialize_key` and `serialize_value` individually.
//
// There is a third optional method on the `SerializeMap` trait. The
// `serialize_entry` method allows serializers to optimize for the case where
// key and value are both available simultaneously. In JSON it doesn't make a
// difference so the default behavior for `serialize_entry` is fine.
impl<'a, S: Saver> ser::SerializeMap for &'a mut Serializer<S> {
    type Ok = ();
    type Error = Error;

    // The Serde data model allows map keys to be any serializable type. JSON
    // only allows string keys so the implementation below will produce invalid
    // JSON if the key serializes as something other than a string.
    //
    // A real JSON serializer would need to validate that map keys are strings.
    // This can be done by using a different Serializer to serialize the key
    // (instead of `&mut **self`) and having that other serializer only
    // implement `serialize_str` and return an error on any other data type.
    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
        where
            T: ?Sized + Serialize,
    {
        // if !self.output.ends_with('{') {
        //     self.output += ",";
        // }
        // key.serialize(&mut **self)

        Ok(())
    }

    // It doesn't make a difference whether the colon is printed at the end of
    // `serialize_key` or at the beginning of `serialize_value`. In this case
    // the code is a bit simpler having it here.
    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: ?Sized + Serialize,
    {
        // self.output += ":";
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Self::Error> {
        // self.output += "}";
        Ok(())
    }
}

// Structs are like maps in which the keys are constrained to be compile-time
// constant strings.
impl<'a, S: Saver> ser::SerializeStruct for &'a mut Serializer<S> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
        where
            T: ?Sized + Serialize,
    {
        // if !self.output.ends_with('{') {
        //     self.output += ",";
        // }
        key.serialize(&mut **self)?;
        // self.output += ":";
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Self::Error> {
        // self.output += "}";
        Ok(())
    }
}

// Similar to `SerializeTupleVariant`, here the `end` method is responsible for
// closing both of the curly braces opened by `serialize_struct_variant`.
impl<'a, S: Saver> ser::SerializeStructVariant for &'a mut Serializer<S> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
        where
            T: ?Sized + Serialize,
    {
        // if !self.output.ends_with('{') {
        //     self.output += ",";
        // }
        key.serialize(&mut **self)?;
        // self.output += ":";
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Self::Error> {
        // self.output += "}}";
        Ok(())
    }
}

#[derive(serde::Serialize)]
pub struct Task {
    // pub id: xid::Id,
    pub r#type: String,
    pub retries: u64,
    pub timeout: Duration,
    pub worker: String,
    pub payload: Vec<u8>,
    // pub state: TaskState,
    //
    // pub created_at: Instant,
    // pub started_at: Option<Instant>,
    // pub failed_at: Option<Instant>,
    // pub finished_at: Option<Instant>,
    //
    pub timer_id: Option<RedisModuleTimerID>,
}


struct DummySaver;

impl Saver for DummySaver {
    fn double(&self, val: f64) {
        println!("double: {:?}", val);
    }

    fn float(&self, val: f32) {
        println!("float: {:?}", val);
    }

    fn unsigned(&self, val: u64) {
        println!("unsigned: {:?}", val);
    }

    fn signed(&self, val: i64) {
        println!("signed: {:?}", val);
    }

    fn string<S: AsRef<str>>(&self, val: S) {
        println!("string: {:?}", val.as_ref());
    }

    fn buffer<S: AsRef<[u8]>>(&self, val: S) {
        println!("buffer: {:?}", val.as_ref());
    }
}

#[test]
fn works() {
    let mut serializer = Serializer {
        saver: DummySaver,
    };

    let task = Task {
        r#type: "".to_string(),
        retries: 0,
        timeout: Default::default(),
        worker: "".to_string(),
        payload: vec![],
        timer_id: None,
    };

    task.serialize(&mut serializer).expect("ok");

    // println!("{}", serializer);
}