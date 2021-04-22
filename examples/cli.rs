#[macro_use]
extern crate clap;
#[macro_use]
extern crate anyhow;

use anyhow::Result;
use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use clap::{App, Arg};
use std::io::BufRead;

use plotters::prelude::*;

fn load_pfm(filename: &str) -> ((usize, usize), Vec<f32>) {
    let f = std::fs::File::open(std::path::Path::new(filename)).unwrap();
    let mut f = std::io::BufReader::new(f);
    // Check the flag
    {
        let mut header_str = String::new();
        f.read_line(&mut header_str).unwrap();
        if header_str != "PF\n" {
            panic!("Wrong PF flag encounter");
        }
    }
    // Check the dim
    let size = {
        let mut img_dim = String::new();
        f.read_line(&mut img_dim).unwrap();
        let img_dim = img_dim
            .split(" ")
            .map(|v| v.trim().parse::<usize>().unwrap())
            .collect::<Vec<_>>();
        assert!(img_dim.len() == 2);
        (img_dim[0], img_dim[1])
    };

    // Check the encoding
    {
        let mut encoding = String::new();
        f.read_line(&mut encoding).unwrap();
        let encoding = encoding.trim().parse::<f32>().unwrap();
        assert!(encoding == -1.0);
    }

    let mut colors = vec![0.0; (size.0 * size.1) * 3];
    for y in 0..size.1 {
        for x in 0..size.0 {
            let index = (size.1 - y - 1) * size.0 + x;
            colors[3 * index] = f.read_f32::<LittleEndian>().unwrap();
            colors[3 * index + 1] = f.read_f32::<LittleEndian>().unwrap();
            colors[3 * index + 2] = f.read_f32::<LittleEndian>().unwrap();
        }
    }

    (size, colors)
}

fn viridis_quintic(x: f32) -> (f32, f32, f32) // output colour ramp
{
    let x = x.min(1.0);
    let x2 = x * x;
    let x3 = x2 * x;
    let x4 = x2 * x2;
    let x5 = x3 * x2;
    let r = 0.280268003 - 0.143510503 * x + 2.225793877 * x2 - 14.815088879 * x3
        + 25.212752309 * x4
        - 11.772589584 * x5;
    let g = -0.002117546 + 1.617109353 * x - 1.909305070 * x2 + 2.701152864 * x3 - 1.685288385 * x4
        + 0.178738871 * x5;
    let b = 0.300805501 + 2.614650302 * x - 12.019139090 * x2 + 28.933559110 * x3
        - 33.491294770 * x4
        + 13.762053843 * x5;
    (r, g, b)
}

