use crate::tables::timelines::{
    insert::insert,
    select::select,
    delete::delete,
};
use anyhow::{Result, ensure, bail};
use common::{consts::TEST_USER_USER, session::gen_jwt};

use color_print::cprintln;

const NUM_TESTS: usize = 1000;

pub async fn verify_insert_then_delete_timeline(
    client: &reqwest::Client,
) -> Result<()> {
    print!("Verify inserting a timeline then deleting it... ");

    let jwt = gen_jwt(TEST_USER_USER);

    for i in 0..NUM_TESTS {
        // println!("Using JWT: {:?}", jwt);

        match insert(&jwt, client).await {
            Ok(id) => {
                match select(client, id).await {
                    Ok(timeline) => {
                        ensure!(timeline.id == id && timeline.author == TEST_USER_USER, "Test case didn't return the right timeline\nIteration: {i}\nTimeline: {timeline:?}\nExpected id: {id}\nExpected author: {TEST_USER_USER}");
                        if let Err(e) = delete(&jwt, client, id).await {
                            bail!("Test case returned an error: {e:?}\nIteration: {i}");
                        }
                    }
                    Err(e) => bail!("Test case returned an error: {e}\nIteration: {i}"),
                }
            }
            Err(e) => bail!("Test case returned an error: {e:?}\nIteration: {i}"),
        }
    }

    cprintln!("<green>Success</green>");

    Ok(())
}
