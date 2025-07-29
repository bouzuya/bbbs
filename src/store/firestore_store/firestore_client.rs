use firestore_path::{CollectionName, DatabaseName, DocumentName};
pub use googleapis_tonic_google_firestore_v1::google;
pub use serde_firestore_value::from_value;
use std::{hash::Hash, str::FromStr as _, sync::Arc};

type MyInterceptor =
    Box<dyn FnMut(tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> + Send + Sync>;
type Client =
    googleapis_tonic_google_firestore_v1::google::firestore::v1::firestore_client::FirestoreClient<
        tonic::service::interceptor::InterceptedService<tonic::transport::Channel, MyInterceptor>,
    >;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(InnerError);

impl From<InnerError> for Error {
    fn from(inner: InnerError) -> Self {
        Self(inner)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("error")]
enum InnerError {
    #[error("channel connect")]
    ChannelConnect(#[source] tonic::transport::Error),
    #[error("channel tls config")]
    ChannelTlsConfig(#[source] tonic::transport::Error),
    #[error("database name from project id")]
    DatabaseNameFromProjectId(#[source] firestore_path::Error),
    #[error("document reference create serialize")]
    DocumentReferenceCreateSerialize(#[source] serde_firestore_value::Error),
    #[error("document reference create create document")]
    DocumentReferenceCreateCreateDocument(#[source] tonic::Status),
    #[error("document reference delete delete document")]
    DocumentReferenceDeleteDeleteDocument(#[source] tonic::Status),
    #[error("document reference get deserialize")]
    DocumentReferenceGetDeserialize(#[source] serde_firestore_value::Error),
    #[error("document reference get get document")]
    DocumentReferenceGetGetDocument(#[source] tonic::Status),
    #[error("document reference list collections list collection ids")]
    DocumentReferenceListCollectionsListCollectionIds(#[source] tonic::Status),
    #[error("document reference set update document")]
    DocumentReferenceSetUpdateDocument(#[source] tonic::Status),
    #[error("invalid collection id")]
    InvalidCollectionId(#[source] firestore_path::Error),
    #[error("invalid document id")]
    InvalidDocumentId(#[source] firestore_path::Error),
    #[error("list documents")]
    ListDocuments(#[source] tonic::Status),
    #[error("list documents invalid document name")]
    ListDocumentsInvalidDocumentName(#[source] firestore_path::Error),
    #[error("metadata value try from")]
    MetadataValueTryFrom(#[source] tonic::metadata::errors::InvalidMetadataValue),
    #[error("new token source provider")]
    NewTokenSourceProvider(#[source] gcloud_auth::error::Error),
    #[error("no project id")]
    NoProjectId,
    #[error("token source token")]
    TokenSourceToken(#[source] Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Clone)]
pub struct Firestore {
    channel: tonic::transport::Channel,
    database_name: firestore_path::DatabaseName,
    token_source: Arc<dyn token_source::TokenSource>,
}

impl Firestore {
    pub async fn new() -> Result<Self, Error> {
        let default_token_source_provider = gcloud_auth::token::DefaultTokenSourceProvider::new(
            gcloud_auth::project::Config::default().with_scopes(&[
                "https://www.googleapis.com/auth/cloud-platform",
                "https://www.googleapis.com/auth/datastore",
            ]),
        )
        .await
        .map_err(InnerError::NewTokenSourceProvider)?;
        let token_source =
            token_source::TokenSourceProvider::token_source(&default_token_source_provider);
        let project_id = default_token_source_provider
            .project_id
            .ok_or(InnerError::NoProjectId)?;
        let channel = tonic::transport::Channel::from_static("https://firestore.googleapis.com")
            .tls_config(tonic::transport::ClientTlsConfig::new().with_webpki_roots())
            .map_err(InnerError::ChannelTlsConfig)?
            .connect()
            .await
            .map_err(InnerError::ChannelConnect)?;
        let database_name = DatabaseName::from_project_id(project_id)
            .map_err(InnerError::DatabaseNameFromProjectId)?;
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
            .map_err(InnerError::TokenSourceToken)?;
        let mut metadata_value = tonic::metadata::AsciiMetadataValue::try_from(token)
            .map_err(InnerError::MetadataValueTryFrom)?;
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
    pub fn collection(&self, collection_id: &str) -> Result<CollectionReference, Error> {
        Ok(CollectionReference {
            collection_name: self
                .database_name()
                .collection(collection_id)
                .map_err(InnerError::InvalidCollectionId)?,
            firestore: self.clone(),
        })
    }
}

struct CollectionReference {
    collection_name: CollectionName,
    firestore: Firestore,
}

impl CollectionReference {
    /// TODO: support document_path
    pub fn doc(&self, document_id: &str) -> Result<DocumentReference, Error> {
        Ok(DocumentReference {
            document_name: self
                .collection_name
                .doc(document_id)
                .map_err(InnerError::InvalidDocumentId)?,
            firestore: self.firestore.clone(),
        })
    }

    pub fn id(&self) -> String {
        self.collection_name.collection_id().to_string()
    }

    pub async fn list_documents(&self) -> Result<Vec<DocumentReference>, Error> {
        let mut firestore_client = self.firestore.client().await?;
        let google::firestore::v1::ListDocumentsResponse {
            documents,
            // TODO: pagination
            next_page_token: _,
        } = firestore_client
            .list_documents(google::firestore::v1::ListDocumentsRequest {
                parent: self.collection_name.parent().map_or_else(
                    || self.collection_name.root_document_name().to_string(),
                    |it| it.to_string(),
                ),
                page_size: 100, // Adjust as needed
                ..Default::default()
            })
            .await
            .map_err(InnerError::ListDocuments)?
            .into_inner();
        documents
            .into_iter()
            .map(|doc| -> Result<DocumentReference, Error> {
                Ok(DocumentReference {
                    document_name: DocumentName::from_str(&doc.name)
                        .map_err(InnerError::ListDocumentsInvalidDocumentName)?,
                    firestore: self.firestore.clone(),
                })
            })
            .collect::<Result<Vec<DocumentReference>, Error>>()
    }

    pub fn parent(&self) -> Option<DocumentReference> {
        self.collection_name
            .parent()
            .map(|parent| DocumentReference {
                document_name: parent,
                firestore: self.firestore.clone(),
            })
    }

    pub fn path(&self) -> String {
        self.collection_name.collection_path().to_string()
    }
}

struct Document<T> {
    create_time: serde_firestore_value::Timestamp,
    fields: T,
    name: String,
    update_time: serde_firestore_value::Timestamp,
}

struct DocumentReference {
    document_name: DocumentName,
    firestore: Firestore,
}

impl DocumentReference {
    // TODO: support collection_path
    pub fn collection(&self, collection_id: &str) -> Result<CollectionReference, Error> {
        Ok(CollectionReference {
            collection_name: self
                .document_name
                .collection(collection_id)
                .map_err(InnerError::InvalidCollectionId)?,
            firestore: self.firestore.clone(),
        })
    }

    /// TODO: support WriteResult
    pub async fn create<T>(&self, data: T) -> Result<(), Error>
    where
        T: serde::Serialize,
    {
        let mut firestore_client = self.firestore.client().await?;
        let value = serde_firestore_value::to_value(&data)
            .map_err(InnerError::DocumentReferenceCreateSerialize)?;
        let fields = match value.value_type.unwrap() {
            google::firestore::v1::value::ValueType::MapValue(map_value) => map_value.fields,
            _ => unreachable!(),
        };
        firestore_client
            .create_document(google::firestore::v1::CreateDocumentRequest {
                parent: self.document_name.parent().parent().map_or_else(
                    || self.document_name.root_document_name().to_string(),
                    |it| it.to_string(),
                ),
                collection_id: self.document_name.collection_id().to_string(),
                document_id: self.document_name.document_id().to_string(),
                document: Some(google::firestore::v1::Document {
                    name: "".to_owned(),
                    fields,
                    create_time: None,
                    update_time: None,
                }),
                mask: None,
            })
            .await
            .map_err(InnerError::DocumentReferenceCreateCreateDocument)?;
        Ok(())
    }

    /// TODO: support pre_condition
    pub async fn delete(&self) -> Result<(), Error> {
        let mut firestore_client = self.firestore.client().await?;
        firestore_client
            .delete_document(google::firestore::v1::DeleteDocumentRequest {
                name: self.document_name.to_string(),
                current_document: None,
            })
            .await
            .map_err(InnerError::DocumentReferenceDeleteDeleteDocument)?;
        Ok(())
    }

    /// Option<Document<T>> instead of DocumentSnapshot
    pub async fn get<T>(&self) -> Result<Option<Document<T>>, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut firestore_client = self.firestore.client().await?;
        let document = firestore_client
            .get_document(google::firestore::v1::GetDocumentRequest {
                name: self.document_name.to_string(),
                mask: None,
                consistency_selector: None,
            })
            .await
            .map(tonic::Response::into_inner)
            .map(Some)
            .or_else(|status| match status.code() {
                tonic::Code::NotFound => Ok(None),
                _ => Err(status),
            })
            .map_err(InnerError::DocumentReferenceGetGetDocument)?;
        Ok(document
            .map(
                |google::firestore::v1::Document {
                     name,
                     fields,
                     create_time,
                     update_time,
                 }| {
                    serde_firestore_value::from_value::<T>(&google::firestore::v1::Value {
                        value_type: Some(google::firestore::v1::value::ValueType::MapValue(
                            google::firestore::v1::MapValue { fields },
                        )),
                    })
                    .map(|fields| Document::<T> {
                        name,
                        fields,
                        create_time: serde_firestore_value::Timestamp::from(
                            create_time.expect("create_time is required"),
                        ),
                        update_time: serde_firestore_value::Timestamp::from(
                            update_time.expect("update_time is required"),
                        ),
                    })
                },
            )
            .transpose()
            .map_err(InnerError::DocumentReferenceGetDeserialize)?)
    }

    pub fn id(&self) -> String {
        self.document_name.document_id().to_string()
    }

    pub async fn list_collections(&self) -> Result<Vec<CollectionReference>, Error> {
        let mut firestore_client = self.firestore.client().await?;
        // TODO: support pagination
        let google::firestore::v1::ListCollectionIdsResponse {
            collection_ids,
            next_page_token: _,
        } = firestore_client
            .list_collection_ids(google::firestore::v1::ListCollectionIdsRequest {
                parent: self.document_name.to_string(),
                page_size: 100,
                page_token: "".to_owned(),
                consistency_selector: None,
            })
            .await
            .map_err(InnerError::DocumentReferenceListCollectionsListCollectionIds)?
            .into_inner();
        collection_ids
            .into_iter()
            .map(|collection_id| -> Result<CollectionReference, Error> {
                Ok(CollectionReference {
                    collection_name: self
                        .document_name
                        .collection(collection_id)
                        .map_err(InnerError::InvalidCollectionId)?,
                    firestore: self.firestore.clone(),
                })
            })
            .collect::<Result<Vec<CollectionReference>, Error>>()
    }

    pub fn parent(&self) -> CollectionReference {
        CollectionReference {
            collection_name: self.document_name.parent(),
            firestore: self.firestore.clone(),
        }
    }

    pub fn path(&self) -> String {
        self.document_name.document_path().to_string()
    }

    /// TODO: support WriteResult
    pub async fn set<T>(&self, data: T) -> Result<(), Error>
    where
        T: serde::Serialize,
    {
        let mut firestore_client = self.firestore.client().await?;
        let value = serde_firestore_value::to_value(&data)
            .map_err(InnerError::DocumentReferenceCreateSerialize)?;
        let fields = match value.value_type.unwrap() {
            google::firestore::v1::value::ValueType::MapValue(map_value) => map_value.fields,
            _ => unreachable!(),
        };
        firestore_client
            .update_document(google::firestore::v1::UpdateDocumentRequest {
                document: Some(google::firestore::v1::Document {
                    name: self.document_name.to_string(),
                    fields,
                    create_time: None,
                    update_time: None,
                }),
                update_mask: None,
                mask: None,
                current_document: None,
            })
            .await
            .map_err(InnerError::DocumentReferenceSetUpdateDocument)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rand::{Rng, distr::SampleString};

    use super::*;

    #[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    struct DocumentData {
        s: String,
        n: i64,
        b: bool,
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test() -> anyhow::Result<()> {
        let firestore = build_firestore().await?;
        let collection_ref = firestore.collection("col")?;
        let document_refs = collection_ref.list_documents().await?;
        for document_ref in document_refs {
            assert_eq!(document_ref.parent().path(), collection_ref.path());
            assert_eq!(
                document_ref.path(),
                collection_ref.doc(&document_ref.id())?.path()
            );
        }
        Ok(())
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_firestore_collection() -> anyhow::Result<()> {
        let firestore = build_firestore().await?;
        let collection_ref = firestore.collection("col")?;
        assert_eq!(collection_ref.path(), "col");
        Ok(())
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_collection_reference_doc() -> anyhow::Result<()> {
        let firestore = build_firestore().await?;
        let collection_ref = firestore.collection("col")?;
        let document_ref = collection_ref.doc("doc1")?;
        assert_eq!(document_ref.id(), "doc1");

        // TODO: support document_path
        Ok(())
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_collection_reference_id() -> anyhow::Result<()> {
        let firestore = build_firestore().await?;
        let collection_ref = firestore.collection("col")?;
        assert_eq!(collection_ref.id(), "col");
        // TODO: test collection_path
        Ok(())
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_collection_reference_list_documents() -> anyhow::Result<()> {
        let firestore = build_firestore().await?;
        let document_data1 = build_document_data();
        let document_data2 = build_document_data();

        let collection_ref = firestore.collection("col")?;

        // reset collection
        for document_ref in collection_ref.list_documents().await? {
            document_ref.delete().await?;
        }
        collection_ref.doc("doc1")?.set(&document_data1).await?;
        collection_ref.doc("doc2")?.set(&document_data2).await?;

        assert_eq!(
            collection_ref
                .list_documents()
                .await?
                .into_iter()
                .map(|it| it.path())
                .collect::<Vec<String>>(),
            vec![
                collection_ref.doc("doc1")?.path(),
                collection_ref.doc("doc2")?.path(),
            ]
        );
        Ok(())
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_collection_reference_parent() -> anyhow::Result<()> {
        let firestore = build_firestore().await?;
        let collection_ref = firestore.collection("col")?;
        assert_eq!(collection_ref.parent().map(|it| it.path()), None);

        let collection_ref = firestore
            .collection("col")?
            .doc("doc1")?
            .collection("col2")?;
        assert_eq!(
            collection_ref.parent().map(|it| it.path()),
            Some("col/doc1".to_owned())
        );
        Ok(())
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_collection_reference_path() -> anyhow::Result<()> {
        let firestore = build_firestore().await?;
        let collection_ref = firestore.collection("col")?;
        assert_eq!(collection_ref.path(), "col");

        // TODO: Use Firesstore::collection(collection_path)
        let collection_ref = firestore
            .collection("col1")?
            .doc("doc1")?
            .collection("col2")?;
        assert_eq!(collection_ref.path(), "col1/doc1/col2");
        Ok(())
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_document_reference_collection() -> anyhow::Result<()> {
        let firestore = build_firestore().await?;
        // TODO: Use Firesstore::doc(document_path)
        let document_ref = firestore.collection("col")?.doc("doc1")?;
        let collection_ref = document_ref.collection("col2")?;
        assert_eq!(collection_ref.path(), "col/doc1/col2");

        // TODO: support collection_path
        Ok(())
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_document_reference_create() -> anyhow::Result<()> {
        let firestore = build_firestore().await?;
        let document_data = build_document_data();

        // TODO: Use Firesstore::doc(document_path)
        let document_ref = firestore.collection("col")?.doc("doc1")?;

        // reset document
        document_ref.delete().await?;

        document_ref.create(&document_data).await?;

        // TODO: test write result

        assert_eq!(
            document_ref
                .get::<DocumentData>()
                .await?
                .map(|it| it.fields),
            Some(document_data)
        );
        Ok(())
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_document_reference_delete() -> anyhow::Result<()> {
        let firestore = build_firestore().await?;
        let document_data = build_document_data();
        let document_ref = firestore.collection("col")?.doc("doc1")?;

        // reset document
        document_ref.delete().await?;
        document_ref.create(&document_data).await?;

        // TODO: Use Firestore::doc(document_path)
        document_ref.delete().await?;

        // TODO: test write result
        assert_eq!(
            document_ref
                .get::<DocumentData>()
                .await?
                .map(|it| it.fields),
            None
        );
        Ok(())
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_document_reference_get() -> anyhow::Result<()> {
        let document_data = build_document_data();
        let firestore = build_firestore().await?;

        // TODO: Use Firestore::doc(document_path)
        let document_ref = firestore.collection("col")?.doc("doc1")?;
        document_ref.set(&document_data).await?;

        assert_eq!(
            document_ref
                .get::<DocumentData>()
                .await?
                .map(|it| it.fields),
            Some(document_data)
        );
        Ok(())
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_document_reference_id() -> anyhow::Result<()> {
        let firestore = build_firestore().await?;
        // TODO: Use Firestore::doc(document_path)
        let document_ref = firestore.collection("col")?.doc("doc1")?;
        assert_eq!(document_ref.id(), "doc1");

        // TODO: Use Firestore::doc(document_path)
        let document_ref = firestore
            .collection("col1")?
            .doc("doc1")?
            .collection("col2")?
            .doc("doc2")?;
        assert_eq!(document_ref.id(), "doc2");
        Ok(())
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_document_reference_list_collections() -> anyhow::Result<()> {
        let firestore = build_firestore().await?;
        let document_ref = firestore.collection("col")?.doc("doc1")?;
        document_ref.set(&build_document_data()).await?;
        document_ref
            .collection("col2")?
            .doc("doc2")?
            .set(&build_document_data())
            .await?;
        document_ref
            .collection("col3")?
            .doc("doc3")?
            .set(&build_document_data())
            .await?;
        let collections = document_ref.list_collections().await?;
        assert_eq!(
            collections
                .into_iter()
                .map(|it| it.path())
                .collect::<Vec<String>>(),
            vec!["col/doc1/col2".to_owned(), "col/doc1/col3".to_owned()]
        );
        Ok(())
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_document_reference_parent() -> anyhow::Result<()> {
        let firestore = build_firestore().await?;
        // TODO: Use Firesstore::doc(document_path)
        let document_ref = firestore.collection("col")?.doc("doc1")?;
        assert_eq!(document_ref.parent().path(), "col");

        // TODO: Use Firesstore::doc(document_path)
        let document_ref = firestore
            .collection("col1")?
            .doc("doc1")?
            .collection("col2")?
            .doc("doc2")?;
        assert_eq!(document_ref.parent().path(), "col1/doc1/col2");
        Ok(())
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_document_reference_path() -> anyhow::Result<()> {
        let firestore = build_firestore().await?;
        // TODO: Use Firesstore::doc(document_path)
        let document_ref = firestore.collection("col")?.doc("doc1")?;
        assert_eq!(document_ref.path(), "col/doc1");

        // TODO: Use Firestore::doc(document_path)
        let document_ref = firestore
            .collection("col1")?
            .doc("doc1")?
            .collection("col2")?
            .doc("doc2")?;
        assert_eq!(document_ref.path(), "col1/doc1/col2/doc2");
        Ok(())
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_document_reference_set() -> anyhow::Result<()> {
        let firestore = build_firestore().await?;
        let document_data = build_document_data();
        // TODO: Use Firestore::doc(document_path)
        let document_ref = firestore.collection("col")?.doc("doc1")?;
        document_ref.set(&document_data).await?;

        // TODO: test write result
        assert_eq!(
            document_ref
                .get::<DocumentData>()
                .await?
                .map(|it| it.fields),
            Some(document_data)
        );
        Ok(())
    }

    fn build_document_data() -> DocumentData {
        let mut rng = rand::rng();
        let len = rng.random_range(1..=100_usize);
        DocumentData {
            s: rand::distr::Alphabetic.sample_string(&mut rng, len),
            n: rng.random::<i64>(),
            b: rng.random::<bool>(),
        }
    }

    async fn build_firestore() -> Result<Firestore, Error> {
        Firestore::new().await
    }
}
