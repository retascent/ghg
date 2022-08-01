# `earth_height` images

Images in the `earth_height` directory represent heightmap information of the Earth at various levels of detail. The
original image source is here: https://visibleearth.nasa.gov/images/73934/topography

Similar to OpenGL level-of-detail, the highest-detail images are in `earth_height/0`, and each level past that is 1/4
the area of the previous level (half width by half height). Within each of those folders are a few different things:

- `full.png`: The full-sized image at the specified level of detail.
- Directory `{N}x{M}`: A split-apart version of the full-sized image, split into `N` columns and `M` rows. Images within
  the directory are named by `{column_index}.{row_index}.png`.

# `earth_color` images

Similar to `earth_height`, except it provides original color images for the Earth in August 2004. The original image
source is
here: https://visibleearth.nasa.gov/images/73776/august-blue-marble-next-generation-w-topography-and-bathymetry
