use std::ops::Sub;

use ghg_data_core::metadata::{ChannelMetadata, Metadata};
use image::{
	GrayAlphaImage, GrayImage, ImageBuffer, Luma, LumaA, Pixel, Rgb, RgbImage, Rgba, RgbaImage,
};
use itertools::izip;

pub trait DataType = Copy + Clone + Default + PartialOrd + Sub<Output = Self>;

#[derive(Clone)]
pub struct Data1d<T: DataType> {
	pub columns: Vec<T>,
}

impl<T: DataType> Data1d<T> {
	fn new(size: usize) -> Self {
		let mut columns = Vec::with_capacity(size);
		columns.resize(size, T::default());
		Self { columns }
	}

	fn width(&self) -> usize { self.columns.len() }
}

#[derive(Clone)]
pub struct Data2d<T: DataType> {
	pub rows: Vec<Data1d<T>>,
}

impl<T: DataType> Data2d<T> {
	pub fn new(width: usize, height: usize) -> Self {
		let mut rows = Vec::with_capacity(height);
		rows.resize(height, Data1d::new(width));
		Self { rows }
	}

	pub fn width(&self) -> usize {
		if self.rows.len() == 0 {
			return 0;
		}
		self.rows[0].width()
	}

	pub fn height(&self) -> usize { self.rows.len() }
}

#[derive(Clone)]
pub struct Data2dStatistics<T: DataType> {
	pub name: String,
	pub data: Data2d<T>,
	pub min: Option<T>,
	pub max: Option<T>,
}

pub trait PixelMappable<T: DataType> {
	fn get_pixel_map(&self) -> Box<dyn Fn(&T) -> u8>;
}

impl PixelMappable<f64> for Data2dStatistics<f64> {
	fn get_pixel_map(&self) -> Box<dyn Fn(&f64) -> u8> {
		let range = self.max.unwrap() - self.min.unwrap();
		let offset = self.min.unwrap();
		Box::new(move |value: &f64| {
			let portion = (*value - offset) / range;
			(255.0 * portion) as u8
		})
	}
}

impl<T: DataType> Sub for &Data2dStatistics<T>
where
	T: Sub<Output = T>,
{
	type Output = Data2dStatistics<T>;

	fn sub(self, rhs: Self) -> Self::Output {
		let mut difference = Data2d::new(self.data.width(), self.data.height());
		let mut min = None;
		let mut max = None;

		for (row, (a_row, b_row)) in self.data.rows.iter().zip(rhs.data.rows.iter()).enumerate() {
			for (col, (a_val, b_val)) in a_row.columns.iter().zip(b_row.columns.iter()).enumerate()
			{
				let val_difference = *b_val - *a_val;
				if max.is_none() || val_difference > max.unwrap() {
					max = Some(val_difference);
				}
				if min.is_none() || val_difference < min.unwrap() {
					min = Some(val_difference);
				}
				difference.rows[row].columns[col] = val_difference;
			}
		}

		Data2dStatistics {
			name: format!("{} - {}", self.name, rhs.name),
			data: difference,
			min,
			max,
		}
	}
}

pub trait ToImage<P: Pixel> {
	type Data;
	fn width(&self) -> usize;
	fn height(&self) -> usize;
	fn to_image(&self) -> ImageBuffer<P, Vec<P::Subpixel>>;
}

impl<T: DataType> ToImage<Luma<u8>> for Data2dStatistics<T>
where
	Data2dStatistics<T>: PixelMappable<T>,
{
	type Data = T;

	fn width(&self) -> usize { self.data.width() }

	fn height(&self) -> usize { self.data.height() }

	fn to_image(&self) -> ImageBuffer<Luma<u8>, Vec<u8>> {
		let pixel_map = PixelMappable::<T>::get_pixel_map(self);
		let mut output_buffer = Vec::with_capacity(self.width() * self.height());
		for row in self.data.rows.iter().rev() {
			for val in row.columns.iter() {
				output_buffer.push(pixel_map(val));
			}
		}

		GrayImage::from_raw(self.width() as u32, self.height() as u32, output_buffer)
			.expect("Failed to create image!")
	}
}

impl<T: DataType> ToImage<Luma<u8>> for [Data2dStatistics<T>; 1]
where
	Data2dStatistics<T>: PixelMappable<T>,
{
	type Data = T;

	fn width(&self) -> usize { self[0].width() }

	fn height(&self) -> usize { self[0].height() }

	fn to_image(&self) -> ImageBuffer<Luma<u8>, Vec<u8>> { return self[0].to_image() }
}

