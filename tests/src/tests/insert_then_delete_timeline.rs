use crate::tables::timelines::{
    insert::insert,
    select::select,
    delete::delete,
};
use anyhow::{Result, ensure, Context};
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

        let id = insert(&jwt, client)
            .await
            .context("Couldn't insert timeline\nIteration: {i}")?;
        let timeline = select(client, id)
            .await
            .context("Couldn't select timeline\nIteration: {i}")?;
        ensure!(
            timeline.id == id && timeline.author == TEST_USER_USER,
            "Test case didn't return the right timeline\nIteration: {i}\nTimeline: {timeline:?}\nExpected id: {id}\nExpected author: {TEST_USER_USER}"
        );
        delete(&jwt, client, id)
            .await
            .context("Couldn't delete timeline\nIteration: {i}")?;
    }

    cprintln!("<green>Success</green>");

    Ok(())
}
