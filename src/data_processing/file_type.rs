
use std::ffi::OsStr;
use std::marker::PhantomData;
use std::path::Path;
use crate::data_processing::data_model::{Data2d, Data2dStatistics, DataType};

pub trait VariableDescriptor = Clone;

pub trait DataFile<TVar: VariableDescriptor, TData: DataType> {
    fn extension() -> &'static OsStr;
    fn open(path: &Path) -> Result<Self, String> where Self: Sized;
    fn read_variables(&self, variables: &[TVar]) -> Vec<Data2dStatistics<TData>>;
}

pub struct Nc4<T: DataType> {
    path: String,
    contents: netcdf::File,
    t: PhantomData<T>,
}

impl<T: DataType + netcdf::Numeric> DataFile<String, T> for Nc4<T> {
    fn extension() -> &'static OsStr {
        OsStr::new("nc4")
    }

    fn open(path: &Path) -> Result<Self, String> {
        let path_str: String = path.to_str().unwrap().to_owned();
        match netcdf::open(path) {
            Ok(contents) => Ok(Self{ path: path_str, contents, t: Default::default() }),
            Err(error) => Err(format!("NC4 error occurred: {error:?}")),
        }
    }

    fn read_variables(&self, variables: &[String]) -> Vec<Data2dStatistics<T>> {
        println!("Reading data from {:?}. Variables: {:?}", self.path, variables);
        let mut all_data = Vec::new();

        for name in variables {
            if let Some(v) = self.contents.variable(name.as_str()) {
                all_data.push(Self::read_variable(&v))
            } else {
                panic!("Unknown variable {name} in file {:?}", self.path)
            }
        }
        assert_eq!(all_data.len(), variables.len());

        all_data
    }
}

impl<T: DataType + netcdf::Numeric> Nc4<T> {
    const TIME_DIMENSION: usize = 0;
    const LATITUDE_DIMENSION: usize = 1;
    const LONGITUDE_DIMENSION: usize = 2;

    fn read_variable(v: &netcdf::Variable) -> Data2dStatistics<T> {
        println!("Reading variable: {:?} (length = {})", v.name(), v.len());

        let dim = v.dimensions();
        assert_eq!(dim.len(), 3);
        assert_eq!(dim[Self::TIME_DIMENSION].len(), 1);

        let latitude_points = dim[Self::LATITUDE_DIMENSION].len();
        let longitude_points = dim[Self::LONGITUDE_DIMENSION].len();

        let mut data = Data2d::<T>::new(longitude_points, latitude_points);
        let mut min = None;
        let mut max = None;

        let mut valid: usize = 0;
        for row in 0..latitude_points {
            for column in 0..longitude_points {
                if let Ok(val) = v.value::<T>(Some(&[0, row, column])) {
                    if max.is_none() || val > max.unwrap() {
                        max = Some(val);
                    }
                    if min.is_none() || val < min.unwrap() {
                        min = Some(val);
                    }
                    data.rows[row].columns[column] = val;
                    valid += 1;
                }
            }
        }

        assert_eq!(valid, data.rows.len() * data.rows[0].columns.len());

        Data2dStatistics::new(data, min, max)
    }
}
