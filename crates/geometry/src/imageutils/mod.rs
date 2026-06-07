use image::RgbImage;
use std::f64::consts::PI;

/// Extracts a flat rectilinear image from an equirectangular panorama
/// `yaw_deg`: 0 = Front, 90 = Right, 180 = Back, 270 = Left
pub fn extract_flat_view(
    pano: &RgbImage,
    out_width: u32,
    out_height: u32,
    yaw_deg: f64,
    fov_deg: f64,
) -> RgbImage {
    let mut out_img = RgbImage::new(out_width, out_height);

    let pano_w = pano.width() as f64;
    let pano_h = pano.height() as f64;

    let yaw_rad = yaw_deg.to_radians();
    // Focal length based on Field of View (90 degrees is standard)
    let focal_len = (out_width as f64 / 2.0) / (fov_deg.to_radians() / 2.0).tan();

    for y in 0..out_height {
        for x in 0..out_width {
            // 1. Map target pixel to 3D camera plane (Z is forward)
            let cx = (x as f64) - (out_width as f64 / 2.0);
            let cy = (y as f64) - (out_height as f64 / 2.0);
            let cz = focal_len;

            // 2. Rotate the 3D vector by our desired Yaw
            let rx = cx * yaw_rad.cos() + cz * yaw_rad.sin();
            let ry = cy; // Keeping pitch at 0 (horizon level)
            let rz = -cx * yaw_rad.sin() + cz * yaw_rad.cos();

            // 3. Convert 3D vector to Spherical Coordinates (Longitude/Latitude)
            let theta = rx.atan2(rz); // Longitude
            let phi = ry.atan2((rx * rx + rz * rz).sqrt()); // Latitude

            // 4. Map Spherical Coordinates to the raw Panorama X/Y pixels
            let pano_x = (theta / (2.0 * PI) + 0.5) * pano_w;
            let pano_y = (phi / PI + 0.5) * pano_h;

            // Constrain to image bounds safely
            let px = (pano_x as u32).clamp(0, pano.width() - 1);
            let py = (pano_y as u32).clamp(0, pano.height() - 1);

            // 5. Copy the pixel!
            out_img.put_pixel(x, y, *pano.get_pixel(px, py));
        }
    }
    out_img
}
