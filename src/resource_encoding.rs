/// What sort of file encoding the resource is using (i.e. text or binary)
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum ResourceEncoding {
    Txt,
    Bin,
}
