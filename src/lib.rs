//! # Gerber commands
//!
//! This crate implements the basic building blocks of Gerber (RS-274X, aka
//! Extended Gerber version 2) code. It focusses on the low level types and does
//! not do any semantic checking.
//!
//! For example, you can use an aperture without defining it. This will
//! generate syntactically valid but semantially invalid Gerber code, but this
//! module won't complain.
//!
//! Minimal required Rust version: 1.6.

extern crate chrono;
extern crate uuid;
extern crate conv;
#[macro_use] extern crate quick_error;

mod types;
mod attributes;
mod codegen;
mod coordinates;
mod errors;

pub use types::*;
pub use attributes::*;
pub use codegen::*;
pub use coordinates::*;
pub use errors::*;


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_to_code() {
        //! The to_code method of the GerberCode trait should generate strings.
        let comment = GCode::Comment("testcomment".to_string());
        assert_eq!(comment.to_code().unwrap(), "G04 testcomment *".to_string());
    }

    #[test]
    fn test_vec_to_code() {
        //! A `Vec<T: GerberCode>` should also implement `GerberCode`.
        let mut v = Vec::new();
        v.push(GCode::Comment("comment 1".to_string()));
        v.push(GCode::Comment("another one".to_string()));
        assert_eq!(v.to_code().unwrap(), "G04 comment 1 *\nG04 another one *".to_string());
    }

    #[test]
    fn test_function_code_to_code() {
        //! A `FunctionCode` should implement `GerberCode`
        let c = FunctionCode::GCode(GCode::Comment("comment".to_string()));
        assert_eq!(c.to_code().unwrap(), "G04 comment *");
    }

    #[test]
    fn test_command_to_code() {
        //! A `Command` should implement `GerberCode`
        let c = Command::FunctionCode(
            FunctionCode::GCode(
                GCode::Comment("comment".to_string())
            )
        );
        assert_eq!(c.to_code().unwrap(), "G04 comment *");
    }

    #[test]
    fn test_interpolation_mode() {
        let mut commands = Vec::new();
        let c1 = GCode::InterpolationMode(InterpolationMode::Linear);
        let c2 = GCode::InterpolationMode(InterpolationMode::ClockwiseCircular);
        let c3 = GCode::InterpolationMode(InterpolationMode::CounterclockwiseCircular);
        commands.push(c1);
        commands.push(c2);
        commands.push(c3);
        assert_eq!(commands.to_code().unwrap(), "G01*\nG02*\nG03*".to_string());
    }

    #[test]
    fn test_region_mode() {
        let mut commands = Vec::new();
        commands.push(GCode::RegionMode(true));
        commands.push(GCode::RegionMode(false));
        assert_eq!(commands.to_code().unwrap(), "G36*\nG37*".to_string());
    }

    #[test]
    fn test_quadrant_mode() {
        let mut commands = Vec::new();
        commands.push(GCode::QuadrantMode(QuadrantMode::Single));
        commands.push(GCode::QuadrantMode(QuadrantMode::Multi));
        assert_eq!(commands.to_code().unwrap(), "G74*\nG75*".to_string());
    }

    #[test]
    fn test_end_of_file() {
        let c = MCode::EndOfFile;
        assert_eq!(c.to_code().unwrap(), "M02*".to_string());
    }

    #[test]
    fn test_coordinates() {
        macro_rules! assert_coords {
            ($x:expr, $y:expr, $result:expr) => {{
                assert_eq!(Coordinates { x: $x, y: $y }.to_code().unwrap(), $result.to_string());
            }}
        }
        assert_coords!(Some(10), Some(20), "X10Y20");
        assert_coords!(None, None, ""); // TODO should we catch this?
        assert_coords!(Some(10), None, "X10");
        assert_coords!(None, Some(20), "Y20");
        assert_coords!(Some(0), Some(-400), "X0Y-400");
    }

    #[test]
    fn test_offset() {
        macro_rules! assert_coords {
            ($x:expr, $y:expr, $result:expr) => {{
                assert_eq!(CoordinateOffset { x: $x, y: $y }.to_code().unwrap(), $result.to_string());
            }}
        }
        assert_coords!(Some(10), Some(20), "I10J20");
        assert_coords!(None, None, ""); // TODO should we catch this?
        assert_coords!(Some(10), None, "I10");
        assert_coords!(None, Some(20), "J20");
        assert_coords!(Some(0), Some(-400), "I0J-400");
    }

    #[test]
    fn test_operation_interpolate() {
        let c1 = Operation::Interpolate(
            Coordinates::new(100, 200),
            Some(CoordinateOffset::new(5, 10))
        );
        assert_eq!(c1.to_code().unwrap(), "X100Y200I5J10D01*".to_string());
        let c2 = Operation::Interpolate(
            Coordinates::at_y(-200),
            None
        );
        assert_eq!(c2.to_code().unwrap(), "Y-200D01*".to_string());
        let c3 = Operation::Interpolate(
            Coordinates::at_x(1),
            Some(CoordinateOffset::at_y(2))
        );
        assert_eq!(c3.to_code().unwrap(), "X1J2D01*".to_string());
    }


    #[test]
    fn test_operation_move() {
        let c = Operation::Move(Coordinates::new(23, 42));
        assert_eq!(c.to_code().unwrap(), "X23Y42D02*".to_string());
    }

    #[test]
    fn test_operation_flash() {
        let c = Operation::Flash(Coordinates::new(23, 42));
        assert_eq!(c.to_code().unwrap(), "X23Y42D03*".to_string());
    }

    #[test]
    fn test_select_aperture() {
        let c1 = DCode::SelectAperture(10);
        assert_eq!(c1.to_code().unwrap(), "D10*".to_string());
        let c2 = DCode::SelectAperture(2147483647);
        assert_eq!(c2.to_code().unwrap(), "D2147483647*".to_string());
    }

    #[test]
    fn test_coordinate_format() {
        let c = ExtendedCode::CoordinateFormat(2, 5);
        assert_eq!(c.to_code().unwrap(), "%FSLAX25Y25*%".to_string());
    }

    #[test]
    fn test_unit() {
        let c1 = ExtendedCode::Unit(Unit::Millimeters);
        let c2 = ExtendedCode::Unit(Unit::Inches);
        assert_eq!(c1.to_code().unwrap(), "%MOMM*%".to_string());
        assert_eq!(c2.to_code().unwrap(), "%MOIN*%".to_string());
    }

    #[test]
    fn test_aperture_circle_definition() {
        let ad1 = ApertureDefinition {
            code: 10,
            aperture: Aperture::Circle(Circle { diameter: 4.0, hole_diameter: Some(2.0) }),
        };
        let ad2 = ApertureDefinition {
            code: 11,
            aperture: Aperture::Circle(Circle { diameter: 4.5, hole_diameter: None }),
        };
        assert_eq!(ad1.to_code().unwrap(), "10C,4X2".to_string());
        assert_eq!(ad2.to_code().unwrap(), "11C,4.5".to_string());
    }

    #[test]
    fn test_aperture_rectangular_definition() {
        let ad1 = ApertureDefinition {
            code: 12,
            aperture: Aperture::Rectangle(Rectangular { x: 1.5, y: 2.25, hole_diameter: Some(3.8) }),
        };
        let ad2 = ApertureDefinition {
            code: 13,
            aperture: Aperture::Rectangle(Rectangular { x: 1.0, y: 1.0, hole_diameter: None }),
        };
        let ad3 = ApertureDefinition {
            code: 14,
            aperture: Aperture::Obround(Rectangular { x: 2.0, y: 4.5, hole_diameter: None }),
        };
        assert_eq!(ad1.to_code().unwrap(), "12R,1.5X2.25X3.8".to_string());
        assert_eq!(ad2.to_code().unwrap(), "13R,1X1".to_string());
        assert_eq!(ad3.to_code().unwrap(), "14O,2X4.5".to_string());
    }

    #[test]
    fn test_aperture_polygon_definition() {
        let ad1 = ApertureDefinition {
            code: 15,
            aperture: Aperture::Polygon(Polygon { diameter: 4.5, vertices: 3, rotation: None, hole_diameter: None }),
        };
        let ad2 = ApertureDefinition {
            code: 16,
            aperture: Aperture::Polygon(Polygon { diameter: 5.0, vertices: 4, rotation: Some(30.6), hole_diameter: None }),
        };
        let ad3 = ApertureDefinition {
            code: 17,
            aperture: Aperture::Polygon(Polygon { diameter: 5.5, vertices: 5, rotation: None, hole_diameter: Some(1.8) }),
        };
        assert_eq!(ad1.to_code().unwrap(), "15P,4.5X3".to_string());
        assert_eq!(ad2.to_code().unwrap(), "16P,5X4X30.6".to_string());
        assert_eq!(ad3.to_code().unwrap(), "17P,5.5X5X0X1.8".to_string());
    }

    #[test]
    fn test_polarity_to_code() {
        let d = ExtendedCode::LoadPolarity(Polarity::Dark);
        let c = ExtendedCode::LoadPolarity(Polarity::Clear);
        assert_eq!(d.to_code().unwrap(), "%LPD*%".to_string());
        assert_eq!(c.to_code().unwrap(), "%LPC*%".to_string());
    }

    #[test]
    fn test_step_and_repeat_to_code() {
        let o = ExtendedCode::StepAndRepeat(StepAndRepeat::Open {
            repeat_x: 2, repeat_y: 3, distance_x: 2.0, distance_y: 3.0,
        });
        let c = ExtendedCode::StepAndRepeat(StepAndRepeat::Close);
        assert_eq!(o.to_code().unwrap(), "%SRX2Y3I2J3*%".to_string());
        assert_eq!(c.to_code().unwrap(), "%SR*%".to_string());
    }

    #[test]
    fn test_delete_attribute_to_code() {
        let d = ExtendedCode::DeleteAttribute("foo".into());
        assert_eq!(d.to_code().unwrap(), "%TDfoo*%".to_string());
    }

    #[test]
    fn test_file_attribute_to_code() {
        let a = ExtendedCode::FileAttribute(FileAttribute::Part(Part::Other("foo".into())));
        assert_eq!(a.to_code().unwrap(), "%TF.Part,Other,foo*%".to_string());
    }

}
