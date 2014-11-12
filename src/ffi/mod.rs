// The MIT License (MIT)
//
// Copyright (c) 2014 J.C. Moyer
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub use self::lua::lua_Alloc;
pub use self::lua::lua_CFunction;
pub use self::lua::lua_Ctx;
pub use self::lua::lua_Debug;
pub use self::lua::lua_Hook;
pub use self::lua::lua_Integer;
pub use self::lua::lua_KFunction;
pub use self::lua::lua_Number;
pub use self::lua::lua_Reader;
pub use self::lua::lua_State;
pub use self::lua::lua_Unsigned;
pub use self::lua::lua_Writer;

pub use self::lua::lua_absindex;
pub use self::lua::lua_arith;
pub use self::lua::lua_atpanic;
pub use self::lua::lua_call;
pub use self::lua::lua_callk;
pub use self::lua::lua_checkstack;
pub use self::lua::lua_close;
pub use self::lua::lua_compare;
pub use self::lua::lua_concat;
pub use self::lua::lua_copy;
pub use self::lua::lua_createtable;
pub use self::lua::lua_dump;
pub use self::lua::lua_error;
pub use self::lua::lua_gc;
pub use self::lua::lua_getallocf;
pub use self::lua::lua_getextraspace;
pub use self::lua::lua_getfield;
pub use self::lua::lua_getglobal;
pub use self::lua::lua_gethook;
pub use self::lua::lua_gethookcount;
pub use self::lua::lua_gethookmask;
pub use self::lua::lua_getinfo;
pub use self::lua::lua_getlocal;
pub use self::lua::lua_getmetatable;
pub use self::lua::lua_getstack;
pub use self::lua::lua_gettable;
pub use self::lua::lua_gettop;
pub use self::lua::lua_getupvalue;
pub use self::lua::lua_getuservalue;
pub use self::lua::lua_insert;
pub use self::lua::lua_isboolean;
pub use self::lua::lua_iscfunction;
pub use self::lua::lua_isfunction;
pub use self::lua::lua_isinteger;
pub use self::lua::lua_islightuserdata;
pub use self::lua::lua_isnil;
pub use self::lua::lua_isnone;
pub use self::lua::lua_isnoneornil;
pub use self::lua::lua_isnumber;
pub use self::lua::lua_isstring;
pub use self::lua::lua_istable;
pub use self::lua::lua_isthread;
pub use self::lua::lua_isuserdata;
pub use self::lua::lua_isyieldable;
pub use self::lua::lua_len;
pub use self::lua::lua_load;
pub use self::lua::lua_newstate;
pub use self::lua::lua_newtable;
pub use self::lua::lua_newthread;
pub use self::lua::lua_newuserdata;
pub use self::lua::lua_next;
pub use self::luaconf::lua_numtointeger;
pub use self::lua::lua_pcall;
pub use self::lua::lua_pcallk;
pub use self::lua::lua_pop;
pub use self::lua::lua_pushboolean;
pub use self::lua::lua_pushcclosure;
pub use self::lua::lua_pushcfunction;
pub use self::lua::lua_pushfstring;
pub use self::lua::lua_pushglobaltable;
pub use self::lua::lua_pushinteger;
pub use self::lua::lua_pushlightuserdata;
pub use self::lua::lua_pushliteral;
pub use self::lua::lua_pushlstring;
pub use self::lua::lua_pushnil;
pub use self::lua::lua_pushnumber;
pub use self::lua::lua_pushstring;
pub use self::lua::lua_pushthread;
pub use self::lua::lua_pushvalue;
//pub use self::lua::lua_pushvfstring; -- not implemented
pub use self::lua::lua_rawequal;
pub use self::lua::lua_rawget;
pub use self::lua::lua_rawgeti;
pub use self::lua::lua_rawgetp;
pub use self::lua::lua_rawlen;
pub use self::lua::lua_rawset;
pub use self::lua::lua_rawseti;
pub use self::lua::lua_rawsetp;
pub use self::lua::lua_register;
pub use self::lua::lua_remove;
pub use self::lua::lua_replace;
pub use self::lua::lua_resume;
pub use self::lua::lua_rotate;
pub use self::lua::lua_setallocf;
pub use self::lua::lua_setfield;
pub use self::lua::lua_setglobal;
pub use self::lua::lua_sethook;
pub use self::lua::lua_setlocal;
pub use self::lua::lua_setmetatable;
pub use self::lua::lua_settable;
pub use self::lua::lua_settop;
pub use self::lua::lua_setupvalue;
pub use self::lua::lua_setuservalue;
pub use self::lua::lua_status;
pub use self::lua::lua_strtonum;
pub use self::lua::lua_toboolean;
pub use self::lua::lua_tocfunction;
pub use self::lua::lua_tointeger;
pub use self::lua::lua_tointegerx;
pub use self::lua::lua_tolstring;
pub use self::lua::lua_tonumber;
pub use self::lua::lua_tonumberx;
pub use self::lua::lua_topointer;
pub use self::lua::lua_tostring;
pub use self::lua::lua_tothread;
pub use self::lua::lua_touserdata;
pub use self::lua::lua_type;
pub use self::lua::lua_typename;
pub use self::lua::lua_upvalueid;
pub use self::lua::lua_upvalueindex;
pub use self::lua::lua_upvaluejoin;
pub use self::lua::lua_version;
pub use self::lua::lua_xmove;
pub use self::lua::lua_yield;
pub use self::lua::lua_yieldk;

