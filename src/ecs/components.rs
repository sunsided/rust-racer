//! The data behind each component in `ecs.yaml`; sillyecs wraps each `*Data` in a
//! `*Component`.

mod car_model;
mod driver;
mod speed;
mod step_origin;
mod track_position;

pub use car_model::CarModelData;
pub use driver::DriverData;
pub use speed::SpeedData;
pub use step_origin::StepOriginData;
pub use track_position::TrackPositionData;
