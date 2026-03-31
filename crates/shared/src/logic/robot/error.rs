use std::convert::Infallible;

pub trait RobotErrorHandler {
    type Error;

    fn handle(err: Box<dyn std::error::Error>) -> Result<(), Self::Error>;
}

pub struct InfallibleRobotErrorHandler;

impl RobotErrorHandler for InfallibleRobotErrorHandler {
    type Error = Infallible;

    fn handle(_err: Box<dyn std::error::Error>) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub struct PropagatingRobotErrorHandler;

impl RobotErrorHandler for PropagatingRobotErrorHandler {
    type Error = Box<dyn std::error::Error>;

    fn handle(err: Box<dyn std::error::Error>) -> Result<(), Self::Error> {
        Err(err)
    }
}
