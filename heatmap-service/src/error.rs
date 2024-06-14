use actix_http::StatusCode;
use actix_web::{error::InternalError, Error};

pub trait ActixMapResult<T> {
    fn actix_map_result(self) -> Result<T, Error>;
}

impl<T, E> ActixMapResult<T> for Result<T, E>
where
    E: 'static + std::error::Error,
{
    fn actix_map_result(self) -> Result<T, Error> {
        self.map_err(|error| InternalError::new(error, StatusCode::INTERNAL_SERVER_ERROR).into())
    }
}

pub trait ActixExpect<T> {
    fn actix_expect(self, reason: &'static str) -> Result<T, Error>;
}

impl<T> ActixExpect<T> for Option<T> {
    fn actix_expect(self, reason: &'static str) -> Result<T, Error> {
        self.ok_or_else(|| InternalError::new(reason, StatusCode::INTERNAL_SERVER_ERROR).into())
    }
}
