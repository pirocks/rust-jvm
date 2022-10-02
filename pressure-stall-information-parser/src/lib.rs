//https://docs.kernel.org/accounting/psi.html
//https://github.com/uprt/memory-pressure

pub struct PressureStallLine{
    avg10: f64,
    avg60: f64,
    avg300: f64,
    total: f64
}

pub struct PressureStallInformation{
    some:PressureStallLine,
    full:PressureStallLine
}



#[cfg(test)]
pub mod test{
    const SAMPLE: &str = "some avg10=0.00 avg60=0.00 avg300=0.00 total=0
full avg10=0.00 avg60=0.00 avg300=0.00 total=0";

    #[test]
    pub fn test_sample_parse() {

    }
}