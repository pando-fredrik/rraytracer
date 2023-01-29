use std::ops::{Add, Mul, Sub};
use std::thread;
use std::thread::JoinHandle;
use image::{ImageBuffer, Pixel, Rgb};

struct Ray {
    origin: Vector,
    direction: Vector,
}

#[derive(Copy, Clone)]
struct Vector {
    x: f64,
    y: f64,
    z: f64,
}

impl Add<Vector> for Vector {
    type Output = Vector;

    fn add(self, rhs: Vector) -> Vector {
        Vector {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Mul<f64> for Vector {
    type Output = Vector;

    fn mul(self, rhs: f64) -> Vector {
        Vector {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Sub for Vector {
    type Output = Vector;

    fn sub(self, other: Vector) -> Vector {
        Vector {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}


impl Vector {
    fn new(x: f64, y: f64, z: f64) -> Vector {
        Vector { x, y, z }
    }

    fn normalize(&self) -> Vector {
        let length = self.length();
        Vector {
            x: self.x / length,
            y: self.y / length,
            z: self.z / length,
        }
    }

    fn dot(&self, other: &Vector) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    fn length_squared(&self) -> f64 {
        self.dot(self)
    }

    fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }
}

struct Sphere {
    center: Vector,
    radius: f64,
    r: f64,
    g: f64,
    b: f64,
    id: u32,
}

fn intersect_ray_sphere(ray: &Ray, sphere: &Sphere) -> Option<Vector> {
    let l = sphere.center - ray.origin;
    let angle = l.dot(&ray.direction);
    if angle < 0.0 {
        return None;
    }
    let d2 = l.length_squared() - angle * angle;
    let r2 = sphere.radius * sphere.radius;
    if d2 > r2 {
        return None;
    }
    let half_angle = (r2 - d2).sqrt();
    let t0 = angle - half_angle;

    let hit_point = ray.origin + ray.direction * t0;
    let hit_normal = (hit_point - sphere.center).normalize();

    Some(hit_normal)
}

fn render_scene(scene: &Vec<Sphere>, light_dir: &Vector, width: u32, height: u32) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let mut image: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    for sphere in scene {
        println!("Render sphere {}", sphere.id);
        for (x, y, pixel) in image.enumerate_pixels_mut() {
            let ray = Ray {
                origin: Vector::new(x as f64, y as f64, 0.0),
                direction: Vector::new(0.0, 0.0, 1.0),
            };
            let hit_normal = match intersect_ray_sphere(&ray, sphere) {
                Some(t) => t,
                None => Vector::new(0.0, 0.0, 0.0),
            };
            let light_intensity = light_dir.dot(&hit_normal).max(0.0);
            let color = Rgb([(light_intensity * sphere.r) as u8, (light_intensity * sphere.g) as u8, (light_intensity * sphere.b) as u8]);
            let rgb = color.channels();
            if rgb[0] > 0 || rgb[1] > 1 || rgb[2] > 0 {
                pixel.blend(&color);
            }
        }
    }
    //image::imageops::blur(&mut final_img, 255.0);

    image
}

fn main() {
    let mut ts: Vec<JoinHandle<()>> = Vec::new();
    let width = 1920.0;
    let height = 1080.0;
    let max_threads = 32;
    for x in 1..313 { //1..313 {
        let t = thread::spawn(move || {
            let mut scene = Vec::new();


            let sphere3 = Sphere {
                center: Vector::new((width / 1.2) + -(x as f64 / 10.0).cos() * 80.0, height / 2.0, 0.1 + (x as f64 / 160.0).cos().abs()),
                radius: 100.0 * (0.1 + (x as f64 / 100.0).cos().abs()),
                r: 255.0,
                g: 255.0,
                b: 0.0,
                id: 2,
            };
            scene.push(sphere3);


            let sphere2 = Sphere {
                center: Vector::new((width / 5.0) + (x as f64 / 20.0).sin() * 30.0, height / 2.0, 0.1 + (x as f64 / 100.0).sin().abs()),
                radius: 100.0 * (1.5 + (x as f64 / 100.0).sin().abs()),
                r: 255.0,
                g: 0.0,
                b: 0.0,
                id: 1,
            };
            scene.push(sphere2);

            let sphere = Sphere {
                center: Vector::new((width / 2.0) + (x as f64 / 10.0).sin() * 50.0, height / 2.0, 0.1 + (x as f64 / 100.0).cos().abs()),
                radius: 100.0 * (0.1 + (x as f64 / 100.0).sin().abs()),
                r: 128.0,
                g: 156.0,
                b: 255.0,
                id: 0,
            };
            scene.push(sphere);

            let light_dir = Vector::new((x as f64 / 15.0).sin(), (x as f64 / 10.0).sin(), -(x as f64 / 10.0).cos());

            let image = render_scene(&scene, &light_dir, width as u32, height as u32);
            println!("Rendering scene {x:03}");
            image.save(format!("render{x:03}.png")).unwrap();
        });
        ts.push(t);
        if ts.len() > max_threads {
            println!("Hack... waiting for threads");
            for t in ts.into_iter() {
                t.join().unwrap();
            }
            ts = Vec::new();
        }
    }
    for t in ts.into_iter() {
        t.join().unwrap();
    }
    // ffmpeg -framerate 30 -pattern_type glob -i '*.png' \
    //   -c:v libx264 -pix_fmt yuv420p out.mp4
}