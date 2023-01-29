use std::ops::{Add, Mul, Sub};
use std::thread;
use std::thread::JoinHandle;
use image::{ImageBuffer, Rgb};

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
        Vector {x, y, z }
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

fn render_scene(sphere: &Sphere, light_dir: &Vector, width: u32, height: u32) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let mut image = ImageBuffer::new(width, height);
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let ray = Ray {
            origin: Vector::new(x as f64, y as f64, 0.0),
            direction: Vector::new(0.0, 0.0, 1.0),
        };
        let hit_normal = match intersect_ray_sphere(&ray, &sphere) {
            Some(t) => t,
            None => Vector::new(0.0, 0.0, 0.0),
        };
        let light_intensity = light_dir.dot(&hit_normal).max(0.0);
        let color = Rgb([(light_intensity * 128.0) as u8, (light_intensity * 156.0) as u8, (light_intensity * 255.0) as u8]);
        *pixel = color;
    }
    image
}

fn main() {
    let mut ts:Vec<JoinHandle<()>> = Vec::new();
    let width = 1920.0;
    let height = 1080.0;
    let max_threads = 32;
    for x in 1..313 {
        let t =  thread::spawn( move || {
            let sphere = Sphere {
                center: Vector::new((width/2.0)+ (x as f64/10.0).sin()*50.0,height/2.0, 0.0),
                radius: 100.0*(0.1+(x as f64/100.0).sin().abs()),
            };
            let light_dir = Vector::new((x as f64/15.0).sin(), (x as f64/10.0).sin(), -(x as f64/10.0).cos());

            let image = render_scene(&sphere, &light_dir,width as u32, height as u32);
            println!("Rendering scene {:03}",x);
            image.save(format!("render{:03}.png", x)).unwrap();
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

    // ffmpeg -framerate 30 -pattern_type glob -i '*.png' \
    //   -c:v libx264 -pix_fmt yuv420p out.mp4
}