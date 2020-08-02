#[derive(Clone, Debug, PartialEq)]
pub enum TrajCommand {
    Pos(i32, i32),
    Speed(i32),
    Angle(i32),
    VAngle(i32),
    Delay(u32),
    DestAngle(i32, i32),
}
