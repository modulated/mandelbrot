use num_complex::{Complex64};
use std::{str::FromStr, usize, vec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 6 {
        println!("Error: arguments not matching.");
        println!("mandelbrot FILE PIXELS CENTER STEPS ZOOM_PERCENT ITERATIONS");
        std::process::exit(1);
    }

    let filename = &args[0];
    let pixel_dimensions = parse_pair::<usize>(&args[1], 'x').expect("Unable to parse bounds.");
    let centre_point = parse_complex(&args[2]).expect("Unable to parse upper left corner.");
    let zoom_steps = args[3].parse::<u32>().expect("Unable to parse zoom steps");
    let zoom_percent = (args[4].parse::<u32>().expect("Unable to parse iterations.")) as f64/100.0;
    let iterations = args[5].parse::<u64>().expect("Unable to parse iterations.");
    let split_file = filename.split('.').collect::<Vec<&str>>();

    let mut current_span = 10.0;

    println!{"Mandelbrot zoom. Resolution {:?}, centre coordinate {}", pixel_dimensions, centre_point};

    for i in 0 .. zoom_steps {
        
        println!("Generating image {}, zoom level: {}", i, current_span);
        let upper_left = Complex64::new(centre_point.re - current_span / 2.0, centre_point.im + current_span / 2.0);
        let lower_right = Complex64::new(centre_point.re + current_span / 2.0, centre_point.im - current_span / 2.0);
        let this_file =  format!("{}-{:03}.{}", split_file[0], i, split_file[1]);

        println!("Rendering file {}. upper_left {}, lower_right {}", this_file, upper_left, lower_right);


        render_image(pixel_dimensions, upper_left, lower_right, &iterations, &this_file)?;
        
        current_span = current_span * zoom_percent;
    }
    
    Ok(())
}

fn render_image(bounds: (usize, usize), upper_left: Complex64, lower_right: Complex64, iterations: &u64, filename: &str) -> Result<(), Box<dyn std::error::Error>> {

    let mut pixels: Vec<Vec<u8>> = vec![vec![0,0,0]; bounds.0 * bounds.1 * 3];
    let threads = 8;
    let rows_per_band = bounds.1 / threads + 1;

    

    {
        let bands: Vec<&mut [Vec<u8>]> = pixels.chunks_mut(rows_per_band * bounds.0).collect();
        
        let _res = crossbeam::scope(|spawner| {
            for (i, band) in bands.into_iter().enumerate() {
                let top = rows_per_band * i;
                let height = band.len() / bounds.0;
                let band_bounds = (bounds.0, height);
                let band_upper_left = pixel_to_point(bounds, (0, top), upper_left, lower_right);
                let band_lower_right = pixel_to_point(bounds, (bounds.0, top + height), upper_left, lower_right);
                spawner.spawn(move |_| {
                    render(band, band_bounds, band_upper_left, band_lower_right, &iterations);
                });
            }
        });
    }

    // let flat: Vec<u8> = pixels.into_iter().flatten().collect();
    
    write_image(filename, &pixels, bounds).expect("Error writing PNG.");

    Ok(())
}

fn render(pixels: &mut [Vec<u8>], bounds: (usize, usize), upper_left: Complex64, lower_right: Complex64, iterations: &u64){
    assert!(pixels.len() == bounds.0 * bounds.1);

    let black = vec![0u8,0u8,0u8];
    let colormap = gen_colormap();

    for row in 0 .. bounds.1 {
        for column in 0 .. bounds.0 {
            let point = pixel_to_point(bounds, (column, row), upper_left, lower_right);
            
            pixels[row * bounds.0 + column] = match escape_time(point, iterations) {
                None => black.clone(),
                Some(count) => colormap[(count % (std::u8::MAX as u64)) as usize].clone()
            };
        }
    }
}

