use std::fmt;

use futures::TryStreamExt;
use mongodb::{
    bson::{doc, Document},
    options::{ClientOptions, UpdateOptions, ServerApi, ServerApiVersion},
    Client, Collection,
};

use crate::config::ValueConfig;

#[derive(Debug)]
pub enum OpError {
    FailedConnection {
        message: String,
    },
    InvalidQuery {
        message: String,
    },
    InsertionError {
        message: String,
    },
    UpdateError {
        message: String,
    },
    DeletionError {
        message: String,
    },
    SearchError {
        message: String,
    },
}

impl fmt::Display for OpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpError::FailedConnection { message } => {
                write!(f, "Failed to a connection with MongoDb cluster | Error: {}", message)
            },
            OpError::InvalidQuery { message } => {
                write!(f, "Query was Invalid. Could not process it | Error: {}", message)
            },
            OpError::InsertionError { message } => {
                write!(f, "Failed to insert into collection | Error: {}", message)
            },
            OpError::UpdateError { message } => {
                write!(f, "Failed to update collection | Error: {}", message)
            },
            OpError::DeletionError { message } => {
                write!(f, "Failed to delete from collection | Error: {}", message)
            },
            OpError::SearchError { message } => {
                write!(f, "Invalid search arguments | Error: {}", message)
            },
        }
    }
}

/// Manages MongoDB Client
pub struct ClientManager {
    client: Client,
}

impl ClientManager {
    /// Creates a new MongoDB client from an environment variable or default URI.
    pub async fn new(value_config: &ValueConfig) -> Result<Self, OpError> {
        let uri = &value_config.database.uri;
        
        let mut client_options = ClientOptions::parse(uri)
            .await
            .map_err(|e| {
                return OpError::FailedConnection { 
                    message: e.to_string() 
                };
            })?;

        let server_api = ServerApi::builder()
        .version(ServerApiVersion::V1)
        .build();

        client_options.server_api = Some(server_api);
        
        // Get a handle to the cluster
        let client = Client::with_options(client_options)
        .map_err(|e| {
            return OpError::FailedConnection { 
                message: e.to_string() 
            };
        })?;
        
        // Ping the server to see if you can connect to the cluster
        client
            .database("admin")
            .run_command(doc! {"ping": 1}, None)
            .await
            .map_err(|e| {
                return OpError::FailedConnection { 
                    message: e.to_string() 
                };
            })?;

        println!("Pinged your deployment. You successfully connected to MongoDB cluster!");
        
        Ok(Self{client})
}

    /// Returns a reference to the MongoDB client
    pub fn get_client(&self) -> &Client {
        &self.client
    }
}

/// Handles Database Operations
pub struct DatabaseOps {
    collection: Collection<Document>,
}

impl DatabaseOps {
    /// Creates a new `DatabaseOps` instance
    pub fn new(client: &Client, database: &str, collection: &str) -> Self {
        let db = client.database(database);
        let collection = db.collection::<Document>(collection);
        Self { collection }
    }

    /// Inserts multiple documents into the collection
    pub async fn insert_many(&self, docs: Vec<Document>) -> Result<(), OpError> {
        match self.collection.insert_many(docs, None).await {
            Ok(_) => Ok(()),
            Err(e) => Err(OpError::InsertionError {
                message: format!("Failed to insert documents: {}", e),
            }),
        }
    }

    /// Updates multiple documents based on a filter
    pub async fn update_many(&self, filter: Document, update: Document) -> Result<(), OpError> {
        let update_doc = doc! { "$set": update };
        match self.collection.update_many(filter, update_doc, UpdateOptions::default()).await {
            Ok(_) => Ok(()),
            Err(e) => Err(OpError::UpdateError {
                message: format!("Failed to update documents: {}", e),
            }),
        }
    }

    /// Deletes multiple documents based on a filter
    pub async fn delete_many(&self, filter: Document) -> Result<(), OpError> {
        match self.collection.delete_many(filter, None).await {
            Ok(_) => Ok(()),
            Err(e) => Err(OpError::DeletionError {
                message: format!("Failed to delete documents: {}", e),
            }),
        }
    }

    /// Searches for documents matching a filter
    pub async fn search(&self, filter: Document) -> Result<Vec<Document>, OpError> {
        match self.collection.find(filter, None).await {
            Ok(mut cursor) => {
                let mut results = Vec::new();
                while let Some(doc) = cursor.try_next().await
                .map_err(|e| OpError::SearchError { message: 
                    format!("Failed to retrieve document: {}", e)
                })? {
                    results.push(doc);
                }
                Ok(results)
            }
            Err(e) => Err(OpError::SearchError {
                message: format!("Failed to search documents: {}", e),
            }),
        }
    }
}