use hyper::{Method, Response, Body, Request, StatusCode};
use bytes::buf::BufExt;
use crate::schema::*;
use crate::cluster::Cluster;
use std::convert::TryFrom;
use std::sync::Arc;
use crate::error::AppError;

pub async fn request_routes(
    cluster: Arc<Cluster>,
    request: Request<Body>
) -> Result<Response<Body>, AppError> {
    let parts: Vec<&str> = request
        .uri()
        .path()
        .split("/")
        .filter(|part| !part.is_empty())
        .collect();
    match (request.method(), parts.as_slice()) {
        // {{{ v1
        (&Method::POST, ["scheduler", "api", "cron"]) => {
            info!("POST -> /scheduler/api/cron");
            let body = hyper::body::aggregate(request).await
                .map_err(|_| AppError::UnexpectedError)?;
            let v1_job: V1CronJob = serde_json::from_reader(body.reader())?;
            let job = Job::try_from(v1_job)?;
            cluster.push(job).await?;
            Ok(Response::new(Body::from("{}")))
        },
        (&Method::DELETE, ["scheduler", "api"]) => {
            info!("DELETE -> /scheduler/api");
            cluster.clear().await?;
            Ok(Response::new(Body::from("{}")))
        },
        (&Method::POST, ["scheduler", "api"]) => {
            info!("POST -> /scheduler/api");
            let body = hyper::body::aggregate(request).await
                .map_err(|_| AppError::UnexpectedError)?;
            let v1_job: V1Job = serde_json::from_reader(body.reader())?;
            let job = Job::from(v1_job);
            let key = V1JobKey::new(job.id.clone());

            cluster.push(job).await?;
            Ok(Response::new(Body::from(serde_json::to_string(&key)?)))
        },
        (&Method::DELETE, ["scheduler", "api", id]) => {
            info!("DELETE -> /scheduler/api/{}", id);
            let removed = cluster.remove(&id).await?;
            match removed {
                Some(_) => Ok(Response::new(Body::from("{}"))),
                None =>
                    Ok(
                        Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .header("Content-Type", "application/json")
                            .body(Body::from("{}"))
                            .unwrap()
                    )
            }
        },
        // }}}
        // {{{ v2
        (&Method::POST, ["api", "job"]) => {
            info!("POST -> /api/job");
            let body = hyper::body::aggregate(request).await.map_err(|_| AppError::UnexpectedError)?;
            let v2_job: V1Job = serde_json::from_reader(body.reader())?;
            let job = Job::from(v2_job);
            let response = serde_json::to_string(&V2JobResponse::from(&job))?;
            cluster.push(job).await?;
            Ok(Response::new(Body::from(response)))
        },
        (&Method::DELETE, ["api", "job"]) => {
            info!("DELETE -> /scheduler/api");
            cluster.clear().await?;
            Ok(
                Response::builder()
                    .status(StatusCode::NO_CONTENT)
                    .body(Body::from(""))
                    .unwrap()
            )
        },
        (&Method::DELETE, ["api", "job", id]) => {
            info!("DELETE -> /api/job/{}", id);
            let removed = cluster.remove(&id).await?;
            match removed {
                Some(_) => Ok(
                    Response::builder()
                        .status(StatusCode::NO_CONTENT)
                        .body(Body::from(""))
                        .unwrap()
                ),
                None =>
                    Ok(
                        Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(Body::from(""))
                            .unwrap()
                    )
            }
        },
        (&Method::POST, ["api", "cron"]) => {
            info!("POST -> /api/cron");
            let body = hyper::body::aggregate(request).await.map_err(|_| AppError::UnexpectedError)?;
            let v2_job: V2CronJob = serde_json::from_reader(body.reader())?;
            let job = Job::try_from(v2_job)?;
            let response = serde_json::to_string(&V2CronJobResponse::from(&job))?;
            cluster.push(job).await?;
            Ok(Response::new(Body::from(response)))
        },
        // }}}
        (method, parts) => {
            info!("{} -> {}: NOT_FOUND", method, parts.join("/"));
            Ok(
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header("Content-Type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap()
            )
        }
    }
}

pub async fn handle_request(
        cluster: Arc<Cluster>,
        request: Request<Body>
        ) -> Result<Response<Body>, AppError> {
    let result = request_routes(cluster, request).await;
    result.or_else(|err| {
        error!("Error: {}", err);
        let code = match err {
            AppError::ValidationError => StatusCode::BAD_REQUEST,
            AppError::UnexpectedError => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NodeUnreachable => StatusCode::SERVICE_UNAVAILABLE,
            AppError::RpcDeserializationError => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::UnexpectedRpcError(message) => {
                error!("RpcError - {}", message);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        Ok(
            Response::builder()
            .status(code)
            .header("Content-Type", "application/json")
            .body(Body::from("{}"))
            .unwrap()
        )
    })
}