fn gen_colormap() -> Vec<Vec<u8>> {
    let mut result = Vec::with_capacity(256);
    let data = [(48,18,59),(50,21,67),(51,24,74),(52,27,81),(53,30,88),(54,33,95),(55,36,102),(56,39,109),(57,42,115),(58,45,121),(59,47,128),(60,50,134),(61,53,139),(62,56,145),(63,59,151),(63,62,156),(64,64,162),(65,67,167),(65,70,172),(66,73,177),(66,75,181),(67,78,186),(68,81,191),(68,84,195),(68,86,199),(69,89,203),(69,92,207),(69,94,211),(70,97,214),(70,100,218),(70,102,221),(70,105,224),(70,107,227),(71,110,230),(71,113,233),(71,115,235),(71,118,238),(71,120,240),(71,123,242),(70,125,244),(70,128,246),(70,130,248),(70,133,250),(70,135,251),(69,138,252),(69,140,253),(68,143,254),(67,145,254),(66,148,255),(65,150,255),(64,153,255),(62,155,254),(61,158,254),(59,160,253),(58,163,252),(56,165,251),(55,168,250),(53,171,248),(51,173,247),(49,175,245),(47,178,244),(46,180,242),(44,183,240),(42,185,238),(40,188,235),(39,190,233),(37,192,231),(35,195,228),(34,197,226),(32,199,223),(31,201,221),(30,203,218),(28,205,216),(27,208,213),(26,210,210),(26,212,208),(25,213,205),(24,215,202),(24,217,200),(24,219,197),(24,221,194),(24,222,192),(24,224,189),(25,226,187),(25,227,185),(26,228,182),(28,230,180),(29,231,178),(31,233,175),(32,234,172),(34,235,170),(37,236,167),(39,238,164),(42,239,161),(44,240,158),(47,241,155),(50,242,152),(53,243,148),(56,244,145),(60,245,142),(63,246,138),(67,247,135),(70,248,132),(74,248,128),(78,249,125),(82,250,122),(85,250,118),(89,251,115),(93,252,111),(97,252,108),(101,253,105),(105,253,102),(109,254,98),(113,254,95),(117,254,92),(121,254,89),(125,255,86),(128,255,83),(132,255,81),(136,255,78),(139,255,75),(143,255,73),(146,255,71),(150,254,68),(153,254,66),(156,254,64),(159,253,63),(161,253,61),(164,252,60),(167,252,58),(169,251,57),(172,251,56),(175,250,55),(177,249,54),(180,248,54),(183,247,53),(185,246,53),(188,245,52),(190,244,52),(193,243,52),(195,241,52),(198,240,52),(200,239,52),(203,237,52),(205,236,52),(208,234,52),(210,233,53),(212,231,53),(215,229,53),(217,228,54),(219,226,54),(221,224,55),(223,223,55),(225,221,55),(227,219,56),(229,217,56),(231,215,57),(233,213,57),(235,211,57),(236,209,58),(238,207,58),(239,205,58),(241,203,58),(242,201,58),(244,199,58),(245,197,58),(246,195,58),(247,193,58),(248,190,57),(249,188,57),(250,186,57),(251,184,56),(251,182,55),(252,179,54),(252,177,54),(253,174,53),(253,172,52),(254,169,51),(254,167,50),(254,164,49),(254,161,48),(254,158,47),(254,155,45),(254,153,44),(254,150,43),(254,147,42),(254,144,41),(253,141,39),(253,138,38),(252,135,37),(252,132,35),(251,129,34),(251,126,33),(250,123,31),(249,120,30),(249,117,29),(248,114,28),(247,111,26),(246,108,25),(245,105,24),(244,102,23),(243,99,21),(242,96,20),(241,93,19),(240,91,18),(239,88,17),(237,85,16),(236,83,15),(235,80,14),(234,78,13),(232,75,12),(231,73,12),(229,71,11),(228,69,10),(226,67,10),(225,65,9),(223,63,8),(221,61,8),(220,59,7),(218,57,7),(216,55,6),(214,53,6),(212,51,5),(210,49,5),(208,47,5),(206,45,4),(204,43,4),(202,42,4),(200,40,3),(197,38,3),(195,37,3),(193,35,2),(190,33,2),(188,32,2),(185,30,2),(183,29,2),(180,27,1),(178,26,1),(175,24,1),(172,23,1),(169,22,1),(167,20,1),(164,19,1),(161,18,1),(158,16,1),(155,15,1),(152,14,1),(149,13,1),(146,11,1),(142,10,1),(139,9,2),(136,8,2),(133,7,2),(129,6,2),(126,5,2),(122,4,3)];    
    for c in data.iter() {
        result.push(vec![c.0, c.1, c.2]);
    }

    result
}

#[test]
fn test_gen_colormap() {
    let res = gen_colormap();
    assert_eq!(res.len(), 256);
    println!("{:?}", res);
}


fn write_image(filename: &str, pixels: &[Vec<u8>], bounds: (usize, usize)) -> Result<(), Box<dyn std::error::Error>> {
    
    let mut image: image::RgbImage = image::ImageBuffer::new(bounds.0 as u32, bounds.1 as u32);
    for (i,x) in image.pixels_mut().enumerate() {
        *x = image::Rgb::<u8>([pixels[i][0], pixels[i][1], pixels[i][2]]);
    }
    image.save(filename)?;

    Ok(())
}

fn escape_time(c: Complex64, lim: &u64) -> Option<u64> {
    let mut z = Complex64::new(0.0, 0.0);
    for i in 0..*lim {
        z = z * z + c;
        
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
    }
    None
}


fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    match s.find(separator) {
        None => None,
        Some(index) => {
            match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
                (Ok(l), Ok(r)) => Some((l, r)),
                _ => None
            }
        }
    }
}

fn parse_complex(s: &str) -> Option<Complex64> {
    match parse_pair(s, ',') {
        Some((re, im)) => Some(Complex64::new(re, im)),
        None => None
    }
}

fn pixel_to_point(bounds: (usize, usize), pixel: (usize, usize), upper_left: Complex64, lower_right: Complex64) -> Complex64 {
    let (width, height) = (lower_right.re - upper_left.re, upper_left.im - lower_right.im);
    Complex64::new(
        upper_left.re + pixel.0 as f64 * width / bounds.0 as f64,
        upper_left.im - pixel.1 as f64 * height / bounds.1 as f64,
    )
}

#[test]
fn test_parse_pair() {
    assert_eq!(parse_pair::<i32>("",        ','), None);
    assert_eq!(parse_pair::<i32>("10,",     ','), None);
    assert_eq!(parse_pair::<i32>(",10",     ','), None);
    assert_eq!(parse_pair::<i32>("10,20",   ','), Some((10, 20)));
    assert_eq!(parse_pair::<i32>("10,20xy", ','), None);
    assert_eq!(parse_pair::<f64>("0.5x",    'x'), None);
    assert_eq!(parse_pair::<f64>("0.5x1.5", 'x'), Some((0.5, 1.5)));
}

#[test]
fn test_parse_complex() {
    assert_eq!(parse_complex("1.05,2.51"),Some(Complex64::new(1.05,2.51)));
    assert_eq!(parse_complex("1.05:2.51"),None);
}

#[test]
fn test_pixel_to_point() {
    assert_eq!(pixel_to_point((100,100), (25, 75), Complex64::new(-1.0, 1.0), Complex::new(1.0, -1.0)), Complex64::new(-0.5, -0.5));
}