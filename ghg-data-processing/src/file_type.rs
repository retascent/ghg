use std::default::Default;
use std::ffi::OsStr;
use std::marker::PhantomData;
use std::path::Path;

use crate::data_model::{Data2d, Data2dStatistics, DataType};

pub trait VariableDescriptor = Clone;

pub trait Metadata = Clone;

pub trait DataFile<TVar: VariableDescriptor, TData: DataType, TMetadata: Metadata> {
	fn extension() -> &'static OsStr;
	fn open(path: &Path, metadata: TMetadata) -> Result<Self, String>
	where
		Self: Sized;
	fn read_variables(&self, variables: &[TVar]) -> Vec<Data2dStatistics<TData>>;
}

#[derive(Debug)]
struct CdfReadableData<T: DataType> {
	/// Shared implementation of a netcdf-readable file
	path: String,
	contents: netcdf::File,
	metadata: CdfMetadata,
	t: PhantomData<T>,
}

#[derive(Copy, Clone, Debug)]
pub struct CdfMetadata {
	pub width_dimension: usize,
	pub height_dimension: usize,
}

#[derive(Debug)]
pub struct Nc<T: DataType> {
	/// *.nc files
	data: CdfReadableData<T>,
}

impl<T: DataType + netcdf::NcPutGet> DataFile<String, T, CdfMetadata> for Nc<T> {
	fn extension() -> &'static OsStr { OsStr::new("nc") }

	fn open(path: &Path, metadata: CdfMetadata) -> Result<Self, String>
	where
		Self: Sized,
	{
		match CdfReadableData::open(path, metadata) {
			Ok(data) => Ok(Self { data }),
			Err(error) => Err(error),
		}
	}

	fn read_variables(&self, variables: &[String]) -> Vec<Data2dStatistics<T>> {
		self.data.read_variables(variables)
	}
}

#[derive(Debug)]
pub struct Nc4<T: DataType> {
	/// *.nc4 files
	data: CdfReadableData<T>,
}

impl<T: DataType + netcdf::NcPutGet> DataFile<String, T, CdfMetadata> for Nc4<T> {
	fn extension() -> &'static OsStr { OsStr::new("nc4") }

	fn open(path: &Path, metadata: CdfMetadata) -> Result<Self, String>
	where
		Self: Sized,
	{
		match CdfReadableData::open(path, metadata) {
			Ok(data) => Ok(Self { data }),
			Err(error) => Err(error),
		}
	}

	fn read_variables(&self, variables: &[String]) -> Vec<Data2dStatistics<T>> {
		self.data.read_variables(variables)
	}
}

impl<T: DataType + netcdf::NcPutGet> CdfReadableData<T> {
	fn open(path: &Path, metadata: CdfMetadata) -> Result<Self, String> {
		let path_str: String = path.to_str().unwrap().to_owned();
		match netcdf::open(path) {
			Ok(contents) => Ok(Self { path: path_str, contents, metadata, t: Default::default() }),
			Err(error) => Err(format!("CDF open error occurred: {error:?}")),
		}
	}

	fn read_variables(&self, variables: &[String]) -> Vec<Data2dStatistics<T>> {
		println!("Reading data from {:?}. Variables: {:?}", self.path, variables);
		let mut all_data = Vec::new();

		if variables.len() > 0 {
			for name in variables {
				if let Some(v) = self.contents.variable(name.as_str()) {
					all_data.push(self.read_variable(&v))
				} else {
					panic!("Unknown variable {name} in file {:?}", self.path)
				}
			}
			assert_eq!(all_data.len(), variables.len());
		} else {
			println!("No variables specified; reading all available variables");
			for v in self.contents.variables() {
				all_data.push(self.read_variable(&v))
			}
		}

		all_data
	}
}

impl<T: DataType + netcdf::NcPutGet> CdfReadableData<T> {
	fn read_variable(&self, v: &netcdf::Variable) -> Data2dStatistics<T> {
		println!("Reading variable: {:?} (length = {})", v.name(), v.len());

		let dim = v.dimensions();
		if dim.len() >= 2 {
			return self.read_2d_variable(v);
		} else {
			return self.read_1d_variable(v);
		}
	}

	fn read_2d_variable(&self, v: &netcdf::Variable) -> Data2dStatistics<T> {
		let dim = v.dimensions();
		assert!(dim.len() >= 2);

		let mut indices = vec![0; dim.len()];

		let height = dim[self.metadata.height_dimension].len();
		let width = dim[self.metadata.width_dimension].len();

		let mut data = Data2d::<T>::new(width, height);
		let mut min = None;
		let mut max = None;

		let mut valid: usize = 0;
		for row in 0..height {
			for column in 0..width {
				indices[self.metadata.height_dimension] = row;
				indices[self.metadata.width_dimension] = column;
				if let Some((val, new_min, new_max)) = Self::stat_cell(v, &indices, min, max) {
					data.rows[row].columns[column] = val;
					min = new_min;
					max = new_max;
					valid += 1;
				}
			}
		}

		assert_eq!(valid, data.height() * data.width());

		Data2dStatistics { name: v.name(), data, min, max }
	}

	fn read_1d_variable(&self, v: &netcdf::Variable) -> Data2dStatistics<T> {
		let dim = v.dimensions();
		let width = dim[self.metadata.width_dimension].len();
		let variable_name = v.name();
		assert!(width > 0, "{}", format!("Invalid width for 1D variable {variable_name}: {width}"));

		let mut data = Data2d::<T>::new(width, 1);
		let mut min = None;
		let mut max = None;

		let mut valid: usize = 0;
		for column in 0..width {
			if let Some((val, new_min, new_max)) = Self::stat_cell(v, &[column], min, max) {
				data.rows[0].columns[column] = val;
				min = new_min;
				max = new_max;
				valid += 1;
			}
		}

		assert_eq!(valid, data.width());

		Data2dStatistics { name: v.name(), data, min, max }
	}

	fn stat_cell(
		v: &netcdf::Variable,
		indices: &[usize],
		mut min: Option<T>,
		mut max: Option<T>,
	) -> Option<(T, Option<T>, Option<T>)> {
		if let Ok(val) = v.value::<T, &[usize]>(indices) {
			if max.is_none() || val > max.unwrap() {
				max = Some(val);
			}
			if min.is_none() || val < min.unwrap() {
				min = Some(val);
			}
			return Some((val, min, max));
		}
		return None;
	}
}
