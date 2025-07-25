use firestore_path::{CollectionName, DatabaseName, DocumentName};
pub use googleapis_tonic_google_firestore_v1::google;
pub use serde_firestore_value::from_value;
use std::{str::FromStr as _, sync::Arc};

type MyInterceptor =
    Box<dyn FnMut(tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> + Send + Sync>;
type Client =
    googleapis_tonic_google_firestore_v1::google::firestore::v1::firestore_client::FirestoreClient<
        tonic::service::interceptor::InterceptedService<tonic::transport::Channel, MyInterceptor>,
    >;

#[derive(Debug, thiserror::Error)]
#[error("error")]
pub struct Error(#[source] Box<dyn std::error::Error + Send + Sync>);

#[derive(Clone)]
pub struct FirestoreClient {
    channel: tonic::transport::Channel,
    database_name: firestore_path::DatabaseName,
    token_source: Arc<dyn token_source::TokenSource>,
}

impl FirestoreClient {
    pub async fn new() -> Result<Self, Error> {
        let default_token_source_provider = gcloud_auth::token::DefaultTokenSourceProvider::new(
            gcloud_auth::project::Config::default().with_scopes(&[
                "https://www.googleapis.com/auth/cloud-platform",
                "https://www.googleapis.com/auth/datastore",
            ]),
        )
        .await
        .map_err(Into::into)
        .map_err(Error)?;
        let token_source =
            token_source::TokenSourceProvider::token_source(&default_token_source_provider);
        let project_id = default_token_source_provider
            .project_id
            .ok_or("project_id not found")
            .map_err(Into::into)
            .map_err(Error)?;
        let channel = tonic::transport::Channel::from_static("https://firestore.googleapis.com")
            .tls_config(tonic::transport::ClientTlsConfig::new().with_webpki_roots())
            .map_err(Into::into)
            .map_err(Error)?
            .connect()
            .await
            .map_err(Into::into)
            .map_err(Error)?;
        let database_name = DatabaseName::from_project_id(project_id)
            .map_err(Into::into)
            .map_err(Error)?;
        Ok(Self {
            channel,
            database_name,
            token_source,
        })
    }

    pub async fn client(&self) -> Result<Client, Error> {
        let inner = self.channel.clone();
        let token = self
            .token_source
            .token()
            .await
            .map_err(Into::into)
            .map_err(Error)?;
        let mut metadata_value = tonic::metadata::AsciiMetadataValue::try_from(token)
            .map_err(Into::into)
            .map_err(Error)?;
        metadata_value.set_sensitive(true);
        let interceptor: MyInterceptor = Box::new(
            move |mut request: tonic::Request<()>| -> Result<tonic::Request<()>, tonic::Status> {
                request
                    .metadata_mut()
                    .insert("authorization", metadata_value.clone());
                Ok(request)
            },
        );
        let client = googleapis_tonic_google_firestore_v1::google::firestore::v1::firestore_client::FirestoreClient::with_interceptor(inner, interceptor);
        Ok(client)
    }

    pub fn database_name(&self) -> &DatabaseName {
        &self.database_name
    }

    /// TODO: support collection_path
    pub fn collection(&self, collection_id: &str) -> CollectionReference {
        CollectionReference {
            collection_name: self.database_name().collection(collection_id).unwrap(),
            firestore_client: self.clone(),
        }
    }
}

struct CollectionReference {
    collection_name: CollectionName,
    firestore_client: FirestoreClient,
}

impl CollectionReference {
    /// TODO: support document_path
    pub fn doc(&self, document_id: &str) -> DocumentReference {
        DocumentReference {
            document_name: self.collection_name.doc(document_id).unwrap(),
            firestore_client: self.firestore_client.clone(),
        }
    }

    pub fn id(&self) -> String {
        self.collection_name.collection_id().to_string()
    }

    pub async fn list_documents(&self) -> Result<Vec<DocumentReference>, Error> {
        // TODO: pagination
        let mut firestore_client = self.firestore_client.client().await.unwrap();
        let response = firestore_client
            .list_documents(google::firestore::v1::ListDocumentsRequest {
                parent: self.collection_name.parent().map_or_else(
                    || self.collection_name.root_document_name().to_string(),
                    |it| it.to_string(),
                ),
                page_size: 100, // Adjust as needed
                ..Default::default()
            })
            .await
            .map_err(Into::into)
            .map_err(Error)?;
        Ok(response
            .into_inner()
            .documents
            .into_iter()
            .map(|doc| DocumentReference {
                document_name: DocumentName::from_str(&doc.name).unwrap(),
                firestore_client: self.firestore_client.clone(),
            })
            .collect::<Vec<DocumentReference>>())
    }

    pub fn parent(&self) -> Option<DocumentReference> {
        self.collection_name
            .parent()
            .map(|parent| DocumentReference {
                document_name: parent,
                firestore_client: self.firestore_client.clone(),
            })
    }

    pub fn path(&self) -> String {
        self.collection_name.collection_path().to_string()
    }
}

struct DocumentReference {
    document_name: DocumentName,
    firestore_client: FirestoreClient,
}

impl DocumentReference {
    pub fn id(&self) -> String {
        self.document_name.document_id().to_string()
    }

    pub fn parent(&self) -> CollectionReference {
        CollectionReference {
            collection_name: self.document_name.parent(),
            firestore_client: self.firestore_client.clone(),
        }
    }

    pub fn path(&self) -> String {
        self.document_name.document_path().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test() -> anyhow::Result<()> {
        let firestore = FirestoreClient::new().await?;
        let collection_ref = firestore.collection("col");
        let document_refs = collection_ref.list_documents().await?;
        for document_ref in document_refs {
            assert_eq!(document_ref.parent().path(), collection_ref.path());
            assert_eq!(
                document_ref.path(),
                collection_ref.doc(&document_ref.id()).path()
            );
        }
        Ok(())
    }
}
