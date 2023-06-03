use tokio_postgres::types::ToSql;

/*
   This is based on the answer of this question:
   https://stackoverflow.com/a/46625932
*/
pub fn convert_args<'a, I, T: 'a + Sync>(things: I) -> Vec<&'a (dyn ToSql + Sync)>
where
    I: IntoIterator<Item = &'a T>,
    T: ToSql,
{
    things
        .into_iter()
        .map(|x| x as &(dyn ToSql + Sync))
        .collect()
}
