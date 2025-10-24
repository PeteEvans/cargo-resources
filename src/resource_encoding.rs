/// What sort of file encoding the resource is using (i.e. text or binary)
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum ResourceEncoding {
    Txt,
    Bin,
}
