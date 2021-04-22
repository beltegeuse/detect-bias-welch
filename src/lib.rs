use itertools::izip;

// Rust translation of "Detecting Bias in Monte Carlo Renderers using Welch’s t-test"
// url: http://www.jcgt.org/published/0009/02/01/
// Authors:
// - Alisa Jung
// - Johannes Hanika
// - Carsten Dachsbacher

/*
* === Useful resources:
*
* Sample Variance:
* http://www.statisticshowto.com/sample-variance/
*
* Welch test intro and example:
* https://www.youtube.com/watch?v=2-ecXltt2vI
* https://www.youtube.com/watch?v=gzrmHpA54Sc
*
* p-value table
* https://math.stackexchange.com/questions/808474/p-value-for-lower-upper-tailed-t-test
*/
pub type Float = f64;
pub fn cdf(t: Float, v: i32) -> Float {
    // Algorithm 3: The Integral of Student's t-distribution (Applied statistics 1968 vol 17 p 189, B.E. Cooper)
    let b = v as Float / (v as Float + t * t);
    let mut c = 1.0;
    let mut s = 1.0;
    let ioe = v as i32 % 2;
    let mut k = 2 + ioe;
    if v < 1 {
        return 0.0;
    }
    if v >= 4 {
        while k <= (v - 2) {
            c *= b - b / k as Float;
            s += c;
            k += 2;
        }
    }
    c = t / (v as Float).sqrt();
    if ioe != 1 {
        0.5 + 0.5 * (b).sqrt() * c * s
    } else {
        let v = if v == 1 { 0.0 } else { b * c * s };
        0.5 + (v + (c).atan()) / std::f64::consts::PI
    }
}

pub fn guass_cdf(t: Float) -> Float {
    let tt = if t > 0.0 { t } else { -t };
    let tt = tt / (2.0 as Float).sqrt();

    let x = 1.0 / (1.0 + 0.47047 * tt);
    let erf = 1.0 - x * (0.3480242 + x * (-0.0958798 + 0.7478556 * x)) * (-tt * tt).exp();
    1.0 - erf
}

pub fn compute_welch_t_test(
    welch_1_1: Vec<f32>,
    welch_1_2: Vec<f32>,
    welch_2_1: Vec<f32>,
    welch_2_2: Vec<f32>,
    spp_1: usize,
    spp_2: usize,
) -> Vec<Option<f32>> {
    let inv_spp_1 = 1.0 / spp_1 as f32;
    let inv_spp_2 = 1.0 / spp_2 as f32;

    assert!(spp_1 > 1);
    assert!(spp_2 > 1);

    // Safe guards
    assert!(welch_1_1.len() == welch_1_2.len());
    assert!(welch_2_2.len() == welch_2_1.len());
    assert!(welch_1_1.len() == welch_2_1.len());

    izip!(welch_1_1, welch_1_2, welch_2_1, welch_2_2)
        .map(|(w_1_1, w_1_2, w_2_1, w_2_2)| {
            // Compute variance
            let s_1_2 = (1.0 / (spp_1 as f32 - 1.0)) * (w_1_2 - inv_spp_1 * w_1_1.powi(2));
            let s_2_2 = (1.0 / (spp_2 as f32 - 1.0)) * (w_2_2 - inv_spp_2 * w_2_1.powi(2));

            // Sanity checks
            assert!(s_1_2 >= 0.0);
            assert!(s_2_2 >= 0.0);

            let tmp = s_1_2 * inv_spp_1 + s_2_2 * inv_spp_2;
            if tmp == 0.0 || !tmp.is_finite() {
                None
            } else {
                // Compute t
                let t = -(w_1_1 * inv_spp_1 - w_2_1 * inv_spp_2).abs() / tmp.sqrt();
                // Compute degree of freedom
                let nu = tmp.powi(2)
                    / (s_1_2.powi(2) * inv_spp_1.powi(2) / (spp_1 as f32 - 1.0)
                        + s_2_2.powi(2) * inv_spp_2.powi(2) / (spp_2 as f32 - 1.0));
                let nu = nu.round() as i32;

                // Get the P-value
                Some(2.0 * cdf(t as Float, nu) as f32)
            }
        })
        .collect::<Vec<Option<f32>>>()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
