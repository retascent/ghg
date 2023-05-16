use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub struct ChannelMetadata {
	pub min: f64,
	pub max: f64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Metadata(pub Vec<ChannelMetadata>);

impl FromIterator<ChannelMetadata> for Metadata {
	fn from_iter<T: IntoIterator<Item = ChannelMetadata>>(iter: T) -> Self {
		Self(<Vec<ChannelMetadata> as FromIterator<ChannelMetadata>>::from_iter(iter))
	}
}

impl TryInto<(nglm::Vec3, nglm::Vec3)> for Metadata {
	type Error = String;

	fn try_into(self) -> Result<(nglm::Vec3, nglm::Vec3), Self::Error> {
		if self.0.len() != 3 {
			return Err(format!("Unexpected number of channels: {}", self.0.len()));
		}
		Ok((
			nglm::vec3(self.0[0].min as f32, self.0[1].min as f32, self.0[2].min as f32),
			nglm::vec3(self.0[0].max as f32, self.0[1].max as f32, self.0[2].max as f32),
		))
	}
}

impl TryInto<(nglm::Vec4, nglm::Vec4)> for Metadata {
	type Error = String;

	fn try_into(self) -> Result<(nglm::Vec4, nglm::Vec4), Self::Error> {
		if self.0.len() != 4 {
			return Err(format!("Unexpected number of channels: {}", self.0.len()));
		}
		Ok((
			nglm::vec4(
				self.0[0].min as f32,
				self.0[1].min as f32,
				self.0[2].min as f32,
				self.0[3].min as f32,
			),
			nglm::vec4(
				self.0[0].max as f32,
				self.0[1].max as f32,
				self.0[2].max as f32,
				self.0[3].max as f32,
			),
		))
	}
}
