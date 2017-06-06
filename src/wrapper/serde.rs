use std::fmt;
use std::i64;
use serde::{Serialize, Serializer, ser};

use wrapper::convert::ToLua;
use wrapper::state::State;

pub struct Serde<'a, S: Serialize + ?Sized + 'a>(&'a S);

struct LuaSerializer<'a>(&'a mut State);

struct SerializeSeq<'a>(&'a mut State);
struct SerializeTuple<'a>(&'a mut State);
struct SerializeTupleStruct<'a>(&'a mut State);
struct SerializeTupleVariant<'a>(&'a mut State);
struct SerializeMap<'a>(&'a mut State);
struct SerializeStruct<'a>(&'a mut State);
struct SerializeStructVariant<'a>(&'a mut State);


quick_error! {
    #[derive(Debug)]
    pub enum Error wraps ErrorEnum {
        Custom(msg: String) {
            display("custom serialization error: {}", msg)
            description("custom serialization error")
        }
        IntegerTooLarge(v: u64) {
            display("integer {} is too large for lua", v)
            description("integer is too large for lua")
        }
    }
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        ErrorEnum::Custom(msg.to_string()).into()
    }
}


impl<'a> ser::SerializeSeq for SerializeSeq<'a> {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T: ?Sized>(
        &mut self,
        value: &T
    ) -> Result<(), Self::Error>
    where
        T: Serialize
    {
        unimplemented!();
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!();
    }
}

impl<'a> ser::SerializeTuple for SerializeTuple<'a> {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T: ?Sized>(
        &mut self,
        value: &T
    ) -> Result<(), Self::Error>
    where
        T: Serialize
    {
        unimplemented!();
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!();
    }
}

impl<'a> ser::SerializeTupleStruct for SerializeTupleStruct<'a> {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized>(
        &mut self,
        value: &T
    ) -> Result<(), Self::Error>
    where
        T: Serialize
    {
        unimplemented!();
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!();
    }
}

impl<'a> ser::SerializeTupleVariant for SerializeTupleVariant<'a> {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized>(
        &mut self,
        value: &T
    ) -> Result<(), Self::Error>
    where
        T: Serialize
    {
        unimplemented!();
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!();
    }
}

impl<'a> ser::SerializeMap for SerializeMap<'a> {
    type Ok = ();
    type Error = Error;
    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize
    {
        unimplemented!();
    }
    fn serialize_value<T: ?Sized>(
        &mut self,
        value: &T
    ) -> Result<(), Self::Error>
    where
        T: Serialize
    {
        unimplemented!();
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!();
    }
}

impl<'a> ser::SerializeStruct for SerializeStruct<'a> {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T
    ) -> Result<(), Self::Error>
    where
        T: Serialize
    {
        unimplemented!();
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!();
    }
}

impl<'a> ser::SerializeStructVariant for SerializeStructVariant<'a> {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T
    ) -> Result<(), Self::Error>
    where
        T: Serialize
    {
        unimplemented!();
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!();
    }
}

impl<'a> Serializer for LuaSerializer<'a> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = SerializeSeq<'a>;
    type SerializeTuple = SerializeTuple<'a>;
    type SerializeTupleStruct = SerializeTupleStruct<'a>;
    type SerializeTupleVariant = SerializeTupleVariant<'a>;
    type SerializeMap = SerializeMap<'a>;
    type SerializeStruct = SerializeStruct<'a>;
    type SerializeStructVariant = SerializeStructVariant<'a>;
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        unimplemented!();
    }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.0.push_integer(v as i64);
        Ok(())
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.0.push_integer(v as i64);
        Ok(())
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.0.push_integer(v as i64);
        Ok(())
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.0.push_integer(v);
        Ok(())
    }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.0.push_integer(v as i64);
        Ok(())
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.0.push_integer(v as i64);
        Ok(())
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.0.push_integer(v as i64);
        Ok(())
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        if v <= i64::MAX as u64 {
            self.0.push_integer(v as i64);
            Ok(())
        } else {
            Err(ErrorEnum::IntegerTooLarge(v).into())
        }
    }
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.0.push_number(v as f64);
        Ok(())
    }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.0.push_number(v);
        Ok(())
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.0.push_string(&v.to_string());
        Ok(())
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.0.push_string(v);
        Ok(())
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        unimplemented!();
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.0.push_nil();
        Ok(())
    }
    fn serialize_some<T: ?Sized>(
        self,
        value: &T
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize
    {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!();
    }
    fn serialize_unit_struct(
        self,
        name: &'static str
    ) -> Result<Self::Ok, Self::Error> {
        unimplemented!();
    }
    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str
    ) -> Result<Self::Ok, Self::Error> {
        unimplemented!();
    }
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize {
        unimplemented!();
    }
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize {
        unimplemented!();
    }
    fn serialize_seq(
        self,
        len: Option<usize>
    ) -> Result<Self::SerializeSeq, Self::Error> {
        unimplemented!();
    }
    fn serialize_tuple(
        self,
        len: usize
    ) -> Result<Self::SerializeTuple, Self::Error> {
        unimplemented!();
    }
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        unimplemented!();
    }
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        unimplemented!();
    }
    fn serialize_map(
        self,
        len: Option<usize>
    ) -> Result<Self::SerializeMap, Self::Error> {
        unimplemented!();
    }
    fn serialize_struct(
        self,
        name: &'static str,
        len: usize
    ) -> Result<Self::SerializeStruct, Self::Error> {
        unimplemented!();
    }
    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        unimplemented!();
    }
}

impl<'a, T: Serialize + 'a> ToLua for Serde<'a, T> {
  fn to_lua(&self, state: &mut State) {
    self.0.serialize(LuaSerializer(state))
        .expect("serialization error")
  }
}

#[cfg(test)]
mod test {
    use {State};
    use super::Serde;

    #[test]
    fn serialize_str() {
      let mut state = State::new();
      state.push(Serde(&"hello"));
    }

    #[test]
    fn serialize_int() {
      let mut state = State::new();
      state.push(Serde(&1i8));
      state.push(Serde(&1i16));
      state.push(Serde(&1i32));
      state.push(Serde(&1i64));
      state.push(Serde(&1u8));
      state.push(Serde(&1u16));
      state.push(Serde(&1u32));
      state.push(Serde(&1u64));
    }

    #[test]
    #[should_panic(expected="IntegerTooLarge")]
    fn serialize_big_int() {
      let mut state = State::new();
      state.push(Serde(&10000000000000000000u64));
    }

    #[test]
    fn serialize_float() {
      let mut state = State::new();
      state.push(Serde(&1.0f32));
      state.push(Serde(&1.0f64));
    }

    #[test]
    fn serialize_char() {
      let mut state = State::new();
      state.push(Serde(&'x'));
    }

    #[test]
    fn serialize_option() {
      let mut state = State::new();
      state.push(Serde(&Some("hello")));
      state.push(Serde(&None::<&str>));
    }
}