fn main() -> Result<()> {
    let matches = App::new("detect-bias-welch")
        .version("0.1.0")
        .about("Detecting bias with Welch's t-test")
        .arg(
            Arg::with_name("img_1_1")
                .required(true)
                .takes_value(true)
                .index(1)
                .help("First image containing sum"),
        )
        .arg(
            Arg::with_name("img_1_2")
                .required(true)
                .takes_value(true)
                .index(2)
                .help("First image containing sum of squared element"),
        )
        .arg(
            Arg::with_name("img_1_spp")
                .required(true)
                .takes_value(true)
                .index(3)
                .help("image 1 number of samples"),
        )
        .arg(
            Arg::with_name("img_2_1")
                .required(true)
                .takes_value(true)
                .index(4)
                .help("Second image containing sum"),
        )
        .arg(
            Arg::with_name("img_2_2")
                .required(true)
                .takes_value(true)
                .index(5)
                .help("Second image containing sum of squared element"),
        )
        .arg(
            Arg::with_name("img_2_spp")
                .required(true)
                .takes_value(true)
                .index(6)
                .help("image 1 number of samples"),
        )
        .arg(
            Arg::with_name("scale")
                .short("s")
                .default_value("1.0")
                .takes_value(true)
                .help("scale output image"),
        )
        .arg(
            Arg::with_name("histogram")
                .short("d")
                .takes_value(true)
                .help("output histogram (.png)"),
        )
        .arg(
            Arg::with_name("output")
                .short("output")
                .takes_value(true)
                .help("output image (.png)"),
        )
        .get_matches();

    // Get parameter values
    let spp_1 = value_t_or_exit!(matches.value_of("img_1_spp"), usize);
    let welch_1_1_path = value_t_or_exit!(matches.value_of("img_1_1"), String);
    let welch_1_2_path = value_t_or_exit!(matches.value_of("img_1_2"), String);

    let spp_2 = value_t_or_exit!(matches.value_of("img_2_spp"), usize);
    let welch_2_1_path = value_t_or_exit!(matches.value_of("img_2_1"), String);
    let welch_2_2_path = value_t_or_exit!(matches.value_of("img_2_2"), String);

    // Read images
    let (size_welch_1_1, welch_1_1) = load_pfm(&welch_1_1_path);
    let (size_welch_2_1, welch_2_1) = load_pfm(&welch_2_1_path);
    let (size_welch_1_2, welch_1_2) = load_pfm(&welch_1_2_path);
    let (size_welch_2_2, welch_2_2) = load_pfm(&welch_2_2_path);

    // Check images have the same size
    if size_welch_1_1.0 != size_welch_2_1.0 || size_welch_1_1.1 != size_welch_2_1.1 {
        bail!(
            "Image size do not match (for sum images): {:?} != {:?}",
            size_welch_1_1,
            size_welch_2_1
        );
    }
    if size_welch_1_2.0 != size_welch_2_2.0 || size_welch_1_2.1 != size_welch_2_2.1 {
        bail!(
            "Image size do not match (for squared sum images): {:?} != {:?}",
            size_welch_1_2,
            size_welch_2_2
        );
    }
    if size_welch_1_2.0 != size_welch_2_1.0 || size_welch_1_2.1 != size_welch_2_1.1 {
        bail!(
            "Image size do not match (across images squared and sum one): {:?} != {:?}",
            size_welch_1_2,
            size_welch_2_1
        );
    }
    if spp_1 <= 1 {
        bail!("Image 1 needs at least 2 spp (spp: {}", spp_1);
    }
    if spp_2 <= 1 {
        bail!("Image 2 needs at least 2 spp (spp: {}", spp_2);
    }

    // Perform the Welch's t-test
    let p_values = detect_bias_welch::compute_welch_t_test(
        welch_1_1, welch_1_2, welch_2_1, welch_2_2, spp_1, spp_2,
    );

    let nb_value_valid = p_values.iter().filter(|c| c.is_some()).count();
    println!(
        "{}/{} Welch samples valid",
        nb_value_valid,
        size_welch_1_1.0 * size_welch_1_1.1 * 3
    );

    // Compute median and average
    {
        let mut value_sorted = p_values
            .iter()
            .filter(|c| c.is_some())
            .map(|v| v.unwrap())
            .collect::<Vec<_>>();
        value_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        println!(" - Median: {}", value_sorted[nb_value_valid / 2]);
        println!(
            " - Average: {}",
            value_sorted.iter().sum::<f32>() / nb_value_valid as f32
        );
    }

    if let Some(output_path) = matches.value_of("output") {
        // Generate PNG
        // For now we only use min to visualize the output
        let pixels = p_values[..]
            .chunks_exact(3)
            .map(|v| match v {
                [None, None, None] => vec![0, 0, 0],
                [r, g, b] => {
                    let r_v = r.unwrap_or(std::f32::MAX);
                    let g_v = g.unwrap_or(std::f32::MAX);
                    let b_v = b.unwrap_or(std::f32::MAX);
                    let v = r_v.min(g_v).min(b_v);
                    let p = viridis_quintic(v);
                    vec![
                        (p.0.min(1.0) * 255.0) as u8,
                        (p.1.min(1.0) * 255.0) as u8,
                        (p.2.min(1.0) * 255.0) as u8,
                    ]
                }
                _ => todo!(),
            })
            .flatten()
            .collect::<Vec<_>>();

        // Transform vec to image
        let image =
            image::RgbImage::from_vec(size_welch_1_1.0 as u32, size_welch_1_1.1 as u32, pixels)
                .unwrap();
        let image = image::DynamicImage::ImageRgb8(image);

        // Scale the image if necessary
        let scale = value_t_or_exit!(matches.value_of("scale"), f32);
        let image = if scale == 1.0 {
            image
        } else {
            image.resize(
                (size_welch_1_1.0 as f32 * scale) as u32,
                (size_welch_1_1.1 as f32 * scale) as u32,
                image::imageops::Nearest,
            )
        };

        // Save color image
        image.save(&std::path::Path::new(&output_path))?;
    }

    // Compute histogram of p-values if needed
    if let Some(outhist) = matches.value_of("histogram") {
        let root = BitMapBackend::new(outhist, (640, 480)).into_drawing_area();
        root.fill(&WHITE)?;

        // Compute the bin for the histogram
        let data = p_values
            .iter()
            .filter(|s| s.is_some())
            .map(|v| (v.unwrap() * 10.0).min(9.0) as u32)
            .collect::<Vec<u32>>();

        // Compute the biggest bin
        let mut nbs = vec![0; 10];
        for d in &data {
            nbs[*d as usize] += 1;
        }
        let nbs_max = *nbs.iter().max().unwrap() as u32;

        // Build Chart
        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(35)
            .y_label_area_size(40)
            .margin(5)
            .caption("Histogram P-values", ("sans-serif", 30))
            .build_cartesian_2d((0u32..9u32).into_segmented(), 0u32..nbs_max)?;

        chart
            .configure_mesh()
            .disable_x_mesh()
            .bold_line_style(&WHITE.mix(0.3))
            .y_desc("Count")
            .x_desc("P-values")
            .axis_desc_style(("sans-serif", 15))
            .x_label_formatter(&|v| match v {
                SegmentValue::CenterOf(v) => {
                    format!("{} - {}", *v as f32 / 10.0, (*v + 1) as f32 / 10.0).to_string()
                }
                SegmentValue::Exact(v) => {
                    format!("{} - {}", *v as f32 / 10.0, (*v + 1) as f32 / 10.0).to_string()
                }
                _ => todo!(),
            })
            .draw()?;

        chart.draw_series(
            Histogram::vertical(&chart)
                .style(BLUE.mix(0.5).filled())
                .data(data.iter().map(|x: &u32| (*x, 1))),
        )?;
    }

    Ok(())
}
