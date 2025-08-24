pub fn fetch_data_as_json<T>(uri: &str) -> Result<T, Box<dyn std::error::Error>>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let data = ureq::get(uri).call()?.into_body().read_json::<T>()?;
    Ok(data)
}