impl<T: DataType> ToImage<LumaA<u8>> for [Data2dStatistics<T>; 2]
where
	Data2dStatistics<T>: PixelMappable<T>,
{
	type Data = T;

	fn width(&self) -> usize {
		assert_eq!(self[0].width(), self[1].width());
		self[0].width()
	}

	fn height(&self) -> usize {
		assert_eq!(self[0].height(), self[1].height());
		self[0].height()
	}

	fn to_image(&self) -> ImageBuffer<LumaA<u8>, Vec<u8>> {
		let pixel_maps: Vec<Box<dyn Fn(&T) -> u8>> =
			self.iter().map(|ds| PixelMappable::<T>::get_pixel_map(ds)).collect();
		let mut output_buffer = Vec::with_capacity(2 * self.width() * self.height());
		for (row1, row2) in self[0].data.rows.iter().rev().zip(self[1].data.rows.iter().rev()) {
			for (val0, val1) in row1.columns.iter().zip(row2.columns.iter()) {
				output_buffer.push(pixel_maps[0](val0));
				output_buffer.push(pixel_maps[1](val1));
			}
		}

		GrayAlphaImage::from_raw(self.width() as u32, self.height() as u32, output_buffer)
			.expect("Failed to create image!")
	}
}

impl<T: DataType> ToImage<Rgb<u8>> for [Data2dStatistics<T>; 3]
where
	Data2dStatistics<T>: PixelMappable<T>,
{
	type Data = T;

	fn width(&self) -> usize {
		assert_eq!(self[0].width(), self[1].width());
		assert_eq!(self[0].width(), self[2].width());
		self[0].width()
	}

	fn height(&self) -> usize {
		assert_eq!(self[0].height(), self[1].height());
		assert_eq!(self[0].height(), self[2].height());
		self[0].height()
	}

	fn to_image(&self) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
		let pixel_maps: Vec<Box<dyn Fn(&T) -> u8>> =
			self.iter().map(|ds| PixelMappable::<T>::get_pixel_map(ds)).collect();
		let mut output_buffer = Vec::with_capacity(3 * self.width() * self.height());
		for rows in izip!(
			self[0].data.rows.iter().rev(),
			self[1].data.rows.iter().rev(),
			self[2].data.rows.iter().rev()
		) {
			for vals in izip!(rows.0.columns.iter(), rows.1.columns.iter(), rows.2.columns.iter()) {
				output_buffer.push(pixel_maps[0](vals.0));
				output_buffer.push(pixel_maps[1](vals.1));
				output_buffer.push(pixel_maps[2](vals.2));
			}
		}

		RgbImage::from_raw(self.width() as u32, self.height() as u32, output_buffer)
			.expect("Failed to create image!")
	}
}

impl<T: DataType> ToImage<Rgba<u8>> for [Data2dStatistics<T>; 4]
where
	Data2dStatistics<T>: PixelMappable<T>,
{
	type Data = T;

	fn width(&self) -> usize {
		assert_eq!(self[0].width(), self[1].width());
		assert_eq!(self[0].width(), self[2].width());
		assert_eq!(self[0].width(), self[3].width());
		self[0].width()
	}

	fn height(&self) -> usize {
		assert_eq!(self[0].height(), self[1].height());
		assert_eq!(self[0].height(), self[2].height());
		assert_eq!(self[0].height(), self[3].height());
		self[0].height()
	}

	fn to_image(&self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
		let pixel_maps: Vec<Box<dyn Fn(&T) -> u8>> =
			self.iter().map(|ds| PixelMappable::<T>::get_pixel_map(ds)).collect();
		let mut output_buffer = Vec::with_capacity(3 * self.width() * self.height());
		for rows in izip!(
			self[0].data.rows.iter().rev(),
			self[1].data.rows.iter().rev(),
			self[2].data.rows.iter().rev(),
			self[3].data.rows.iter().rev(),
		) {
			for vals in izip!(
				rows.0.columns.iter(),
				rows.1.columns.iter(),
				rows.2.columns.iter(),
				rows.3.columns.iter()
			) {
				output_buffer.push(pixel_maps[0](vals.0));
				output_buffer.push(pixel_maps[1](vals.1));
				output_buffer.push(pixel_maps[2](vals.2));
				output_buffer.push(pixel_maps[3](vals.3));
			}
		}

		RgbaImage::from_raw(self.width() as u32, self.height() as u32, output_buffer)
			.expect("Failed to create image!")
	}
}
pub trait ToMetadata {
	fn to_metadata(&self) -> Metadata;
}

impl ToMetadata for [Data2dStatistics<f64>; 3] {
	fn to_metadata(&self) -> Metadata {
		self.iter()
			.map(|ds| ChannelMetadata { min: ds.min.unwrap(), max: ds.max.unwrap() })
			.collect()
	}
}

impl ToMetadata for [Data2dStatistics<f64>; 4] {
	fn to_metadata(&self) -> Metadata {
		self.iter()
			.map(|ds| ChannelMetadata { min: ds.min.unwrap(), max: ds.max.unwrap() })
			.collect()
	}
}