pub use self::lauxlib::luaL_Buffer;
pub use self::lauxlib::luaL_Reg;
pub use self::lauxlib::luaL_Stream;

pub use self::lauxlib::luaL_addchar;
pub use self::lauxlib::luaL_addlstring;
pub use self::lauxlib::luaL_addsize;
pub use self::lauxlib::luaL_addstring;
pub use self::lauxlib::luaL_addvalue;
pub use self::lauxlib::luaL_argcheck;
pub use self::lauxlib::luaL_argerror;
pub use self::lauxlib::luaL_buffinit;
pub use self::lauxlib::luaL_buffinitsize;
pub use self::lauxlib::luaL_callmeta;
pub use self::lauxlib::luaL_checkany;
pub use self::lauxlib::luaL_checkint;
pub use self::lauxlib::luaL_checkinteger;
pub use self::lauxlib::luaL_checklong;
pub use self::lauxlib::luaL_checklstring;
pub use self::lauxlib::luaL_checknumber;
pub use self::lauxlib::luaL_checkoption;
pub use self::lauxlib::luaL_checkstack;
pub use self::lauxlib::luaL_checkstring;
pub use self::lauxlib::luaL_checktype;
pub use self::lauxlib::luaL_checkudata;
pub use self::lauxlib::luaL_checkversion;
pub use self::lauxlib::luaL_dofile;
pub use self::lauxlib::luaL_dostring;
pub use self::lauxlib::luaL_error;
pub use self::lauxlib::luaL_execresult;
pub use self::lauxlib::luaL_fileresult;
pub use self::lauxlib::luaL_getmetafield;
pub use self::lauxlib::luaL_getmetatable;
pub use self::lauxlib::luaL_getsubtable;
pub use self::lauxlib::luaL_gsub;
pub use self::lauxlib::luaL_len;
pub use self::lauxlib::luaL_loadbuffer;
pub use self::lauxlib::luaL_loadbufferx;
pub use self::lauxlib::luaL_loadfile;
pub use self::lauxlib::luaL_loadfilex;
pub use self::lauxlib::luaL_loadstring;
pub use self::lauxlib::luaL_newlib;
pub use self::lauxlib::luaL_newlibtable;
pub use self::lauxlib::luaL_newmetatable;
pub use self::lauxlib::luaL_newstate;
pub use self::lualib::luaL_openlibs;
pub use self::lauxlib::luaL_optint;
pub use self::lauxlib::luaL_optinteger;
pub use self::lauxlib::luaL_optlong;
pub use self::lauxlib::luaL_optlstring;
pub use self::lauxlib::luaL_optnumber;
pub use self::lauxlib::luaL_optstring;
pub use self::lauxlib::luaL_prepbuffer;
pub use self::lauxlib::luaL_prepbuffsize;
pub use self::lauxlib::luaL_pushresult;
pub use self::lauxlib::luaL_pushresultsize;
pub use self::lauxlib::luaL_ref;
pub use self::lauxlib::luaL_requiref;
pub use self::lauxlib::luaL_setfuncs;
pub use self::lauxlib::luaL_setmetatable;
pub use self::lauxlib::luaL_testudata;
pub use self::lauxlib::luaL_tolstring;
pub use self::lauxlib::luaL_traceback;
pub use self::lauxlib::luaL_typename;
pub use self::lauxlib::luaL_unref;
pub use self::lauxlib::luaL_where;

// commonly used constants from lua.h
pub use self::lua::LUA_MULTRET;
pub use self::lua::LUA_REGISTRYINDEX;

pub use self::lua::{LUA_RIDX_MAINTHREAD, LUA_RIDX_GLOBALS};

pub use self::lua::{LUA_OPADD, LUA_OPSUB, LUA_OPMUL, LUA_OPDIV, LUA_OPIDIV};
pub use self::lua::{LUA_OPMOD, LUA_OPPOW, LUA_OPUNM};
pub use self::lua::{LUA_OPBNOT, LUA_OPBAND, LUA_OPBOR, LUA_OPBXOR, LUA_OPSHL, LUA_OPSHR};
pub use self::lua::{LUA_OPEQ, LUA_OPLT, LUA_OPLE};

pub use self::lua::{LUA_OK, LUA_ERRRUN, LUA_ERRMEM, LUA_ERRERR, LUA_ERRGCMM};

pub use self::lua::{LUA_TNONE, LUA_TNIL, LUA_TNUMBER, LUA_TBOOLEAN, LUA_TSTRING};
pub use self::lua::{LUA_TTABLE, LUA_TFUNCTION, LUA_TUSERDATA, LUA_TTHREAD, LUA_TLIGHTUSERDATA};

pub use self::lua::{LUA_HOOKCALL, LUA_HOOKRET, LUA_HOOKTAILCALL, LUA_HOOKLINE, LUA_HOOKCOUNT};

pub use self::lua::{LUA_GCSTOP, LUA_GCRESTART, LUA_GCCOLLECT, LUA_GCCOUNT, LUA_GCCOUNTB};
pub use self::lua::{LUA_GCSTEP, LUA_GCSETPAUSE, LUA_GCSETSTEPMUL, LUA_GCISRUNNING};

// constants from lauxlib.h
pub use self::lauxlib::{LUA_REFNIL, LUA_NOREF};
pub use self::lauxlib::{LUA_ERRFILE, LUA_FILEHANDLE};

mod glue;
pub mod luaconf;
pub mod lua;
pub mod lauxlib;
pub mod lualib;

