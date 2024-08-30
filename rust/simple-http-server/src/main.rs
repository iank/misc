use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;

use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::{Method, StatusCode};

use path_clean;

#[derive(Debug, PartialEq)]
enum PathError {
    OutOfBounds,
    NotFound,
}

async fn hello(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let raw_path = req.uri().path();
    println!("Request for {raw_path:#?}");
    match (req.method(), canonicalize_path(req.uri().path())) {
        (&Method::GET, Ok(path)) => {
            println!("Serving {path:#?}");
            Ok(Response::new(full(fs::read_to_string(path).unwrap())))
        }

        (&Method::GET, Err(PathError::OutOfBounds)) => {
            println!("Path out of bounds");
            let mut forbidden = Response::new(empty());
            *forbidden.status_mut() = StatusCode::FORBIDDEN;
            Ok(forbidden)
        }

        // Any other request type or GET, NotFound
        _ => {
            println!("Not found");
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

fn canonicalize_path(input: &str) -> Result<PathBuf, PathError> {
    let relative_path = if input.starts_with('/') {
        &input[1..]
    } else {
        input
    };

    let cwd = std::env::current_dir().map_err(|_| PathError::NotFound)?;
    let full_path = cwd.join(relative_path);

    // If we just canonicalize the full_path then we can't tell the difference
    // between a nonexistent path outside the cwd and one inside the cwd. We would
    // also return OutOfBounds if a symlink inside the cwd resolved to outside of it
    let clean_path = path_clean::clean(full_path);

    if clean_path.starts_with(&cwd) {
        match clean_path.canonicalize() {
            Ok(p) => Ok(p),
            Err(_) => Err(PathError::NotFound),
        }
    } else {
        Err(PathError::OutOfBounds)
    }
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // We create a TcpListener and bind it to 127.0.0.1:3000
    let listener = TcpListener::bind(addr).await?;

    // We start a loop to continuously accept incoming connections
    loop {
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits
        // as if they implement `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, service_fn(hello))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonicalize_path_with_valid_path_1() {
        let result = canonicalize_path("/src/main.rs");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.ends_with("src/main.rs"));
    }

    #[test]
    fn test_canonicalize_path_with_valid_path_2() {
        let result = canonicalize_path("/src/../src/main.rs");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.ends_with("src/main.rs"));
    }

    #[test]
    fn test_canonicalize_path_with_nonexistent_path() {
        let result = canonicalize_path("/src/dummy.rs");
        assert_eq!(result, Err(PathError::NotFound));
    }

    #[test]
    fn test_canonicalize_path_with_out_of_bounds_file_exists() {
        let result = canonicalize_path("/../../../../../etc/passwd");
        assert_eq!(result, Err(PathError::OutOfBounds));
    }

    #[test]
    fn test_canonicalize_path_with_out_of_bounds_nonexistent() {
        let result = canonicalize_path("/src/../../dummy.rs");
        assert_eq!(result, Err(PathError::OutOfBounds));
    }
}
