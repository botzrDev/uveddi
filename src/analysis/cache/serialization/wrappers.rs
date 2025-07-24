//! Wrappers for rkyv serialization of non-Archive-compatible std types.

use rkyv::{Archive, Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::path::PathBuf;

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ArchivableSystemTime(pub SystemTime);

impl From<SystemTime> for ArchivableSystemTime {
    fn from(time: SystemTime) -> Self {
        Self(time)
    }
}

impl From<ArchivableSystemTime> for SystemTime {
    fn from(time: ArchivableSystemTime) -> Self {
        time.0
    }
}

impl std::ops::Deref for ArchivableSystemTime {
    type Target = SystemTime;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Archive for ArchivableSystemTime {
    type Archived = rkyv::Archived<u128>;
    type Resolver = rkyv::Resolver<u128>;

    unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
        let nanos = self.0.duration_since(UNIX_EPOCH).unwrap().as_nanos();
        nanos.resolve(pos, resolver, out);
    }
}

impl<S: rkyv::ser::Serializer + ?Sized> Serialize<S> for ArchivableSystemTime {
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        let nanos = self.0.duration_since(UNIX_EPOCH).unwrap().as_nanos();
        nanos.serialize(serializer)
    }
}

use rkyv::de::deserializers::SharedDeserializeMap;
impl Deserialize<ArchivableSystemTime, SharedDeserializeMap> for rkyv::Archived<u128> {
    fn deserialize(&self, deserializer: &mut SharedDeserializeMap) -> Result<ArchivableSystemTime, rkyv::de::deserializers::SharedDeserializeMapError> {
        let nanos: u128 = rkyv::Deserialize::deserialize(self, deserializer)?;
        let duration = Duration::from_nanos(nanos as u64); // Note: truncation for extreme future dates
        Ok(ArchivableSystemTime(UNIX_EPOCH + duration))
    }
}

// Serde implementations for ArchivableSystemTime
impl serde::Serialize for ArchivableSystemTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let nanos = self.0.duration_since(UNIX_EPOCH)
            .map_err(|e| serde::ser::Error::custom(format!("SystemTime error: {}", e)))?
            .as_nanos();
        serializer.serialize_u128(nanos)
    }
}

impl<'de> serde::Deserialize<'de> for ArchivableSystemTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let nanos = <u128 as serde::Deserialize>::deserialize(deserializer)?;
        let duration = Duration::from_nanos(nanos as u64);
        Ok(ArchivableSystemTime(UNIX_EPOCH + duration))
    }
}

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ArchivablePathBuf(pub PathBuf);

impl From<PathBuf> for ArchivablePathBuf {
    fn from(path: PathBuf) -> Self {
        Self(path)
    }
}

impl From<ArchivablePathBuf> for PathBuf {
    fn from(path: ArchivablePathBuf) -> Self {
        path.0
    }
}

impl std::ops::Deref for ArchivablePathBuf {
    type Target = PathBuf;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Archive for ArchivablePathBuf {
    type Archived = rkyv::Archived<String>;
    type Resolver = rkyv::Resolver<String>;

    unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
        let string = self.0.to_string_lossy().into_owned();
        string.resolve(pos, resolver, out);
    }
}

impl<S: rkyv::ser::Serializer + ?Sized> Serialize<S> for ArchivablePathBuf {
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        let string = self.0.to_string_lossy().into_owned();
        string.serialize(serializer)
    }
}

impl Deserialize<ArchivablePathBuf, SharedDeserializeMap> for rkyv::Archived<String> {
    fn deserialize(&self, deserializer: &mut SharedDeserializeMap) -> Result<ArchivablePathBuf, rkyv::de::deserializers::SharedDeserializeMapError> {
        let string: String = rkyv::Deserialize::deserialize(self, deserializer)?;
        Ok(ArchivablePathBuf(PathBuf::from(string)))
    }
}

// Serde implementations for ArchivablePathBuf
impl serde::Serialize for ArchivablePathBuf {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let string = self.0.to_string_lossy();
        serializer.serialize_str(&string)
    }
}

impl<'de> serde::Deserialize<'de> for ArchivablePathBuf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string = <String as serde::Deserialize>::deserialize(deserializer)?;
        Ok(ArchivablePathBuf(PathBuf::from(string)))
    }
}
