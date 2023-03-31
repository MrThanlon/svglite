use std::f64::consts::PI;

#[test]
fn test() {
    let y: f64 = -10.;
    let x: f64 = 0.; 
    assert_eq!(y.atan2(x), -PI / 2.);
}
