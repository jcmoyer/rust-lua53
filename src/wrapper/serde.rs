use std::fmt;
use std::{i32, i64};
use serde::{Serialize, Serializer, ser};

use wrapper::convert::ToLua;
use wrapper::state::State;

pub struct Serde<'a, S: Serialize + ?Sized + 'a>(&'a S);

struct LuaSerializer<'a>(&'a mut State);

struct SerializeSeq<'a> {
    state: &'a mut State,
    table_index: i32,
    current_subscript: i32
}

struct SerializeTuple<'a>(&'a mut State);
struct SerializeTupleStruct<'a>(&'a mut State);
struct SerializeTupleVariant<'a>(&'a mut State);

struct SerializeMap<'a> {
    state: &'a mut State,
    table_index: i32,
}

struct SerializeStruct<'a> {
    state: &'a mut State,
    table_index: i32,
}

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
        TableSizeTooLarge(v: u64) {
            display("table size {} is too large for lua", v)
            description("table size is too large for lua (31 bits max)")
        }
    }
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        ErrorEnum::Custom(msg.to_string()).into()
    }
}

impl<'a> SerializeSeq<'a> {
    fn new(state: &'a mut State, prealloc: i32) -> SerializeSeq<'a> {
        state.create_table(prealloc, 0);
        SerializeSeq {
            table_index: state.get_top(),
            current_subscript: 0,
            state,
        }
    }
}

impl<'a> ser::SerializeSeq for SerializeSeq<'a> {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T: ?Sized>(&mut self, value: &T)
        -> Result<(), Self::Error>
        where T: Serialize
    {
        if self.current_subscript == i32::MAX {
            return Err(ErrorEnum::TableSizeTooLarge(
                self.current_subscript as u64).into());
        }
        self.current_subscript += 1;
        value.serialize(LuaSerializer(self.state))?;
        self.state.raw_seti(self.table_index, self.current_subscript as i64);
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        // table is already at the top of the stack
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for SerializeTuple<'a> {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T: ?Sized>(&mut self, _value: &T)
        -> Result<(), Self::Error>
        where T: Serialize
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
    fn serialize_field<T: ?Sized>(&mut self, _value: &T)
        -> Result<(), Self::Error>
        where T: Serialize
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
    fn serialize_field<T: ?Sized>(&mut self, _value: &T)
        -> Result<(), Self::Error>
        where T: Serialize
    {
        unimplemented!();
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!();
    }
}

impl<'a> SerializeMap<'a> {
    fn new(state: &'a mut State, prealloc: i32) -> SerializeMap<'a> {
        state.create_table(0, prealloc);
        SerializeMap {
            table_index: state.get_top(),
            state,
        }
    }
}

impl<'a> ser::SerializeMap for SerializeMap<'a> {
    type Ok = ();
    type Error = Error;
    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize
    {
        key.serialize(LuaSerializer(self.state))
    }
    fn serialize_value<T: ?Sized>(
        &mut self,
        value: &T
    ) -> Result<(), Self::Error>
    where
        T: Serialize
    {
        value.serialize(LuaSerializer(self.state))?;
        self.state.raw_set(self.table_index);
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        // table is already at the top of the stack
        Ok(())
    }
}

impl<'a> SerializeStruct<'a> {
    fn new(state: &'a mut State, fields: i32) -> SerializeStruct<'a> {
        state.create_table(0, fields);
        SerializeStruct {
            table_index: state.get_top(),
            state,
        }
    }
}

impl<'a> ser::SerializeStruct for SerializeStruct<'a> {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T)
        -> Result<(), Self::Error>
        where T: Serialize
    {
        value.serialize(LuaSerializer(self.state))?;
        self.state.set_field(self.table_index, key);
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        // table is already at the top of the stack
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for SerializeStructVariant<'a> {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, _value: &T)
        -> Result<(), Self::Error>
        where T: Serialize
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
    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
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
    fn serialize_unit_struct(self, _name: &'static str)
        -> Result<Self::Ok, Self::Error>
    {
        unimplemented!();
    }
    fn serialize_unit_variant(self,
        _name: &'static str, _variant_index: u32, _variant: &'static str)
        -> Result<Self::Ok, Self::Error>
    {
        unimplemented!();
    }
    fn serialize_newtype_struct<T: ?Sized>(self,
        _name: &'static str, _value: &T)
        -> Result<Self::Ok, Self::Error>
        where T: Serialize
    {
        unimplemented!();
    }
    fn serialize_newtype_variant<T: ?Sized>(self,
        _name: &'static str, _variant_index: u32, _variant: &'static str,
        _value: &T
        ) -> Result<Self::Ok, Self::Error>
        where T: Serialize
    {
        unimplemented!();
    }
    fn serialize_seq(self, len: Option<usize>)
        -> Result<Self::SerializeSeq, Self::Error>
    {
        if len.map(|x| x <= i32::MAX as usize).unwrap_or(true) {
            Ok(SerializeSeq::new(self.0, len.map(|x| x as i32).unwrap_or(0)))
        } else {
            Err(ErrorEnum::IntegerTooLarge(len.unwrap() as u64).into())
        }
    }
    fn serialize_tuple(self, _len: usize)
        -> Result<Self::SerializeTuple, Self::Error>
    {
        unimplemented!();
    }
    fn serialize_tuple_struct(self,
        _name: &'static str, _len: usize)
        -> Result<Self::SerializeTupleStruct, Self::Error>
    {
        unimplemented!();
    }
    fn serialize_tuple_variant(self,
        _name: &'static str, _variant_index: u32, _variant: &'static str,
        _len: usize)
        -> Result<Self::SerializeTupleVariant, Self::Error>
    {
        unimplemented!();
    }
    fn serialize_map(self, len: Option<usize>)
        -> Result<Self::SerializeMap, Self::Error>
    {
        Ok(SerializeMap::new(self.0, len.map(|x| x as i32).unwrap_or(0)))
    }
    fn serialize_struct(self, _name: &'static str, len: usize)
        -> Result<Self::SerializeStruct, Self::Error>
    {
        if len <= i32::MAX as usize {
            Ok(SerializeStruct::new(self.0, len as i32))
        } else {
            return Err(ErrorEnum::TableSizeTooLarge(len as u64).into());
        }
    }
    fn serialize_struct_variant(self,
        _name: &'static str, _variant_index: u32, _variant: &'static str,
        _len: usize)
        -> Result<Self::SerializeStructVariant, Self::Error>
    {
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
    use std::collections::HashMap;
    use std::time::Duration;

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

    #[test]
    fn serialize_list() {
      let mut state = State::new();
      let x: &[u32] = &[1, 2, 3];
      state.push(Serde(&x));
      let x = vec![1, 2, 3];
      state.push(Serde(&x));
    }

    #[test]
    fn serialize_map() {
      let mut state = State::new();
      let x = vec![
        ("x", 1),
        ("y", 2),
      ].into_iter().collect::<HashMap<_, _>>();
      state.push(Serde(&x));
    }

    #[test]
    fn serialize_struct() {
      let mut state = State::new();
      // Duration is serialized as a struct with two fields
      // so we can test it without using `serde_derive`
      state.push(Serde(&Duration::from_millis(12345)));
    }


}
