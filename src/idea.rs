use actix_web::web::{Data, Json, Path};
use actix_web::{web, HttpResponse};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use diesel::result::Error;
use diesel::{ExpressionMethods, Insertable, Queryable, RunQueryDsl};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::constants::{APPLICATION_JSON, CONNECTION_POOL_ERROR};
use crate::like::{list_likes, Like};
use crate::response::Response;
use crate::{DBPool, DBPooledConnection};

use super::schema::ideas;
use diesel::query_dsl::methods::{FilterDsl, LimitDsl, OrderDsl};
use std::str::FromStr;

pub type Ideas = Response<Idea>;

#[derive(Debug, Deserialize, Serialize)]
pub struct Idea {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub message: String,
    pub image: String,
    pub likes: Vec<Like>,
}

impl Idea {
    pub fn new(message: String, image: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            message,
            image,
            likes: vec![],
        }
    }

    pub fn to_idea_db(&self) -> IdeaDB {
        IdeaDB {
            id: Uuid::new_v4(),
            created_at: Utc::now().naive_utc(),
            message: self.message.clone(),
        }
    }

    pub fn add_likes(&self, likes: Vec<Like>) -> Self {
        Self {
            id: self.id.clone(),
            created_at: self.created_at.clone(),
            message: self.message.clone(),
            likes,
        }
    }
}

#[table_name = "ideas"]
#[derive(Queryable, Insertable)]
pub struct IdeaDB {
    pub id: Uuid,
    pub created_at: NaiveDateTime,
    pub message: String,
}

impl IdeaDB {
    fn to_idea(&self) -> Idea {
        Idea {
            id: self.id.to_string(),
            created_at: Utc.from_utc_datetime(&self.created_at),
            message: self.message.clone(),
            likes: vec![],
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IdeaRequest {
    pub message: Option<String>,
}

impl IdeaRequest {
    pub fn to_idea(&self) -> Option<Idea> {
        match &self.message {
            Some(message) => Some(Idea::new(message.to_string())),
            None => None,
        }
    }
}

fn list_ideas(total_ideas: i64, conn: &DBPooledConnection) -> Result<Ideas, Error> {
    use crate::schema::ideas::dsl::*;

    let _ideas = match ideas
        .order(created_at.desc())
        .limit(total_ideas)
        .load::<IdeaDB>(conn)
    {
        Ok(tws) => tws,
        Err(_) => vec![],
    };

    Ok(Ideas {
        results: _ideas
            .into_iter()
            .map(|t| t.to_idea())
            .collect::<Vec<Idea>>(),
    })
}

fn find_idea(_id: Uuid, conn: &DBPooledConnection) -> Result<Idea, Error> {
    use crate::schema::ideas::dsl::*;

    let res = ideas.filter(id.eq(_id)).load::<IdeaDB>(conn);
    match res {
        Ok(ideas_db) => match ideas_db.first() {
            Some(idea_db) => Ok(idea_db.to_idea()),
            _ => Err(Error::NotFound),
        },
        Err(err) => Err(err),
    }
}

fn create_idea(idea: Idea, conn: &DBPooledConnection) -> Result<Idea, Error> {
    use crate::schema::ideas::dsl::*;

    let idea_db = idea.to_idea_db();
    let _ = diesel::insert_into(ideas).values(&idea_db).execute(conn);

    Ok(idea_db.to_idea())
}

fn delete_idea(_id: Uuid, conn: &DBPooledConnection) -> Result<(), Error> {
    use crate::schema::ideas::dsl::*;

    let res = diesel::delete(ideas.filter(id.eq(_id))).execute(conn);
    match res {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

/// list 50 last ideas `/ideas`
#[get("/ideas")]
pub async fn list(pool: Data<DBPool>) -> HttpResponse {
    let conn = pool.get().expect(CONNECTION_POOL_ERROR);
    let mut ideas = web::block(move || list_ideas(50, &conn)).await.unwrap();

    let conn = pool.get().expect(CONNECTION_POOL_ERROR);
    let ideas_with_likes = Ideas {
        results: ideas
            .results
            .iter_mut()
            .map(|t| {
                let _likes = list_likes(Uuid::from_str(t.id.as_str()).unwrap(), &conn).unwrap();
                t.add_likes(_likes.results)
            })
            .collect::<Vec<Idea>>(),
    };

    HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json(ideas_with_likes)
}

/// create a idea `/ideas`
#[post("/ideas")]
pub async fn create(idea_req: Json<IdeaRequest>, pool: Data<DBPool>) -> HttpResponse {
    let conn = pool.get().expect(CONNECTION_POOL_ERROR);

    let idea = web::block(move || create_idea(idea_req.to_idea().unwrap(), &conn)).await;

    match idea {
        Ok(idea) => HttpResponse::Created()
            .content_type(APPLICATION_JSON)
            .json(idea),
        _ => HttpResponse::NoContent().await.unwrap(),
    }
}

/// find a idea by its id `/ideas/{id}`
#[get("/ideas/{id}")]
pub async fn get(path: Path<(String,)>, pool: Data<DBPool>) -> HttpResponse {
    let conn = pool.get().expect(CONNECTION_POOL_ERROR);
    let idea =
        web::block(move || find_idea(Uuid::from_str(path.0.as_str()).unwrap(), &conn)).await;

    match idea {
        Ok(idea) => {
            let conn = pool.get().expect(CONNECTION_POOL_ERROR);
            let _likes = list_likes(Uuid::from_str(idea.id.as_str()).unwrap(), &conn).unwrap();

            HttpResponse::Ok()
                .content_type(APPLICATION_JSON)
                .json(idea.add_likes(_likes.results))
        }
        _ => HttpResponse::NoContent()
            .content_type(APPLICATION_JSON)
            .await
            .unwrap(),
    }
}

/// delete a idea by its id `/ideas/{id}`
#[delete("/ideas/{id}")]
pub async fn delete(path: Path<(String,)>, pool: Data<DBPool>) -> HttpResponse {
    // in any case return status 204
    let conn = pool.get().expect(CONNECTION_POOL_ERROR);

    let _ = web::block(move || delete_idea(Uuid::from_str(path.0.as_str()).unwrap(), &conn)).await;

    HttpResponse::NoContent()
        .content_type(APPLICATION_JSON)
        .await
        .unwrap()
}
