use actix_web::{web, App, HttpServer, Result as ActixResult, HttpResponse, middleware::Logger};
use actix_cors::Cors;
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use sqlx::PgPool;
use anyhow::Result;

use super::schema::{create_schema, GraphQLSchema};

pub struct GraphQLServer {
    schema: GraphQLSchema,
    bind_address: String,
}

impl GraphQLServer {
    pub fn new(_pool: PgPool, bind_address: String) -> Self {
        let schema = create_schema();
        
        Self {
            schema,
            bind_address,
        }
    }
    
    pub async fn start(self, pool: PgPool) -> Result<()> {
        println!("🚀 GraphQL server starting at http://{}/graphql", self.bind_address);
        println!("📊 GraphQL playground available at http://{}/playground", self.bind_address);
        
        let schema = self.schema;
        
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(schema.clone()))
                .app_data(web::Data::new(pool.clone()))
                .wrap(
                    Cors::default()
                        .allow_any_origin()
                        .allow_any_method()
                        .allow_any_header()
                        .max_age(3600)
                )
                .wrap(Logger::default())
                .route("/graphql", web::post().to(graphql_handler))
                .route("/graphql", web::get().to(graphql_playground))
                .route("/playground", web::get().to(graphql_playground))
                .route("/health", web::get().to(health_check))
        })
        .bind(&self.bind_address)?
        .run()
        .await?;
        
        Ok(())
    }
}

async fn graphql_handler(
    schema: web::Data<GraphQLSchema>,
    pool: web::Data<PgPool>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let request = req.into_inner().data(pool.get_ref().clone());
    schema.execute(request).await.into()
}

async fn graphql_playground() -> ActixResult<HttpResponse> {
    let source = playground_source(GraphQLPlaygroundConfig::new("/graphql"));
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(source))
}

async fn health_check() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "metaforge-engine-graphql",
        "version": env!("CARGO_PKG_VERSION"),
        "endpoints": {
            "graphql": "/graphql",
            "playground": "/playground"
        }
    })))
}
