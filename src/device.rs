use crate::proto::qconnect::{
    AudioQuality, DeviceCapabilities, DeviceInfo, DeviceType, VolumeRemoteControl,
};

/// How a device presents itself to a session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    pub uuid: [u8; 16],
    pub name: String,
    pub brand: String,
    pub model: String,
    pub kind: DeviceType,
    pub max_audio_quality: AudioQuality,
    pub volume_remote_control: bool,
    pub software_version: String,
}

impl Device {
    pub(crate) fn to_proto(&self) -> DeviceInfo {
        let volume_remote_control = if self.volume_remote_control {
            VolumeRemoteControl::Allowed
        } else {
            VolumeRemoteControl::NotAllowed
        };
        DeviceInfo {
            device_uuid: self.uuid.to_vec(),
            friendly_name: self.name.clone(),
            brand: self.brand.clone(),
            model: self.model.clone(),
            serial_number: String::new(),
            r#type: self.kind.into(),
            capabilities: Some(DeviceCapabilities {
                min_audio_quality: AudioQuality::Mp3.into(),
                max_audio_quality: self.max_audio_quality.into(),
                volume_remote_control: volume_remote_control.into(),
            }),
            software_version: self.software_version.clone(),
        }
    }

    pub(crate) fn from_proto(info: DeviceInfo) -> Self {
        let kind = info.r#type();
        let capabilities = info.capabilities.unwrap_or_default();
        Self {
            uuid: info.device_uuid.try_into().unwrap_or_default(),
            name: info.friendly_name,
            brand: info.brand,
            model: info.model,
            kind,
            max_audio_quality: capabilities.max_audio_quality(),
            volume_remote_control: capabilities.volume_remote_control()
                == VolumeRemoteControl::Allowed,
            software_version: info.software_version,
        }
    }
}
