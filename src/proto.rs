pub mod common {
    tonic::include_proto!("common");
}
pub mod seabird {
    tonic::include_proto!("seabird");
}

pub use self::common::*;
pub use self::seabird::*;
