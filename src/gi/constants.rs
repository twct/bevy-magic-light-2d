use bevy::prelude::*;

pub const GI_SCREEN_PROBE_SIZE: i32 = 8;

pub const SHADER_GI_CAMERA: Handle<Shader> = Handle::weak_from_u128(1371231089456109822);
pub const SHADER_GI_TYPES: Handle<Shader> = Handle::weak_from_u128(4462033275253590181);
pub const SHADER_GI_ATTENUATION: Handle<Shader> = Handle::weak_from_u128(5254739165481917368);
pub const SHADER_GI_HALTON: Handle<Shader> = Handle::weak_from_u128(1287391288877821366);
pub const SHADER_GI_MATH: Handle<Shader> = Handle::weak_from_u128(2387462894328787238);
pub const SHADER_GI_RAYMARCH: Handle<Shader> = Handle::weak_from_u128(9876835068496322894);

pub const SHADER_PIPELINE_SDF: Handle<Shader> = Handle::weak_from_u128(9876835068496322895);
pub const SHADER_PIPELINE_SS_PROBE: Handle<Shader> = Handle::weak_from_u128(9876835068496322896);
pub const SHADER_PIPELINE_SS_BOUNCE: Handle<Shader> = Handle::weak_from_u128(9876835068496322897);
pub const SHADER_PIPELINE_SS_BLEND: Handle<Shader> = Handle::weak_from_u128(9876835068496322898);
pub const SHADER_PIPELINE_SS_FILTER: Handle<Shader> = Handle::weak_from_u128(9876835068496322899);
pub const SHADER_POST_PROCESS: Handle<Shader> = Handle::weak_from_u128(9876835068496322900);
