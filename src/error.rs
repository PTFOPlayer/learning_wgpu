use std::io;
use wgpu::{CreateSurfaceError, RequestDeviceError};
use winit::error::{EventLoopError, OsError};

#[derive(Debug)]
pub enum Error {
    AdapterAquasitionError,
    DeviceCreationError(RequestDeviceError),
    EventLoopError(EventLoopError),
    ExecutionError,
    IoError,
    OsError(OsError),
    CreateSurfaceError(CreateSurfaceError),
}

impl From<io::Error> for Error {
    fn from(_: io::Error) -> Self {
        Error::IoError
    }
}

impl From<EventLoopError> for Error {
    fn from(value: EventLoopError) -> Self {
        Error::EventLoopError(value)
    }
}

impl From<RequestDeviceError> for Error {
    fn from(value: RequestDeviceError) -> Self {
        Error::DeviceCreationError(value)
    }
}

impl From<OsError> for Error {
    fn from(value: OsError) -> Self {
        Error::OsError(value)
    }
}

impl From<CreateSurfaceError> for Error {
    fn from(value: CreateSurfaceError) -> Self {
        Error::CreateSurfaceError(value)
    }
}
