//! Types

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub struct Metadata {
    name: String,
    description: String,
    icons: Vec<String>,
    verification_url: Option<String>,
    /*
    redirect?: {
      native?: string;
      universal?: string;
    };
    */
}
